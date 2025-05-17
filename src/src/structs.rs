use std::{ collections::HashMap, io::BufRead };

use urlencoding::decode;

use super::{
    request::Request,
    response::Response,
    utils::utils::{ contains_blank, count_char_occurrences },
};

#[derive(Clone, PartialEq)]
pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
    Other(String),
}
impl RequestType {
    pub fn from_string(input: &str) -> Result<RequestType, HttpServerError> {
        if contains_blank(input) {
            return Err(HttpServerError::new("Content Type can't contain blank spaces."));
        }

        let res = match input.to_lowercase().as_str() {
            "get" => RequestType::Get,
            "post" => RequestType::Post,
            "put" => RequestType::Put,
            "delete" => RequestType::Delete,
            "head" => RequestType::Head,
            "options" => RequestType::Options,
            "patch" => RequestType::Patch,

            input => RequestType::Other(input.to_string()),
        };
        Ok(res)
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
    Other(String),
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

            ContentType::Other(content_type) => content_type,
        }
    }

    pub fn from_string(input: &str) -> Result<ContentType, HttpServerError> {
        if contains_blank(input) {
            return Err(HttpServerError::new("Content Type can't contain blank spaces."));
        }

        let res = match input.to_lowercase().as_str() {
            // Text Types
            "text/plain" => ContentType::PlainText,
            "text/html" => ContentType::Html,
            "text/css" => ContentType::Css,
            "text/javascript" => ContentType::Javascript,
            "text/xml" => ContentType::Xml,

            // Application Types
            "application/json" => ContentType::Json,
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

            input => ContentType::Other(input.to_string()),
        };
        Ok(res)
    }
    pub fn from_extension(extension: &str) -> Option<ContentType> {
        match extension.to_lowercase().as_str() {
            // Text Types
            "txt" => Some(ContentType::PlainText),
            "html" => Some(ContentType::Html),
            "css" => Some(ContentType::Css),
            "js" => Some(ContentType::Javascript),
            "ts" => Some(ContentType::Javascript),
            "xml" => Some(ContentType::Xml),
            "xaml" => Some(ContentType::Xml),

            // Application Types
            "json" => Some(ContentType::Json),

            // Image Types
            "jpeg" => Some(ContentType::Jpeg),
            "png" => Some(ContentType::Png),
            "gif" => Some(ContentType::Gif),
            "webp" => Some(ContentType::Webp),
            "svg" => Some(ContentType::SvgXml),
            // Video Types
            "mkv" => Some(ContentType::Mkv),
            "mp4" => Some(ContentType::Mp4),

            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct HttpServerError {
    pub reason: String,
}
impl HttpServerError {
    pub fn new(reason: &str) -> HttpServerError {
        return HttpServerError { reason: reason.to_string() };
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
            return Err(HttpServerError::new("Invalid input."));
        }
        let req_type: RequestType = RequestType::from_string(&parts[0].to_lowercase())?;

        let mut path: &str = &decode(parts[1]).unwrap_or_default().to_string();
        //Clean url
        let clean_path = path.replace("..", "").replace("//", "/");
        path = &clean_path;
        //
        let mut query: HashMap<String, String> = HashMap::new();

        if path.contains("?") == true {
            let parts: (&str, &str) = path.split_once("?").unwrap();

            path = parts.0;

            if parts.1.len() > 2 && parts.1.contains("=") == true {
                if parts.1.contains("&") {
                    let queries_string: Vec<&str> = parts.1.split("&").collect();
                    for query_string in queries_string {
                        let query_string: Option<(&str, &str)> = query_string.split_once("=");
                        if query_string.is_some() {
                            let query_string = query_string.unwrap();
                            query.insert(query_string.0.to_string(), query_string.1.to_string());
                        }
                    }
                } else {
                    let query_string: Option<(&str, &str)> = parts.1.split_once("=");

                    if query_string.is_some() {
                        let query_string = query_string.unwrap();
                        query.insert(query_string.0.to_string(), query_string.1.to_string());
                    }
                }
            }
        }

        Ok(Url::new(path.to_owned(), req_type, query))
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
    /// Creates a simple cookie with name and value
    pub fn new_simple(name: String, value: String) -> Cookie {
        return Cookie {
            name: name,
            value: value,
            path: "".to_string(),
            expires: "".to_string(),
        };
    }
    /// Converts cookie into str for writing in response
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
    /// generates set-cookie headers
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
    /// Creates a header with a name and value
    pub fn new(name: &str, value: &str) -> Header {
        return Header {
            name: name.to_owned(),
            value: value.to_owned(),
        };
    }
    /// Converts the header int ostring "{name}: {value}"
    pub fn as_str(&self) -> String {
        format!("{}: {}", self.name, self.value)
    }
    /// Generates headers into string from a list/vec
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
    pub handle: fn(
        req: Request,
        res: Response,
        public_var: Option<T>
    ) -> Result<(), HttpServerError>,
}

impl<T: Clone + std::marker::Send + 'static> EndPoint<T> {
    pub fn new(
        path: String,
        req_type: RequestType,
        handle: fn(
            req: Request,
            res: Response,
            public_var: Option<T>
        ) -> Result<(), HttpServerError>
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
/// Info for bodyitem: name,content type, file name if its a file, and value if urlencoded
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
    pub fn from_str(input: &str) -> Result<BodyItemInfo, HttpServerError> {
        let lines: Vec<&str> = input.lines().collect();

        let mut body_item = BodyItemInfo::default();

        if lines.len() == 0 {
            return Ok(body_item);
        }

        let mut parts: Vec<&str> = Vec::new();

        if lines.len() > 1 {
            parts = lines[1].split(": ").collect();
            if parts.len() > 1 {
                body_item.content_type = ContentType::from_string(parts[1])?;
            } else {
                return Ok(BodyItemInfo::default());
            }
        } else {
            body_item.content_type = ContentType::MultipartForm;
        }

        parts = lines[0].split("; ").collect();

        if parts.len() < 2 || (parts.len() != 3 && lines.len() == 2) {
            return Ok(BodyItemInfo::default());
        }
        if lines.len() == 2 {
            let file_name = parts[2].to_string();
            if file_name.len() > 11 {
                body_item.file_name = Some(
                    parts[2].to_string()[10..file_name.len() - 1].to_string()
                );
            }
        }
        let name = parts[1].replace("name=\"", "");
        if name.len() < 1 {
            body_item.name = None;
        } else {
            body_item.name = Some(name[0..name.len() - 1].to_string());
        }

        return Ok(body_item);
    }
}
/// Holds the data and a pointer to the info
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseCode {
    Continue,
    Ok,
    PartialContent,
    BadRequest,
    NotFound,
    MethodNotAllowed,
    ContentTooLarge,
    RangeNotSatisfiable,

    Other(i64),
}

impl ResponseCode {
    pub fn as_u16(&self) -> u16 {
        match self {
            ResponseCode::Continue => 100,
            ResponseCode::Ok => 200,
            ResponseCode::PartialContent => 206,
            ResponseCode::BadRequest => 400,
            ResponseCode::NotFound => 404,
            ResponseCode::MethodNotAllowed => 405,
            ResponseCode::ContentTooLarge => 413,
            ResponseCode::RangeNotSatisfiable => 416,
            ResponseCode::Other(code) => *code as u16,
        }
    }

    pub fn from_i64(code: i64) -> ResponseCode {
        match code {
            100 => ResponseCode::Continue,
            200 => ResponseCode::Ok,
            206 => ResponseCode::PartialContent,
            400 => ResponseCode::BadRequest,
            404 => ResponseCode::NotFound,
            405 => ResponseCode::MethodNotAllowed,
            413 => ResponseCode::ContentTooLarge,
            416 => ResponseCode::RangeNotSatisfiable,

            _ => ResponseCode::Other(code),
        }
    }

    pub fn to_desc(&self) -> String {
        match self.as_u16() {
            100 => "Continue".to_owned(),
            200 => "OK".to_owned(),
            206 => "Partial Content".to_owned(),
            400 => "Bad Request".to_owned(),
            404 => "NOT FOUND".to_owned(),
            405 => "Method Not Allowed".to_owned(),
            413 => "Content Too Large".to_owned(),
            416 => "Range Not Satisfiable".to_owned(),

            _ => "Unknown".to_owned(),
        }
    }

    pub fn to_string(&self) -> String {
        self.as_u16().to_string()
    }

    pub fn format_string(&self) -> String {
        format!("{} {}", &self.to_string(), &self.to_desc())
    }
}
