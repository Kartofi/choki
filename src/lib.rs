use std::collections::HashMap;
use std::fmt::write;
use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};
use std::{fs, io, thread};
use std::{io::Write, net::*};

use std::io::{BufRead, BufReader, Error, Read};
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
    pub fn new_static(&mut self, path: &str, folder: &str) -> Result<(), HttpServerError> {
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
        let mut path = path.to_owned();
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
        self.static_endpoints.insert(path, folder.to_owned());
        Ok(())
    }
    fn new_endpoint(
        &mut self,
        path: &str,
        req_type: RequestType,
        handle: fn(req: Request, res: Response, public_var: Option<T>),
    ) -> Result<(), HttpServerError> {
        if self.active == true {
            return Err(HttpServerError::new(
                "Server is already running!".to_string(),
            ));
        }
        let mut path = path.to_owned();
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
        path: &str,
        handle: fn(req: Request, res: Response, public_var: Option<T>),
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, RequestType::Get, handle)
    }

    ///Creates a new POST endpoint
    pub fn post(
        &mut self,
        path: &str,
        handle: fn(req: Request, res: Response, public_var: Option<T>),
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, RequestType::Post, handle)
    }
    ///Starts listening on the given port.
    pub fn listen(&mut self, port: u32, address: Option<&str>) -> Result<(), HttpServerError> {
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

        let address = address.unwrap_or("0.0.0.0").to_owned();

        thread::spawn(move || {
            let tcp: TcpListener = TcpListener::bind(format!("{}:{}", address, port)).unwrap();

            for stream in tcp.incoming() {
                let routes_clone = routes.clone();
                let static_routes_clone = static_routes.clone();
                let max_content_length_clone = max_content_length.clone();
                let public_var_clone = public_var.clone();

                pool.execute(move || {
                    let mut stream: TcpStream = stream.unwrap();

                    let mut total_read = 0;
                    let mut buffer: Vec<u8> = Vec::new();
                    let mut buffer2: [u8; 1024] = [0; 1024];

                    let mut bfreader: BufReader<TcpStream> =
                        BufReader::new(stream.try_clone().expect("Failed to create Buffer Reader"));

                    let mut headers_string: String = "".to_string();
                    let mut line = "".to_owned();
                    loop {
                        bfreader.read_line(&mut line).unwrap();

                        if &line == "\r\n" {
                            break;
                        }

                        headers_string.push_str(&line);

                        line = "".to_string();
                    }

                    let lines: Vec<&str> = headers_string.lines().collect();
                    if lines.len() == 1 {
                        return;
                    }
                    let req_url = Url::parse(lines[0]).unwrap();

                    let mut req = Request::parse(lines, Some(req_url.query), None);
                    let content_encoding = req.content_encoding.clone();
                    let mut res =
                        Response::new(stream.try_clone().unwrap(), content_encoding.clone());

                    if max_content_length_clone > 0 && req.content_length > max_content_length_clone
                    {
                        res.send_code(413);
                    } else {
                        let mut sent: bool = false;
                        for route in routes_clone {
                            let match_pattern =
                                Url::match_patern(&req_url.path.clone(), &route.path.clone());
                            if match_pattern.0 == true && req_url.req_type == route.req_type {
                                req.params = match_pattern.1;

                                let mut ll_data: String = "".to_string();
                                let mut count = 0;
                                let mut total_size = 0;
                                let mut body_length_to_read = 0;

                                loop {
                                    if count < 3 {
                                        match bfreader.read_line(&mut ll_data) {
                                            Ok(size) => {
                                                count += 1;
                                                total_read += size;
                                            }
                                            Err(_) => {
                                                break;
                                            }
                                        }
                                        if count >= 3 {
                                            body_length_to_read = req.content_length - total_read;
                                            println!("{}", body_length_to_read);
                                        }
                                        continue;
                                    }

                                    match bfreader.read(&mut buffer2) {
                                        Ok(size) => {
                                            total_size += size;

                                            println!(
                                                "Read {} bytes, total: {} / expected: {}",
                                                size,
                                                total_size,
                                                req.content_length - total_read
                                            );

                                            buffer.extend_from_slice(&buffer2[..size]);
                                            if size == 0 || total_size >= body_length_to_read {
                                                break; // End of file
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                                let mut cleaned = buffer.to_vec();
                                println!("{}", cleaned.len());
                                cleaned = cleaned[2..cleaned.len()
                                    - (req.boudary.clone().unwrap_or_default().len() + 2)]
                                    .to_vec();
                                println!("{}", cleaned.len());
                                fs::write("./image.png", cleaned).unwrap();

                                (route.handle)(req, res, public_var_clone);
                                sent = true;
                                break;
                            }
                        }
                        if sent == false {
                            let mut res2 =
                                Response::new(stream.try_clone().unwrap(), content_encoding);

                            for route in static_routes_clone {
                                if req_url.path.starts_with(&route.0) {
                                    let parts: Vec<&str> = req_url.path.split(&route.0).collect();
                                    if parts.len() == 0 {
                                        continue;
                                    }
                                    let path_str = route.1 + parts[1];
                                    let path = Path::new(&path_str);
                                    if path.exists() {
                                        match File::open(path) {
                                            Ok(file) => {
                                                let bfreader = BufReader::new(file);
                                                res2.pipe_stream(bfreader, None);
                                            }
                                            Err(err) => {
                                                res2.send_code(404);
                                            }
                                        }
                                    } else {
                                        res2.send_code(404);
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
fn remove_repeating_pattern(input: &[u8], pattern: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let pattern_len = pattern.len();
    let mut i = 0;

    while i < input.len() {
        // Check if the current slice matches the pattern
        if i + pattern_len <= input.len() && &input[i..i + pattern_len] == pattern {
            i += pattern_len; // Skip the pattern
        } else {
            result.push(input[i]); // Add the current byte to the result
            i += 1; // Move to the next byte
        }
    }

    result
}
