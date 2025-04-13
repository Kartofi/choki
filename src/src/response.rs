use std::{ io::{ BufReader, Read, Write }, net::TcpStream, result };

use flate2::{ write::GzEncoder, Compression };

use crate::{ src::structs::*, Encoding };

use super::utils::utils::{ map_compression_level };

pub struct Response {
    stream: TcpStream,
    status_code: ResponseCode,

    cookies: Vec<Cookie>,
    headers: Vec<Header>,
    content_encoding: Vec<Encoding>,
    pub use_compression: bool,
}
impl Response {
    pub fn new(stream: TcpStream, content_encoding: Option<Vec<Encoding>>) -> Response {
        return Response {
            stream: stream,
            cookies: Vec::new(),
            headers: Vec::new(),
            content_encoding: content_encoding.unwrap_or_default(),
            use_compression: false,
            status_code: ResponseCode::Ok,
        };
    }
    /// Deletes a cookie
    pub fn delete_cookie(&mut self, name: &str) {
        self.cookies.push(Cookie {
            name: name.to_string(),
            value: "".to_string(),
            path: "/".to_string(),
            expires: "Thu, 01 Jan 1970 00:00:00 GMT".to_string(),
        });
    }
    /// Creates/edits a cookie
    pub fn set_cookie(&mut self, cookie: &Cookie) {
        self.cookies.push(cookie.clone());
    }
    /// Create Header
    pub fn set_header(&mut self, header: &Header) {
        self.headers.push(header.clone());
    }
    /// Delete Header
    pub fn delete_header(&mut self, name: &str) {
        for i in 0..self.headers.len() {
            if self.headers[i].name == name {
                self.headers.remove(i);
                break;
            }
        }
    }
    /// Sets response status default it OK(200)
    pub fn set_status(&mut self, status_code: &ResponseCode) {
        self.status_code = *status_code;
    }
    // Encoding

    fn gzip_compress_data(&self, data: &[u8], compression_level: f32) -> Vec<u8> {
        let mut compression_level = compression_level;
        if compression_level == -1.0 {
            compression_level = 0.2;
        }
        let mut encoder = GzEncoder::new(
            Vec::new(),
            Compression::new(map_compression_level(compression_level))
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
        if
            encoding.encoding_type == EncodingType::Any ||
            encoding.encoding_type == EncodingType::Gzip
        {
            self.set_header(&Header::new("Content-Encoding", &EncodingType::Gzip.to_string()));
            return self.gzip_compress_data(data, encoding.quality);
        }

        return data.to_vec();
    }
    fn prepare_data(&mut self, data: &[u8]) -> Vec<u8> {
        if self.use_compression == true {
            return self.compress_data(data);
        } else {
            return data.to_vec();
        }
    }
    /// Sends string as output in chunks.
    pub fn send_string_chunked(&mut self, data: &str) -> Result<(), HttpServerError> {
        self.send_bytes_chunked(data.as_bytes(), Some(ContentType::PlainText))
    }
    /// Sends string as output.
    pub fn send_string(&mut self, data: &str) -> Result<(), HttpServerError> {
        self.send_bytes(&data.as_bytes(), Some(ContentType::PlainText))
    }
    ///Sends json as output in chunks.
    pub fn send_json_chunked(&mut self, data: &str) -> Result<(), HttpServerError> {
        self.send_bytes_chunked(data.as_bytes(), Some(ContentType::Json))
    }
    ///Sends json as output.
    pub fn send_json(&mut self, data: &str) -> Result<(), HttpServerError> {
        self.send_bytes(&data.as_bytes(), Some(ContentType::Json))
    }
    /// Sends raw bytes.
    pub fn send_bytes(
        &mut self,
        data: &[u8],
        content_type: Option<ContentType>
    ) -> Result<(), HttpServerError> {
        let content_type: ContentType = if !content_type.is_none() {
            content_type.unwrap()
        } else {
            ContentType::None
        };

        let compressed_data = self.prepare_data(data);

        self.headers.push(Header::new("Content-type", content_type.as_str()));
        self.headers.push(Header::new("Content-Length", &compressed_data.len().to_string()));
        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);

        let headers_set_headers = Header::generate_headers(&self.headers);

        let mut response =
            "HTTP/1.1 ".to_owned() +
            &self.status_code.format_string() +
            &headers_set_headers +
            &cookies_set_headers;
        response = response.trim().to_owned();
        response += "\r\n\r\n";

        match self.stream.write_all(&response.as_bytes()) {
            Ok(_res) => {}
            Err(_e) => {}
        }
        match self.stream.write_all(&compressed_data) {
            Ok(_res) => {}
            Err(_e) => {}
        }
        if let Err(e) = self.stream.flush() {
            return Err(HttpServerError::new(&format!("Failed to flush stream: {}", e)));
        }
        Ok(())
    }
    /// Sends raw bytes in chunks.
    pub fn send_bytes_chunked(
        &mut self,
        data: &[u8],
        content_type: Option<ContentType>
    ) -> Result<(), HttpServerError> {
        let content_type: ContentType = if !content_type.is_none() {
            content_type.unwrap()
        } else {
            ContentType::None
        };

        let compressed_data = self.prepare_data(data);

        self.headers.push(Header::new("Content-type", content_type.as_str()));

        self.headers.push(Header::new("Transfer-Encoding", "chunked"));
        self.headers.push(Header::new("Connection", "keep-alive"));

        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);

        let headers_set_headers = Header::generate_headers(&self.headers);

