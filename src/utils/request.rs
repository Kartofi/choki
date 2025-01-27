use std::{
    collections::HashMap,
    io::{BufReader, Read, Write},
    net::TcpStream,
};

use flate2::{write::GzEncoder, Compression};

use crate::{utils::structs::*, Encoding};

use super::utils::map_compression_level;

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
    pub body: Option<Vec<u8>>,
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
        body: Option<Vec<u8>>,
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
        if *self.content_type.as_ref().unwrap() != ContentType::MultipartForm {
            self.body = Some(buffer);
            return;
        }
        let boundary = (&self.boudary.clone().unwrap()).as_bytes().to_owned();

        let mut index = 0;

        let mut segments: Vec<(usize, usize)> = Vec::new();

        replace_bytes(&mut buffer, "\r\n--".as_bytes(), "".as_bytes());
        replace_bytes(
            &mut buffer,
            "\r\nContent-Di".as_bytes(),
            "Content-Di".as_bytes(),
        );

        for i in buffer.clone() {
            let mut matches: Vec<usize> = Vec::new();
            let mut ii = 0;

            if index + boundary.len() - 1 == buffer.len() {
                break;
            }

            for index2 in index..index + boundary.len() {
                if boundary[ii] == buffer[index2] {
                    matches.push(index2);
                    if matches.len() == boundary.len() {
                        if !segments.is_empty() {
                            segments.last_mut().unwrap().1 = matches[0];
                        }

                        segments.push((matches[0] + boundary.len(), 0));
                        break;
                    }
                    ii += 1;
                } else {
                    break;
                }
            }

            index += 1;
        }

        // Clean up the buffer by removing boundary sections and preserving the segments
        let mut cleaned2: Vec<Vec<Vec<u8>>> = Vec::new();

        // Extract segments between the boundaries into `cleaned2`
        for (start, end) in segments {
            if start < end {
                cleaned2.push(split_buffer(&buffer[start..end], "\r\n\r\n".as_bytes()).to_vec());
            }
        }

        // Debug outputs
        for cl in &cleaned2 {
            println!("{:?}", String::from_utf8_lossy(&cl[1]));
        }

        println!("--{}--", self.boudary.clone().unwrap());
        self.body = Some(buffer);
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
