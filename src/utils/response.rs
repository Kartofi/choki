use std::{
    io::{BufReader, Read, Write},
    net::TcpStream,
};

use flate2::{write::GzEncoder, Compression};

use crate::{utils::structs::*, Encoding};

use super::utils::map_compression_level;

pub struct Response {
    stream: TcpStream,
    cookies: Vec<Cookie>,
    headers: Vec<Header>,
    content_encoding: Vec<Encoding>,
    pub use_encoding: bool,
}
impl Response {
    pub fn new(stream: TcpStream, content_encoding: Option<Vec<Encoding>>) -> Response {
        return Response {
            stream: stream,
            cookies: Vec::new(),
            headers: Vec::new(),
            content_encoding: content_encoding.unwrap_or_default(),
            use_encoding: true,
        };
    }
    //Deletes a cookie
    pub fn delete_cookie(&mut self, name: &str) {
        self.cookies.push(Cookie {
            name: name.to_string(),
            value: "".to_string(),
            path: "/".to_string(),
            expires: "Thu, 01 Jan 1970 00:00:00 GMT".to_string(),
        });
    }
    //Creates/edits a cookie
    pub fn set_cookie(&mut self, cookie: &Cookie) {
        self.cookies.push(cookie.clone());
    }
    //Create/Delete Header
    pub fn set_header(&mut self, header: &Header) {
        self.headers.push(header.clone());
    }
    pub fn delete_header(&mut self, name: &str) {
        for i in 0..self.headers.len() {
            if self.headers[i].name == name {
                self.headers.remove(i);
                break;
            }
        }
    }
    // Encoding

    fn gzip_compress_data(&self, data: &[u8], compression_level: f32) -> Vec<u8> {
        let mut compression_level = compression_level;
        if compression_level == -1.0 {
            compression_level = 0.2;
        }
        let mut encoder = GzEncoder::new(
            Vec::new(),
            Compression::new(map_compression_level(compression_level)),
        );
        encoder.write_all(data).unwrap_or_default();
        let compressed_data = encoder.finish().unwrap_or_default();
        return compressed_data;
    }
    fn compress_data(&mut self, data: &[u8]) -> Vec<u8> {
        if self.content_encoding.len() == 0 {
            return data.to_vec();
        }
        let encoding = self.content_encoding[0].clone();
        if encoding.encoding_type == EncodingType::Any
            || encoding.encoding_type == EncodingType::Gzip
        {
            self.set_header(&Header::new(
                "Content-Encoding".to_owned(),
                EncodingType::Gzip.to_string(),
            ));
            return self.gzip_compress_data(data, encoding.quality);
        }

        return data.to_vec();
    }
    fn prepare_data(&mut self, data: &[u8]) -> Vec<u8> {
        if self.use_encoding == true {
            return self.compress_data(data);
        } else {
            return data.to_vec();
        }
    }
    ///Sends string as output.
    pub fn send_string_chunked(&mut self, data: &str) {
        self.send_bytes(data.as_bytes(), Some(ContentType::PlainText));
    }
    pub fn send_string(&mut self, data: &str) {
        let compressed_data = self.prepare_data(data.as_bytes());

        self.set_header(&Header::new(
            "Content-type".to_string(),
            ContentType::PlainText.as_str().to_string(),
        ));
        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);
        let headers_set_headers = Header::generate_headers(&self.headers);

        let response = "HTTP/1.1 200 OK".to_string()
            + &headers_set_headers
            + &cookies_set_headers
            + "\r\n\r\n";