        let mut response =
            "HTTP/1.1 ".to_owned() +
            &self.status_code.format_string() +
            &headers_set_headers +
            &cookies_set_headers;
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

            if let Err(e) = self.stream.write_all(format!("{:X}\r\n", chunk.len()).as_bytes()) {
                return Err(HttpServerError::new(&format!("Failed to write chunk size: {}", e)));
            }

            if let Err(e) = self.stream.write_all(chunk) {
                return Err(HttpServerError::new(&format!("Failed to write chunk data: {}", e)));
            }

            if let Err(e) = self.stream.write_all(b"\r\n") {
                return Err(
                    HttpServerError::new(&format!("Failed to write chunk terminator: {}", e))
                );
            }

            start = end;
        }

        if let Err(e) = self.stream.write_all(b"0\r\n\r\n") {
            return Err(HttpServerError::new(&format!("Failed to write final chunk: {}", e)));
        }

        if let Err(e) = self.stream.flush() {
            return Err(HttpServerError::new(&format!("Failed to flush stream: {}", e)));
        }
        Ok(())
    }
    /// Pipe a whole stream. Aka read everything from input stream and send it.
    pub fn pipe_stream(
        &mut self,
        mut stream: BufReader<impl Read>,
        content_type: Option<ContentType>,
        stream_size: Option<&u64>
    ) -> Result<(), HttpServerError> {
        if let Some(ct) = content_type {
            self.headers.push(Header::new("Content-Type", ct.as_str()));
        }
        if let Some(ss) = stream_size {
            self.headers.push(Header::new("Content-Length", &ss.to_string()));
        }
        self.headers.push(Header::new("Transfer-Encoding", "chunked"));
        self.headers.push(Header::new("Connection", "keep-alive"));

        let headers_set_headers = Header::generate_headers(&self.headers);
        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);

        let mut response =
            "HTTP/1.1 ".to_owned() +
            &self.status_code.format_string() +
            &headers_set_headers +
            &cookies_set_headers;
        response = response.trim().to_owned();
        response += "\r\n\r\n";

        if let Err(e) = self.stream.write_all(response.as_bytes()) {
            return Err(HttpServerError::new(&format!("Failed to write response headers: {}", e)));
        }

        const CHUNK_SIZE: usize = 8192 * 2; // 16 KB chunk size
        let mut buffer = [0; CHUNK_SIZE];
        let mut total_size: i64 = 0;
        let stream_size: i64 = if stream_size.is_some() {
            *stream_size.unwrap() as i64
        } else {
            -1
        };
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    break;
                } // EOF reached
                Ok(n) => {
                    if let Err(e) = self.stream.write_all(format!("{:X}\r\n", n).as_bytes()) {
                        return Err(
                            HttpServerError::new(&format!("Failed to write chunk size: {}", e))
                        );
                    }

                    if let Err(e) = self.stream.write_all(&buffer[..n]) {
                        return Err(
                            HttpServerError::new(&format!("Failed to write chunk data: {}", e))
                        );
                    }

                    if let Err(e) = self.stream.write_all(b"\r\n") {
                        return Err(
                            HttpServerError::new(
                                &format!("Failed to write chunk terminator: {}", e)
                            )
                        );
                    }
                    total_size += n as i64;
                }
                Err(e) => {
                    return Err(HttpServerError::new(&format!("Failed to read from stream: {}", e)));
                }
            }
            if total_size > stream_size && stream_size > 0 {
                break;
            }
        }

        if let Err(e) = self.stream.write_all(b"0\r\n\r\n") {
            return Err(HttpServerError::new(&format!("Failed to write final chunk: {}", e)));
        }

        if let Err(e) = self.stream.flush() {
            return Err(HttpServerError::new(&format!("Failed to flush stream: {}", e)));
        }
        Ok(())
    }
    /// Send Download bytes.
    pub fn send_download_bytes(
        &mut self,
        data: &[u8],
        file_name: &str
    ) -> Result<(), HttpServerError> {
        self.headers.push(
            Header::new("Content-Disposition", &("attachment; filename=".to_string() + file_name))
        );
        self.send_bytes_chunked(data, Some(ContentType::OctetStream))
    }
    /// Send Download stream.
    pub fn send_download_stream(
        &mut self,
        stream: BufReader<impl Read>,
        file_name: &str,
        file_size: Option<&u64>
    ) -> Result<(), HttpServerError> {
        self.headers.push(
            Header::new("Content-Disposition", &("attachment; filename=".to_string() + file_name))
        );

        self.pipe_stream(stream, Some(ContentType::OctetStream), file_size)
    }

    //./ Sends a response code (404, 200...)
    pub fn send_code(&mut self, code: ResponseCode) -> Result<(), HttpServerError> {
        let mut response = "HTTP/1.1 ".to_owned() + &code.format_string();

        self.set_header(&Header::new("Content-Type", "text/plain"));
        self.set_header(&Header::new("Content-Length", &code.to_desc().len().to_string()));
        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);
        let headers_set_headers = Header::generate_headers(&self.headers);
        response += &headers_set_headers;
        response += &cookies_set_headers;

        response += &("\r\n\r\n".to_owned() + &code.to_desc());
        match self.stream.write_all(response.as_bytes()) {
            Ok(_res) => {}
            Err(_e) => {}
        }
        Ok(())
    }
    /// Get raw stream
    pub fn get_stream(&mut self) -> &TcpStream {
        return &self.stream;
    }
}
