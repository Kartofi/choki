use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use std::{fs, thread};
use std::{io::Write, net::*};

use std::io::{Error, Read};
use structs::*;
use threadpool::ThreadPool;

pub mod structs;
pub struct Server {
    active: bool,
    pub max_content_length: usize,
    pub endpoints: Vec<EndPoint>,
    pub static_endpoints: HashMap<String, String>,
}

impl Server {
    pub fn new(max_content_length: Option<usize>) -> Server {
        return Server {
            active: false,
            max_content_length: max_content_length.unwrap_or_default(),
            endpoints: Vec::new(),
            static_endpoints: HashMap::new(),
        };
    }
    ///Creates a new static url
    /// For example a folder named "images" on path /images every image in that folder will be exposed like "/images/example.png"
    pub fn new_static(&mut self, mut path: String, folder: String) -> Result<(), HttpServerError> {
        if self.active == true {
            return Err(HttpServerError::new(
                "Server is already running!".to_string(),
            ));
        }
        let path_: &Path = Path::new(&folder);

        if path_.is_dir() == false || path_.exists() == false {
            return Err(HttpServerError::new(
                "Folder does not exist or the path provided is a file!".to_string(),
            ));
        }

        if self.endpoints.len() > 0
            && self
                .endpoints
                .iter()
                .any(|x| x.path == path && x.req_type == RequestType::Get)
            || self.static_endpoints.len() > 0 && self.static_endpoints.iter().any(|x| x.0 == &path)
        {
            return Err(HttpServerError::new("Endpoint already exists!".to_string()));
        }
        if path.len() > 1 && path.ends_with("/") {
            path.remove(path.len() - 1);
        }
        self.static_endpoints.insert(path, folder);
        Ok(())
    }
    fn new_endpoint(
        &mut self,
        mut path: String,
        req_type: RequestType,
        handle: fn(req: Request, res: Response),
    ) -> Result<(), HttpServerError> {
        if self.active == true {
            return Err(HttpServerError::new(
                "Server is already running!".to_string(),
            ));
        }
        if self.endpoints.len() > 0
            && self
                .endpoints
                .iter()
                .any(|x| x.path == path && x.req_type == req_type)
            || self.static_endpoints.len() > 0 && self.static_endpoints.iter().any(|x| x.0 == &path)
        {
            return Err(HttpServerError::new("Endpoint already exists!".to_string()));
        }
        if path.len() > 1 && path.ends_with("/") {
            path.remove(path.len() - 1);
        }
        self.endpoints.push(EndPoint::new(path, req_type, handle));
        Ok(())
    }

    ///Creates a new GET endpoint
    pub fn get(
        &mut self,
        mut path: String,
        handle: fn(req: Request, res: Response),
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, RequestType::Get, handle)
    }

    ///Creates a new POST endpoint
    pub fn post(
        &mut self,
        mut path: String,
        handle: fn(req: Request, res: Response),
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, RequestType::Post, handle)
    }
    ///Starts listening on the given port.
    pub fn listen(&mut self, port: u32) -> Result<(), HttpServerError> {
        if port > 65_535 {
            return Err(HttpServerError::new(
                "Invalid port: port must be 0-65,535".to_string(),
            ));
        }
        if self.active == true {
            return Err(HttpServerError::new(
                "The server is already running!".to_string(),
            ));
        }
        self.active = true;
        let pool: ThreadPool = ThreadPool::new(6);
        let routes = self.endpoints.clone();
        let static_routes = self.static_endpoints.clone();

        let max_content_length = self.max_content_length.clone();

        thread::spawn(move || {
            let tcp: TcpListener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
            for stream in tcp.incoming() {
                let routes_clone = routes.clone();
                let static_routes_clone = static_routes.clone();
                let max_content_length_clone = max_content_length.clone();

                pool.execute(move || {
                    let mut stream: TcpStream = stream.unwrap();

                    let mut buffer = [0; 1024];

                    stream.read(&mut buffer).expect("Failed to read");

                    let string_req = String::from_utf8_lossy(&buffer);

                    let lines: Vec<&str> = string_req.lines().collect();
                    if lines.len() == 1 {
                        return;
                    }
                    let req_url = Url::parse(lines[0]).unwrap();

                    let req = Request::parse(lines, Some(req_url.query));

                    if max_content_length_clone > 0 && req.content_length > max_content_length_clone
                    {
                        let mut res = Response::new(stream.try_clone().unwrap());
                        res.send_code(413);
                    } else {
                        let mut sent: bool = false;
                        for route in routes_clone {
                            println!("{}", route.path);
                            if route.path == req_url.path && req_url.req_type == route.req_type {
                                let mut res = Response::new(stream.try_clone().unwrap());
                                (route.handle)(req, res);
                                sent = true;
                                break;
                            }
                        }
                        if sent == false {
                            let mut res2 = Response::new(stream.try_clone().unwrap());

                            for route in static_routes_clone {
                                if req_url.path.starts_with(&route.0) {
                                    let parts: Vec<&str> = req_url.path.split(&route.0).collect();
                                    if parts.len() == 0 {
                                        continue;
                                    }
                                    match fs::read(route.1 + parts[1]) {
                                        Ok(data) => {
                                            res2.send_bytes(&data);
                                        }
                                        Err(err) => {
                                            println!("There was error serving this file!");
                                            res2.send_code(404);
                                        }
                                    }

                                    break;
                                }
                            }
                        }
                    }
                    stream.flush().expect("Failed to flush");
                });
            }
        });
        Ok(())
    }
}
