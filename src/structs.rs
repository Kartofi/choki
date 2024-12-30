use std::{
    collections::HashMap,
    io::{BufReader, Read, Write},
    net::TcpStream,
    string,
    time::Instant,
    vec,
};

use flate2::{write::GzEncoder, Compression};

#[derive(Clone, PartialEq)]
pub enum RequestType {
    Get = 1,
    Post = 2,
}
#[derive(Clone, PartialEq)]
pub enum EncodingType {
    Unknown = 1,
    Any = 2,
    Gzip = 3,
}
impl EncodingType {
    pub fn from_string(input: &str) -> EncodingType {
        match input.to_lowercase().as_str() {
            "gzip" => EncodingType::Gzip,
            "*" => EncodingType::Any,
            _ => EncodingType::Unknown,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            EncodingType::Gzip => "gzip".to_owned(),
            EncodingType::Any => "*".to_owned(),
            EncodingType::Unknown => "".to_owned(),
        }
    }
}
#[derive(Clone, PartialEq)]
pub struct Encoding {
    pub encoding_type: EncodingType,
    pub quality: f32,
}
impl Encoding {
    pub fn new(encoding_type: EncodingType, quality: f32) -> Encoding {
        return Encoding {
            encoding_type: encoding_type,
            quality: quality,
        };
    }
}
#[derive(Clone, PartialEq, Debug)]
pub enum ContentType {
    //None
    None,

    // Text Types
    PlainText,
    Html,
    Css,
    Javascript,
    Xml,

    // Application Types
    Json,
    XmlApp,
    Pdf,
    JavascriptApp,
    Zip,
    // Image Types
    Jpeg,
    Png,
    Gif,
    Webp,
    SvgXml,

    OctetStream,
}

impl ContentType {
    pub fn as_str(&self) -> &str {
        match self {
            ContentType::None => "",
            // Text Types
            ContentType::PlainText => "text/plain",
            ContentType::Html => "text/html",
            ContentType::Css => "text/css",
            ContentType::Javascript => "text/javascript",
            ContentType::Xml => "text/xml",

            // Application Types
            ContentType::Json => "application/json",
            ContentType::XmlApp => "application/xml",
            ContentType::Pdf => "application/pdf",
            ContentType::JavascriptApp => "application/javascript",
            ContentType::Zip => "application/zip",
            ContentType::OctetStream => "application/octet-stream",
            // Image Types
            ContentType::Jpeg => "image/jpeg",
            ContentType::Png => "image/png",
            ContentType::Gif => "image/gif",
            ContentType::Webp => "image/webp",
            ContentType::SvgXml => "image/svg+xml",
        }
    }

    pub fn from_string(input: &str) -> ContentType {
        match input.to_lowercase().as_str() {
            // Text Types
            "text/plain" => ContentType::PlainText,
            "text/html" => ContentType::Html,
            "text/css" => ContentType::Css,
            "text/javascript" => ContentType::Javascript,
            "text/xml" => ContentType::Xml,

            // Application Types
            "application/json" => ContentType::Json,
            "application/xml" => ContentType::XmlApp,
            "application/pdf" => ContentType::Pdf,
            "application/javascript" => ContentType::JavascriptApp,
            "application/zip" => ContentType::Zip,
            "application/octet-stream" => ContentType::OctetStream,
            // Image Types
            "image/jpeg" => ContentType::Jpeg,
            "image/png" => ContentType::Png,
            "image/gif" => ContentType::Gif,
            "image/webp" => ContentType::Webp,
            "image/svg+xml" => ContentType::SvgXml,

            _ => ContentType::None,
        }
    }
}

