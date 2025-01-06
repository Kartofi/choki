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
        let mut total_read = 0;
        let mut buffer: Vec<u8> = Vec::new();
        let mut buffer2: [u8; 2048] = [0; 2048];

        loop {
            match bfreader.read(&mut buffer2) {
                Ok(size) => {
                    total_size += size;

                    println!(
                        "Read {} bytes, total: {} / expected: {}",
                        size,
                        total_size,
                        self.content_length - total_read
                    );

                    buffer.extend_from_slice(&buffer2[..size]);
                    if size == 0 || total_size >= self.content_length {
                        break; // End of file
                    }
                }
                Err(_) => break,
            }
        }
        let mut cleaned = buffer.to_vec();
        self.body = Some(cleaned);
    }
}