        match self
            .stream
            .write_all(&[response.as_bytes(), &compressed_data].concat())
        {
            Ok(_res) => {}
            Err(_e) => {}
        }
    }
    ///Sends json as output.
    pub fn send_json_chunked(&mut self, data: &str) {
        self.send_bytes(data.as_bytes(), Some(ContentType::Json));
    }
    pub fn send_json(&mut self, data: &str) {
        let compressed_data = self.prepare_data(data.as_bytes());

        self.set_header(&Header::new(
            "Content-type".to_string(),
            ContentType::Json.as_str().to_string(),
        ));
        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);
        let headers_set_headers = Header::generate_headers(&self.headers);

        let response = "HTTP/1.1 200 OK".to_string()
            + &headers_set_headers
            + &cookies_set_headers
            + "\r\n\r\n";

        match self
            .stream
            .write_all(&[response.as_bytes(), &compressed_data].concat())
        {
            Ok(_res) => {}
            Err(_e) => {}
        }
    }
    //Sends raw bytes
    pub fn send_bytes(&mut self, data: &[u8], content_type: Option<ContentType>) {
        let content_type: ContentType = if !content_type.is_none() {
            content_type.unwrap()
        } else {
            ContentType::None
        };

        let compressed_data = self.prepare_data(data);

        self.headers.push(Header::new(
            "Content-type".to_string(),
            content_type.as_str().to_string(),
        ));
        self.headers.push(Header::new(
            "Content-length".to_string(),
            compressed_data.len().to_string(),
        ));
        self.headers.push(Header::new(
            "Transfer-Encoding".to_string(),
            "chunked".to_string(),
        ));
        self.headers.push(Header::new(
            "Connection".to_string(),
            "keep-alive".to_string(),
        ));

        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);

        let headers_set_headers = Header::generate_headers(&self.headers);

        let mut response =
            "HTTP/1.1 200 OK".to_owned() + &headers_set_headers + &cookies_set_headers;
        response = response.trim().to_owned();
        response += "\r\n\r\n";

        match self.stream.write_all(&response.as_bytes()) {
            Ok(_res) => {}
            Err(_e) => {}
        }
        // Define the chunk size
        const CHUNK_SIZE: usize = 1024;

        let mut start = 0;

        while start < compressed_data.len() {
            let end = (start + CHUNK_SIZE).min(compressed_data.len());
            let chunk = &compressed_data[start..end];

            // Write the chunk size in hexadecimal, followed by CRLF
            if let Err(e) = self
                .stream
                .write_all(format!("{:X}\r\n", chunk.len()).as_bytes())
            {
                eprintln!("Failed to write chunk size: {}", e);
                return;
            }

            // Write the chunk data, followed by CRLF
            if let Err(e) = self.stream.write_all(chunk) {
                eprintln!("Failed to write chunk data: {}", e);
                return;
            }

            if let Err(e) = self.stream.write_all(b"\r\n") {
                eprintln!("Failed to write chunk terminator: {}", e);
                return;
            }

            start = end;
        }

        if let Err(e) = self.stream.write_all(b"0\r\n\r\n") {
            eprintln!("Failed to write final chunk: {}", e);
            return;
        }

        if let Err(e) = self.stream.flush() {
            eprintln!("Failed to flush stream: {}", e);
        }
    }
    // Pipe a whole stream
    pub fn pipe_stream(
        &mut self,
        mut stream: BufReader<impl Read>,
        content_type: Option<ContentType>,
    ) {
        if let Some(ct) = content_type {
            self.headers.push(Header::new(
                "Content-Type".to_string(),
                ct.as_str().to_owned(),
            ));
        }
        self.headers.push(Header::new(
            "Transfer-Encoding".to_string(),
            "chunked".to_string(),
        ));
        self.headers.push(Header::new(
            "Connection".to_string(),
            "keep-alive".to_string(),
        ));

        let headers_set_headers = Header::generate_headers(&self.headers);
        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);

        let mut response =
            "HTTP/1.1 200 OK".to_owned() + &headers_set_headers + &cookies_set_headers;
        response = response.trim().to_owned();
        response += "\r\n\r\n";

        if let Err(e) = self.stream.write_all(response.as_bytes()) {
            eprintln!("Failed to write response headers: {}", e);
            return;
        }

        const CHUNK_SIZE: usize = 8192 * 2; // 16 KB chunk size
        let mut buffer = [0; CHUNK_SIZE];

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break, // EOF reached
                Ok(n) => {
                    if let Err(e) = self.stream.write_all(format!("{:X}\r\n", n).as_bytes()) {
                        eprintln!("Failed to write chunk size: {}", e);
                        return;
                    }

                    if let Err(e) = self.stream.write_all(&buffer[..n]) {
                        eprintln!("Failed to write chunk data: {}", e);
                        return;
                    }

                    if let Err(e) = self.stream.write_all(b"\r\n") {
                        eprintln!("Failed to write chunk terminator: {}", e);
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from stream: {}", e);
                    break;
                }
            }
        }

        if let Err(e) = self.stream.write_all(b"0\r\n\r\n") {
            eprintln!("Failed to write final chunk: {}", e);
            return;
        }

        if let Err(e) = self.stream.flush() {
            eprintln!("Failed to flush stream: {}", e);
        }
    }
    // Send Download
    pub fn send_download_bytes(&mut self, data: &[u8], file_name: &str) {
        self.use_encoding = false;
        self.headers.push(Header::new(
            "Content-Disposition".to_string(),
            "attachment; filename=".to_string() + file_name,
        ));
        self.send_bytes(data, Some(ContentType::OctetStream));
    }
    pub fn send_download_stream(&mut self, mut stream: BufReader<impl Read>, file_name: &str) {
        self.use_encoding = false;
        self.headers.push(Header::new(
            "Content-Disposition".to_string(),
            "attachment; filename=".to_string() + file_name,
        ));
        self.pipe_stream(stream, Some(ContentType::OctetStream));
    }
    //Sends a response code (404, 200...)
    pub fn send_code(&mut self, code: usize) {
        let mut response = "HTTP/1.1 ".to_owned()
            + &code.to_string()
            + (match code {
                100 => " Continue",
                404 => " NOT FOUND\r\n\r\nPAGE NOT FOUND",
                413 => " Content Too Large",
                _ => " OK\r\n\r\n",
            });
        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);
        let headers_set_headers = Header::generate_headers(&self.headers);
        response += &cookies_set_headers;
        response += &headers_set_headers;

        match self.stream.write_all(response.as_bytes()) {
            Ok(_res) => {}
            Err(_e) => {}
        }
    }
    // Get raw stream
    pub fn get_stream(&mut self) -> &TcpStream {
        return &self.stream;
    }
}
