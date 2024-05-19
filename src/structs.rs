use std::{collections::HashMap, io::Write, net::TcpStream, string};

#[derive(Clone, PartialEq)]

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

        let mut path: &str = parts[1];
        let mut query: HashMap<String, String> = HashMap::new();
        if path.contains("?") == true && path.contains("=") == true {
            let parts: Vec<&str> = path.split("?").collect();
            path = parts[0];
            if parts[1].len() > 2 {
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
}

pub struct Response {
    stream: TcpStream,
}
impl Response {
    pub fn new(stream: TcpStream) -> Response {
        return Response { stream: stream };
    }
    ///Sends string as output.
    pub fn send_string(&mut self, data: &str) {
        let response = "HTTP/1.1 200 OK\r\n\r\n".to_owned() + data;

        self.stream
            .write_all(response.as_bytes())
            .expect("Failed to write");
    }
    pub fn send_bytes(&mut self, data: &[u8]) {
        let response = "HTTP/1.1 200 OK\r\n\r\n";

        self.stream
            .write_all(response.as_bytes())
            .expect("Failed to write");
        self.stream.write_all(data).expect("Failed to write");
    }
    pub fn send_code(&mut self, code: usize) {
        let response = "HTTP/1.1 ".to_owned()
            + &code.to_string()
            + match code {
                404 => " NOT FOUND\r\n\r\n",
                413 => " PAYLOAD TOO LARGE\r\n\r\n",
                _ => " OK\r\n\r\n",
            };

        self.stream
            .write_all(response.as_bytes())
            .expect("Failed to write");
    }
}
pub struct Request {
    pub query: HashMap<String, String>,
    pub user_agent: Option<String>,
    pub content_length: usize,
}
impl Request {
    pub fn new(
        query: HashMap<String, String>,
        user_agent: Option<String>,
        content_length: usize,
    ) -> Request {
        return Request {
            query: query,
            user_agent: user_agent,
            content_length: content_length,
        };
    }
    pub fn parse(lines: Vec<&str>, query: Option<HashMap<String, String>>) -> Request {
        let mut req = Request::new(query.unwrap_or_default(), None, 0);
        for line in lines {
            if line.starts_with("User-Agent:") {
                let parts: Vec<&str> = line.split("Agent: ").collect();

                req.user_agent = Some(parts[1].to_string());
            } else if line.starts_with("Content-Length:") {
                let parts: Vec<&str> = line.split(" ").collect();

                req.content_length = match parts[1].parse::<usize>() {
                    Ok(res) => res,
                    Err(err) => 0,
                };
            }
        }
        return req;
    }
}
#[derive(Clone)]
pub struct EndPoint {
    pub path: String,
    pub req_type: RequestType,
    pub handle: fn(req: Request, res: Response),
}
impl EndPoint {
    pub fn new(
        path: String,
        req_type: RequestType,
        handle: fn(req: Request, res: Response),
    ) -> EndPoint {
        return EndPoint {
            path: path,
            req_type: req_type,
            handle: handle,
        };
    }
}
