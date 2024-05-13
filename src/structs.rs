use std::{collections::HashMap, hash::Hash};

#[derive(Clone)]
pub enum RequestType {
    Get = 1,
    Post = 2,
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

        let path = parts[1];
        let query: HashMap<String, String> = HashMap::new();

        Ok(Url::new(path.to_string(), req_type, query))
    }
}
#[derive(Clone)]
pub struct EndPoint {
    pub path: String,
    pub req_type: RequestType,
}
impl EndPoint {
    pub fn new(path: String, req_type: RequestType) -> EndPoint {
        return EndPoint {
            path: path,
            req_type: req_type,
        };
    }
}