#[derive(Debug)]
pub struct HttpServerError {
    pub reason: String,
}
impl HttpServerError {
    pub fn new(reason: String) -> HttpServerError {
        return HttpServerError { reason: reason };
    }
}
pub struct Url {
    pub path: String,
    pub req_type: RequestType,
    pub query: HashMap<String, String>,
}
impl Url {
    pub fn new(path: String, req_type: RequestType, query: HashMap<String, String>) -> Url {
        return Url {
            path: path,
            req_type: req_type,
            query: query,
        };
    }
    pub fn parse(input: &str) -> Result<Url, HttpServerError> {
        let parts: Vec<&str> = input.split(" ").collect();
        if parts.len() != 3 {
            return Err(HttpServerError::new("Invalid input.".to_string()));
        }
        let req_type: RequestType = if parts[0].to_lowercase() == "get" {
            RequestType::Get
        } else {
            RequestType::Post
        };

        let mut path: &str = parts[1];
        let mut query: HashMap<String, String> = HashMap::new();

        if path.contains("?") == true {
            let parts: Vec<&str> = path.split("?").collect();
            path = parts[0];

            if parts[1].len() > 2 && parts[1].contains("=") == true {
                if parts[1].contains("&") {
                    let queries_string: Vec<&str> = parts[1].split("&").collect();
                    for query_string in queries_string {
                        let query_string: Vec<&str> = query_string.split("=").collect();
                        if query_string.len() == 2 {
                            query.insert(query_string[0].to_string(), query_string[1].to_string());
                        }
                    }
                } else {
                    let query_string: Vec<&str> = parts[1].split("=").collect();

                    if query_string.len() == 2 {
                        query.insert(query_string[0].to_string(), query_string[1].to_string());
                    }
                }
            }
        }

        Ok(Url::new(path.to_string(), req_type, query))
    }
    pub fn match_patern(input: &str, pattern: &str) -> (bool, HashMap<String, String>) {
        let mut parts_input: Vec<&str> = input.split('/').filter(|a| a.len() > 0).collect();
        let mut parts_pattern: Vec<&str> = pattern.split('/').filter(|a| a.len() > 0).collect();

        if parts_input.len() != parts_pattern.len() {
            return (false, HashMap::new());
        }
        if parts_input.len() == 0 && parts_pattern.len() == 0 {
            return (true, HashMap::new());
        }
        let mut params: HashMap<String, String> = HashMap::new();
        for i in 0..parts_input.len() {
            if parts_input[i] != parts_pattern[i] && parts_pattern[i].contains("[") == false {
                return (false, HashMap::new());
            }
            if parts_input[i] != parts_pattern[i] {
                let name: String = parts_pattern[i]
                    .chars()
                    .filter(|&c| c != '[' && c != ']')
                    .collect();
                params.insert(name, parts_input[i].to_string());
            }
        }
        return (true, params);
    }
}
#[derive(Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: String,
    pub expires: String,
}
impl Cookie {
    pub fn new_simple(name: String, value: String) -> Cookie {
        return Cookie {
            name: name,
            value: value,
            path: "".to_string(),
            expires: "".to_string(),
        };
    }

    pub fn as_str(&self) -> String {
        let mut cookie_str = format!("{}={}", self.name, self.value);

        if !self.path.is_empty() {
            cookie_str.push_str(&format!("; Path={}", self.path));
        }

        if !self.expires.is_empty() {
            cookie_str.push_str(&format!("; Expires={}", self.expires));
        }

        cookie_str
    }
    pub fn generate_set_cookie_headers(cookies: &Vec<Cookie>) -> String {
        cookies
            .iter()
            .map(|cookie| format!("\nSet-Cookie: {}", cookie.as_str()))
            .collect()
    }
}

#[derive(Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}
impl Header {
    pub fn new(name: String, value: String) -> Header {
        return Header {
            name: name,
            value: value,
        };
    }

    pub fn as_str(&self) -> String {
        let mut cookie_str = format!("{}: {}", self.name, self.value);

        cookie_str
    }
    pub fn generate_headers(headers: &Vec<Header>) -> String {
        let mut headers_str = "\n".to_string();
        if headers.len() == 0 {
            return "".to_string();
        }
        let last_header = headers.last().unwrap().name.clone();
        for header in headers {
            headers_str.push_str(&header.as_str());
            if last_header != header.name {
                headers_str.push_str("\n");
            }
        }
        headers_str
    }
}

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
    pub fn send_string(&mut self, data: &str) {
        self.use_encoding = true;
        self.send_bytes(data.as_bytes(), Some(ContentType::PlainText));
    }
    ///Sends json as output.
    pub fn send_json(&mut self, data: &str) {
        self.use_encoding = true;
        self.send_bytes(data.as_bytes(), Some(ContentType::Json));
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

        // Write the data in chunks
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

        let headers_set_headers = Header::generate_headers(&self.headers); // Assuming generate_headers exists
        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);

        let mut response =
            "HTTP/1.1 200 OK".to_owned() + &headers_set_headers + &cookies_set_headers;
        response = response.trim().to_owned();
        response += "\r\n\r\n";

        if let Err(e) = self.stream.write_all(response.as_bytes()) {
            eprintln!("Failed to write response headers: {}", e);
            return;
        }

        const CHUNK_SIZE: usize = 8192; // 8 KB chunk size for efficient streaming
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
                100 => "Continue",
                404 => " NOT FOUND\r\n\r\nPAGE NOT FOUND",
                413 => " PAYLOAD TOO LARGE\r\n\r\nPAYLOAD TOO LARGE",
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
}
#[derive(Clone)]
pub struct EndPoint<T: Clone + std::marker::Send + 'static> {
    pub path: String,
    pub req_type: RequestType,
    pub handle: fn(req: Request, res: Response, public_var: Option<T>),
}

impl<T: Clone + std::marker::Send + 'static> EndPoint<T> {
    pub fn new(
        path: String,
        req_type: RequestType,
        handle: fn(req: Request, res: Response, public_var: Option<T>),
    ) -> EndPoint<T> {
        if count_char_occurrences(&path, '[') != count_char_occurrences(&path, ']') {
            panic!("Syntax error in pattern: {}", path);
        }
        return EndPoint {
            path: path,
            req_type: req_type,
            handle: handle,
        };
    }
}

fn count_char_occurrences(s: &str, target: char) -> usize {
    s.chars().filter(|&c| c == target).count()
}
fn map_compression_level(compression_float: f32) -> u32 {
    if compression_float <= 0.0 {
        0
    } else if compression_float >= 1.0 {
        10
    } else {
        (compression_float * 10.0).round() as u32
    }
}
