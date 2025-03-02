use std::{ collections::HashMap, io::BufRead };

use super::{ request::Request, response::Response, utils::count_char_occurrences };

#[derive(Clone, PartialEq)]
pub enum RequestType {
    Unknown = 0,
    Get = 1,
    Post = 2,
    Put = 3,
    Delete = 4,
}
impl RequestType {
    pub fn from_string(input: &str) -> RequestType {
        match input.to_lowercase().as_str() {
            "get" => RequestType::Get,
            "post" => RequestType::Post,
            "put" => RequestType::Put,
            "delete" => RequestType::Delete,
            _ => RequestType::Unknown,
        }
    }
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
    MultipartForm,
    UrlEncoded,
    // Image Types
    Jpeg,
    Png,
    Gif,
    Webp,
    SvgXml,
    // Videos
    Mkv,
    Mp4,

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
            ContentType::MultipartForm => "multipart/form-data",
            ContentType::UrlEncoded => "application/x-www-form-urlencoded",
            // Image Types
            ContentType::Jpeg => "image/jpeg",
            ContentType::Png => "image/png",
            ContentType::Gif => "image/gif",
            ContentType::Webp => "image/webp",
            ContentType::SvgXml => "image/svg+xml",
            //Video Types
            ContentType::Mkv => "video/mkv",
            ContentType::Mp4 => "video/mp4",
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
            "multipart/form-data" => ContentType::MultipartForm,
            "application/x-www-form-urlencoded" => ContentType::UrlEncoded,
            // Image Types
            "image/jpeg" => ContentType::Jpeg,
            "image/png" => ContentType::Png,
            "image/gif" => ContentType::Gif,
            "image/webp" => ContentType::Webp,
            "image/svg+xml" => ContentType::SvgXml,
            // Video Types
            "video/mkv" => ContentType::Mkv,
            "video/mp4" => ContentType::Mp4,
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
        let req_type: RequestType = RequestType::from_string(&parts[0].to_lowercase());

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
        let parts_input: Vec<&str> = input
            .split('/')
            .filter(|a| a.len() > 0)
            .collect();
        let parts_pattern: Vec<&str> = pattern
            .split('/')
            .filter(|a| a.len() > 0)
            .collect();

        if parts_input.len() != parts_pattern.len() {
            return (false, HashMap::new());
        }
        if parts_input.len() == 0 && parts_pattern.len() == 0 {
            return (true, HashMap::new());
        }
        let mut params: HashMap<String, String> = HashMap::new();
        for i in 0..parts_input.len() {
            if
                parts_input[i] != parts_pattern[i] &&
                parts_pattern[i].contains("[") == false &&
                parts_pattern[i].contains("]") == false
            {
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
            .map(|cookie| format!("\r\nSet-Cookie: {}", cookie.as_str()))
            .collect()
    }
}

#[derive(Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}
impl Header {
    pub fn new(name: &str, value: &str) -> Header {
        return Header {
            name: name.to_owned(),
            value: value.to_owned(),
        };
    }

    pub fn as_str(&self) -> String {
        let cookie_str = format!("{}: {}", self.name, self.value);

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
                headers_str.push_str("\r\n");
            }
        }
        headers_str
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
        handle: fn(req: Request, res: Response, public_var: Option<T>)
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
#[derive(Clone)]
pub struct BodyItemInfo {
    pub content_type: ContentType,
    pub name: Option<String>,

    pub file_name: Option<String>, // Special only for files

    value: Option<String>, // Only for urlencoded
}

impl BodyItemInfo {
    pub fn default() -> BodyItemInfo {
        return BodyItemInfo {
            content_type: ContentType::None,
            name: None,
            file_name: None,
            value: None,
        };
    }
    pub fn new_simple(content_type: ContentType) -> BodyItemInfo {
        return BodyItemInfo {
            content_type: content_type,
            name: None,
            file_name: None,
            value: None,
        };
    }
    pub fn new_url(name: String, value: String) -> BodyItemInfo {
        return BodyItemInfo {
            content_type: ContentType::UrlEncoded,
            name: Some(name),
            file_name: None,
            value: Some(value),
        };
    }
    pub fn to_body_item(&self) -> BodyItem {
        return BodyItem::new_url(self, self.value.clone().unwrap_or_default());
    }
    pub fn from_str(input: &str) -> BodyItemInfo {
        let lines: Vec<&str> = input.lines().collect();

        let mut body_item = BodyItemInfo::default();

        if lines.len() == 0 {
            return body_item;
        }

        let mut parts: Vec<&str> = Vec::new();

        if lines.len() > 1 {
            parts = lines[1].split(": ").collect();
            if parts.len() > 1 {
                body_item.content_type = ContentType::from_string(parts[1]);
            } else {
                return BodyItemInfo::default();
            }
        } else {
            body_item.content_type = ContentType::MultipartForm;
        }

        parts = lines[0].split("; ").collect();

        if parts.len() < 2 || (parts.len() != 3 && lines.len() == 2) {
            return BodyItemInfo::default();
        }
        if lines.len() == 2 {
            body_item.file_name = Some(parts[2].to_string());
        }
        let name = parts[1].replace("name=\"", "");
        if name.len() < 1 {
            body_item.name = None;
        } else {
            body_item.name = Some(name[0..name.len() - 1].to_string());
        }

        return body_item;
    }
}

pub struct BodyItem<'a> {
    pub info: &'a BodyItemInfo,
    pub data: &'a [u8],
    pub value: String,
}
impl<'a> BodyItem<'a> {
    pub fn new(info: &'a BodyItemInfo, data: &'a [u8]) -> BodyItem<'a> {
        return BodyItem {
            info: info,
            data: data,
            value: "".to_owned(),
        };
    }
    pub fn new_url(info: &'a BodyItemInfo, value: String) -> BodyItem<'a> {
        return BodyItem {
            info: info,
            data: &[],
            value: value,
        };
    }
}
#[derive(Clone, Copy)]
pub enum ResponseCode {
    Continue = 100,
    Ok = 200,
    PartialContent = 206,
    BadRequest = 400,
    NotFound = 404,
    MethodNotAllowed = 405,
    ContentTooLarge = 413,
}
impl ResponseCode {
    pub fn to_desc(&self) -> String {
        match *self as i32 {
            100 => "Continue".to_owned(),
            200 => "OK".to_owned(),
            206 => "Partial Content".to_owned(),
            400 => "Bad Request".to_owned(),
            404 => "NOT FOUND".to_owned(),
            405 => "Method Not Allowed".to_owned(),
            413 => "Content Too Large".to_owned(),

            _ => "Unknown".to_owned(),
        }
    }
    pub fn to_string(&self) -> String {
        (*self as i32).to_string()
    }
}
