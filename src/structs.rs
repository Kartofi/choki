use std::{collections::HashMap, io::Write, net::TcpStream, string};

#[derive(Clone, PartialEq)]
pub enum RequestType {
    Get = 1,
    Post = 2,
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
}
impl Response {
    pub fn new(stream: TcpStream) -> Response {
        return Response {
            stream: stream,
            cookies: Vec::new(),
            headers: Vec::new(),
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
    ///Sends string as output.
    pub fn send_string(&mut self, data: &str) {
        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);
        let headers_set_headers = Header::generate_headers(&self.headers);
        let response = "HTTP/1.1 200 OK".to_string()
            + &headers_set_headers
            + &cookies_set_headers
            + "Content-type: "
            + ContentType::PlainText.as_str()
            + "\r\n\r\n"
            + data;

        match self.stream.write_all(response.as_bytes()) {
            Ok(_res) => {}
            Err(_e) => {}
        }
    }
    ///Sends json as output.
    pub fn send_json(&mut self, data: &str) {
        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);
        let headers_set_headers = Header::generate_headers(&self.headers);
        let response = "HTTP/1.1 200 OK".to_string()
            + &headers_set_headers
            + &cookies_set_headers
            + "Content-type: "
            + ContentType::Json.as_str()
            + "\r\n\r\n"
            + data;

        match self.stream.write_all(response.as_bytes()) {
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
        let content_type_string = format!("\nContent-type: {}", content_type.as_str());
        let content_length_string = format!("\nContent-length: {}\r\n\r\n", data.len());

        let cookies_set_headers = Cookie::generate_set_cookie_headers(&self.cookies);
        let headers_set_headers = Header::generate_headers(&self.headers);

        let response = "HTTP/1.1 200 OK".to_owned()
            + &headers_set_headers
            + (if content_type == ContentType::None {
                ""
            } else {
                &content_type_string
            })
            + &content_length_string
            + &cookies_set_headers;

        match self.stream.write_all(response.as_bytes()) {
            Ok(_res) => {}
            Err(_e) => {}
        }

        match self.stream.write_all(data) {
            Ok(_res) => {}
            Err(_e) => {}
        }
    }

    //Sends a response code (404, 200...)
    pub fn send_code(&mut self, code: usize) {
        let mut response = "HTTP/1.1 ".to_owned()
            + &code.to_string()
            + (match code {
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
}

pub struct Request {
    pub query: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub cookies: Vec<Cookie>,
    pub user_agent: Option<String>,
    pub content_length: usize,
}
impl Request {
    pub fn new(
        query: HashMap<String, String>,
        params: HashMap<String, String>,
        user_agent: Option<String>,
        cookies: Vec<Cookie>,
        content_length: usize,
    ) -> Request {
        return Request {
            query: query,
            params: params,
            user_agent: user_agent,
            cookies: cookies,
            content_length: content_length,
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
            None,
            Vec::new(),
            0,
        );
        for line in lines {
            if line.starts_with("User-Agent:") {
                let parts: Vec<&str> = line.split("Agent: ").collect();

                req.user_agent = Some(parts[1].to_string());
            } else if line.starts_with("Content-Length:") {
                let parts: Vec<&str> = line.split(" ").collect();
                if parts.len() > 0 {
                    req.content_length = match parts[1].parse::<usize>() {
                        Ok(res) => res,
                        Err(err) => 0,
                    };
                }
            } else if line.starts_with("Cookie:") {
                let parts: Vec<&str> = line.split("Cookie: ").collect();
                if parts.len() == 2 {
                    let cookies: Vec<&str> = parts[1].split("; ").collect();
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
