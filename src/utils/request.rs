use std::{
    collections::HashMap,
    io::{BufReader, Read, Write},
    net::TcpStream,
};

use crate::{utils::structs::*, Encoding};

pub struct Request {
    pub query: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub cookies: Vec<Cookie>,
    pub user_agent: Option<String>,
    pub content_encoding: Option<Vec<Encoding>>,
    pub content_length: usize,
    // BODY
    pub content_type: Option<ContentType>,
    pub boudary: Option<String>,
    pub body: Option<Vec<BodyItem>>,
}
impl Request {
    pub fn new(
        query: HashMap<String, String>,
        params: HashMap<String, String>,
        cookies: Vec<Cookie>,
        user_agent: Option<String>,
        content_encoding: Option<Vec<Encoding>>,
        content_length: usize,

        content_type: Option<ContentType>,
        boudary: Option<String>,
        body: Option<Vec<BodyItem>>,
    ) -> Request {
        return Request {
            query: query,
            params: params,
            cookies: cookies,
            user_agent: user_agent,
            content_encoding: content_encoding,
            content_length: content_length,

            content_type: content_type,
            boudary: boudary,
            body: body,
        };
    }
    pub fn parse(
        lines: Vec<&str>,
        query: Option<HashMap<String, String>>,
        params: Option<HashMap<String, String>>,
    ) -> Request {
        let mut req = Request::new(
            query.unwrap_or_default(),
            params.unwrap_or_default(),
            Vec::new(),
            None,
            None,
            0,
            None,
            None,
            None,
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

                if parts.len() > 0 {
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
                        encodings
                            .push(Encoding::new(EncodingType::from_string(encoding_str), -1.0));
                    }
                }
                req.content_encoding = Some(encodings);
            } else if lower_line.starts_with("content-type:") {
                let content_type: String = extract_data(line, "Content-Type: ");
                let parts: Vec<&str> = content_type.split("; ").collect();
                if parts.len() > 0 {
                    req.content_type = Some(ContentType::from_string(parts[0]));
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
                        req.cookies.push(Cookie::new_simple(
                            cookie_parts[0].to_string(),
                            cookie_parts[1].to_string(),
                        ));
                    }
                }
            }
        }
        return req;
    }

    pub fn extract_body(&mut self, bfreader: &mut BufReader<TcpStream>) {
        let mut total_size = 0;

        let mut buffer: Vec<u8> = Vec::new();
        let mut buffer2: [u8; 4096] = [0; 4096];
        let mut body: Vec<BodyItem> = Vec::new();

        loop {
            match bfreader.read(&mut buffer2) {
                Ok(size) => {
                    total_size += size;
                    println!(
                        "Read {} bytes, total: {} / expected: {}",
                        size, total_size, self.content_length
                    );
                    buffer.extend_from_slice(&buffer2[..size]);
                    if size == 0 || total_size >= self.content_length {
                        break; // End of file
                    }
                }
                Err(_) => break,
            }
        }
        let content_type = self.content_type.as_ref().unwrap().clone();
        if content_type != ContentType::MultipartForm {
            body.push(BodyItem::new_simple(content_type, buffer));

            self.body = Some(body);
            return;
        }
        let boundary = (&self.boudary.clone().unwrap()).as_bytes().to_owned();

        replace_bytes(&mut buffer, "\r\n--".as_bytes(), "".as_bytes());
        replace_bytes(
            &mut buffer,
            "\r\nContent-Di".as_bytes(),
            "Content-Di".as_bytes(),
        );

        let mut segments: Vec<(usize, usize)> = Vec::new();

        for index in 0..buffer.len() {
            let mut first_match: usize = usize::MAX;

            let mut matches = 0;

            if index + boundary.len() > buffer.len() {
                break;
            }

            for index2 in index..index + boundary.len() {
                if boundary[matches] == buffer[index2] {
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

        let mut buff: Vec<Vec<u8>> = Vec::new();

        for (start, end) in segments {
            if start < end {
                buff = split_buffer(&buffer[start..end], "\r\n\r\n".as_bytes()).to_vec();
                println!("123{}", String::from_utf8_lossy(&buff[0]));
                BodyItem::from_str(&String::from_utf8_lossy(&buff[0]));
                /*  body.push(BodyItem::new_simple(
                    &String::from_utf8_lossy(&buff[0]),
                    buff[1].clone(),
                ));*/
            }
        }
        buff.clear();
        buffer.clear();
        for item in body.clone() {
            println!("{}", item.content_type.as_str());
        }

        self.body = Some(body);
    }
}
fn replace_bytes(buffer: &mut Vec<u8>, target: &[u8], replacement: &[u8]) {
    let mut i = 0;
    while i <= buffer.len() - target.len() {
        if &buffer[i..i + target.len()] == target {
            buffer.splice(i..i + target.len(), replacement.iter().cloned());
            i += replacement.len();
        } else {
            i += 1; // Move to the next byte
        }
    }
}
fn split_buffer(buffer: &[u8], delimiter: &[u8]) -> Vec<Vec<u8>> {
    let mut segments = Vec::new();
    let mut start = 0;

    let mut i = 0;
    while i <= buffer.len() - delimiter.len() {
        if &buffer[i..i + delimiter.len()] == delimiter {
            segments.push(buffer[start..i].to_vec());
            start = i + delimiter.len();
            i += delimiter.len();
        } else {
            i += 1;
        }
    }
    segments.push(buffer[start..].to_vec());

    segments
}
