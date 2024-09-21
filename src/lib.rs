use std::collections::HashMap;
use std::fmt::write;
use std::path::Path;
use std::time::{Duration, Instant};
use std::{fs, thread};
use std::{io::Write, net::*};

use std::io::{Error, Read};
use structs::*;
use threadpool::ThreadPool;
extern crate num_cpus;

pub mod structs;
pub struct Server<T: Clone + std::marker::Send + 'static> {
    active: bool,
    pub max_content_length: usize,
    pub endpoints: Vec<EndPoint<T>>,
    pub static_endpoints: HashMap<String, String>,

    pub public_var: Option<T>,
}

impl<T: Clone + std::marker::Send + 'static> Server<T> {
    ///max_content_length is the max length of the request in bytes.
    ///
    ///For example if the max is set to 1024 but the request is 1 000 000 it will close it straight away.
    pub fn new(max_content_length: Option<usize>, public_var: Option<T>) -> Server<T> {
        return Server {
            active: false,
            max_content_length: max_content_length.unwrap_or_default(),
            endpoints: Vec::new(),
            static_endpoints: HashMap::new(),
            public_var: public_var,
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
        handle: fn(req: Request, res: Response, public_var: Option<T>),
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
        handle: fn(req: Request, res: Response, public_var: Option<T>),
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, RequestType::Get, handle)
    }

    ///Creates a new POST endpoint
    pub fn post(
        &mut self,
        mut path: String,
        handle: fn(req: Request, res: Response, public_var: Option<T>),
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, RequestType::Post, handle)
    }
    ///Starts listening on the given port.
    pub fn listen(&mut self, port: u32, address: Option<String>) -> Result<(), HttpServerError> {
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
        let pool: ThreadPool = ThreadPool::new(num_cpus::get());
        let routes = self.endpoints.clone();
        let static_routes = self.static_endpoints.clone();

        let max_content_length = self.max_content_length.clone();
        let public_var = self.public_var.clone();

        thread::spawn(move || {
            let tcp: TcpListener = TcpListener::bind(format!(
                "{}:{}",
                address.unwrap_or("0.0.0.0".to_string()),
                port
            ))
            .unwrap();
            for stream in tcp.incoming() {
                let routes_clone = routes.clone();
                let static_routes_clone = static_routes.clone();
                let max_content_length_clone = max_content_length.clone();
                let public_var_clone = public_var.clone();

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

                    let mut req = Request::parse(lines, Some(req_url.query), None);

                    if max_content_length_clone > 0 && req.content_length > max_content_length_clone
                    {
                        let mut res = Response::new(stream.try_clone().unwrap());
                        res.send_code(413);
                    } else {
                        let mut sent: bool = false;
                        for route in routes_clone {
                            let match_pattern =
                                Url::match_patern(&req_url.path.clone(), &route.path.clone());
                            if match_pattern.0 == true && req_url.req_type == route.req_type {
                                let mut res = Response::new(stream.try_clone().unwrap());
                                req.params = match_pattern.1;

                                (route.handle)(req, res, public_var_clone);
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
                                            res2.send_bytes(&data, None);
                                        }
                                        Err(err) => {
                                            //println!("There was error serving this file!");
                                            res2.send_code(404);
                                        }
                                    }
                                    sent = true;
                                    break;
                                }
                            }
                            if sent == false {
                                res2.send_code(404);
                            }
                        }
                    }
                    stream.flush().expect("Failed to flush");
                });
            }
        });
        Ok(())
    }

    ///Locks the thread from stoping (put it in the end of the main file to keep the server running);

    pub fn lock() {
        let dur = Duration::from_secs(5);
        loop {
            std::thread::sleep(dur);
        }
    }
}
