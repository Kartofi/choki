use std::{ collections::HashMap, io::{ BufReader, Read, Write }, net::TcpStream };

use bumpalo::Bump;

use crate::{ src::structs::*, Encoding };

use super::utils::utils::{ replace_bytes, split_buffer_inxeses };

pub struct Request {
    pub query: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub headers: Vec<Header>,
    pub cookies: Vec<Cookie>,
    // User data
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub content_encoding: Option<Vec<Encoding>>,
    pub content_length: usize,
    // BODY
    pub content_type: Option<ContentType>,
    pub boudary: Option<String>,

    body: Vec<BodyItemInfo>,
    body_data_segments: Vec<(usize, usize)>,
    buffer: Vec<u8>,
}

impl Request {
    pub fn new(
        query: HashMap<String, String>,
        params: HashMap<String, String>,
        headers: Vec<Header>,
        cookies: Vec<Cookie>,

        ip: Option<String>,
        user_agent: Option<String>,
        content_encoding: Option<Vec<Encoding>>,
        content_length: usize,

        content_type: Option<ContentType>,
        boudary: Option<String>
    ) -> Request {
        return Request {
            query: query,
            params: params,
            headers: headers,
            cookies: cookies,

            ip: ip,
            user_agent: user_agent,
            content_encoding: content_encoding,
            content_length: content_length,

            content_type: content_type,
            boudary: boudary,

            body: Vec::new(),
            body_data_segments: Vec::new(),
            buffer: Vec::new(),
        };
    }
    pub fn parse(
        lines: &Vec<&str>,
        query: Option<HashMap<String, String>>,
        params: Option<HashMap<String, String>>
    ) -> Result<Request, HttpServerError> {
        let mut req = Request::new(
            query.unwrap_or_default(),
            params.unwrap_or_default(),
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            0,
            None,
            None
        );
        fn extract_data(input: &str, skip_text: &str) -> String {
            return input[skip_text.len()..].to_string();
        }
        for line in lines {
            let lower_line = line.to_lowercase();
            if lower_line.starts_with("user-agent:") {
                let user_agent = extract_data(line, "User-Agent: ");

                req.user_agent = Some(user_agent);
            } else if lower_line.starts_with("content-length:") {
                let parts: Vec<&str> = line.split(" ").collect();

                if parts.len() > 1 {
                    req.content_length = match parts[1].parse::<usize>() {
                        Ok(res) => res,
                        Err(err) => 0,
                    };
                }
            } else if lower_line.starts_with("accept-encoding:") {
                let content_encoding: String = extract_data(line, "Accept-Encoding: ");

                let mut encodings: Vec<Encoding> = Vec::new();
                let encodings_string: Vec<&str> = content_encoding.split(", ").collect();

                let mut encoding: Encoding = Encoding::new(EncodingType::Unknown, -1.0);

                for encoding_str in encodings_string {
                    if encoding_str.contains(";") {
                        let parts: Vec<&str> = encoding_str.split(";").collect();
                        encoding.encoding_type = EncodingType::from_string(parts[0]);
                        if parts.len() > 0 {
                            encoding.encoding_type = EncodingType::from_string(parts[0]);
                        }
                        if parts.len() > 1 {
                            encoding.quality = match parts[1].parse::<f32>() {
                                Ok(res) => res,
                                Err(err) => -1.0,
                            };
                        }
                        encodings.push(encoding.clone());
                    } else {
                        encodings.push(
                            Encoding::new(EncodingType::from_string(encoding_str), -1.0)
                        );
                    }
                }
                req.content_encoding = Some(encodings);
            } else if lower_line.starts_with("content-type:") {
                let content_type: String = extract_data(line, "Content-Type: ");
                let parts: Vec<&str> = content_type.split("; ").collect();
                if parts.len() > 0 {
                    req.content_type = Some(ContentType::from_string(parts[0])?);
                }
                if parts.len() > 1 {
                    let boudary = parts[1].replace("boundary=", "");
                    req.boudary = Some(boudary);
                }
            } else if lower_line.starts_with("cookie:") {
                let cookies_string = extract_data(line, "Cookie: ");
                let cookies: Vec<&str> = cookies_string.split("; ").collect(); // not cookie cuz small chars

                for cookie in cookies {
                    let cookie_parts: Vec<&str> = cookie.split("=").collect();
                    if cookie_parts.len() == 2 {
                        req.cookies.push(
                            Cookie::new_simple(
                                cookie_parts[0].to_string(),
                                cookie_parts[1].to_string()
                            )
                        );
                    }
                }
            } else {
                let mut parts: Vec<&str> = line.split(": ").collect();

                if parts.len() > 1 {
                    let key = parts[0].to_owned();
                    parts.remove(0);
                    let value = parts.join(": ");
                    req.headers.push(Header::new(&key, &value));
                }
            }
        }
        return Ok(req);
    }
    // Body Stuff
    pub fn body(&self) -> Vec<BodyItem> {
        if self.body.len() == 0 {
            return Vec::new();
        }

        let mut res: Vec<BodyItem> = Vec::new();

        let mut temp: &[u8] = &[];

        let mut index = 0;
        for i in 0..self.body.len() {
            let body_info = &self.body[i];
            if body_info.content_type == ContentType::UrlEncoded {
                res.push(body_info.to_body_item());
                continue;
            }
            if index < self.body_data_segments.len() {
                temp =
                    &self.buffer
                        [self.body_data_segments[index].0..self.body_data_segments[index].1];
            }

            index += 1;
            res.push(BodyItem::new(body_info, temp));
            temp = &[];
        }
        return res;
    }
    pub fn extract_body(
        &mut self,
        bfreader: &mut BufReader<TcpStream>,
        bump: Bump
    ) -> Result<bool, HttpServerError> {
        let mut total_size = 0;

        let mut buffer: [u8; 4096] = [0; 4096];

        self.buffer = bump.alloc(Vec::new()).to_vec();

        loop {
            match bfreader.read(&mut buffer) {
                Ok(size) => {
                    total_size += size;

                    self.buffer.extend_from_slice(&buffer[..size]);
                    if size == 0 || total_size >= self.content_length {
                        break; // End of file
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }

        let content_type = self.content_type.as_ref().unwrap().clone();
        if content_type == ContentType::UrlEncoded {
            let string_buffer = String::from_utf8_lossy(&self.buffer).to_string();

            let parts: Vec<&str> = string_buffer.split("&").collect();

            for part in parts {
                let key_value: Option<(&str, &str)> = part.split_once("=");
                if key_value.is_some() {
                    let key_value = key_value.unwrap();

                    let body_item = BodyItemInfo::new_url(
                        key_value.0.to_owned(),
                        key_value.1.to_owned()
                    );

                    self.body.push(body_item);
                }
            }
            return Ok(true);
        }
        if content_type != ContentType::MultipartForm {
            self.body.push(BodyItemInfo::new_simple(content_type));
            self.body_data_segments.push((0, self.buffer.len()));
            return Ok(true);
        }
        let boundary = (&self.boudary.clone().unwrap()).as_bytes().to_owned();

        replace_bytes(&mut self.buffer, "\r\n--".as_bytes(), "".as_bytes());
        replace_bytes(&mut self.buffer, "\r\nContent-Di".as_bytes(), "Content-Di".as_bytes());

        let mut segments: Vec<(usize, usize)> = Vec::new();

        for index in 0..self.buffer.len() {
            let mut first_match: usize = usize::MAX;

            let mut matches = 0;

            if index + boundary.len() > self.buffer.len() {
                break;
            }

            for index2 in index..index + boundary.len() {
                if boundary[matches] == self.buffer[index2] {
                    matches += 1;

                    if first_match == usize::MAX {
                        first_match = index2;
                    }

                    if matches == boundary.len() {
                        if !segments.is_empty() {
                            segments.last_mut().unwrap().1 = first_match;
                        }

                        segments.push((first_match + boundary.len(), 0));
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        let mut data_indexes: Vec<(usize, usize)> = Vec::new();
        let mut body_item_info: BodyItemInfo = BodyItemInfo::default();

        for (start, end) in segments {
            if start < end {
                data_indexes = split_buffer_inxeses(
                    &self.buffer[start..end],
                    "\r\n\r\n".as_bytes()
                );

                body_item_info = BodyItemInfo::from_str(
                    &String::from_utf8_lossy(
                        &self.buffer[data_indexes[0].0 + start..data_indexes[0].1 + start]
                    )
                )?;
                self.body_data_segments.push((
                    data_indexes[1].0 + start,
                    data_indexes[1].1 + start,
                ));

                self.body.push(body_item_info);

                //Empty data after use
                data_indexes.clear();
            }
        }
        return Ok(true);
    }
    pub fn read_only_body(&self, bfreader: &mut BufReader<TcpStream>) {
        let mut total_size = 0;
        let mut buffer: [u8; 4096] = [0; 4096];

        loop {
            match bfreader.read(&mut buffer) {
                Ok(size) => {
                    total_size += size;
                    if size == 0 || total_size >= self.content_length {
                        break; // End of file
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
    }
}
