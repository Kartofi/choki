use std::thread;
use std::{io::Write, net::*};

use std::io::Read;
use structs::*;
use threadpool::ThreadPool;

mod structs;
pub struct Server {
    active: bool,
    pub endpoints: Vec<EndPoint>,
}

impl Server {
    pub fn new() -> Server {
        return Server {
            active: false,
            endpoints: Vec::new(),
        };
    }
    pub fn get(&mut self, path: String) -> Result<(), HttpServerError> {
        if self.active == true {
            return Err(HttpServerError::new(
                "Server is already running!".to_string(),
            ));
        }
        if self.endpoints.len() > 0 && self.endpoints.iter().any(|x| x.path == path) {
            return Err(HttpServerError::new("Endpoint already exists!".to_string()));
        }
        self.endpoints.push(EndPoint::new(path, RequestType::Get));
        Ok(())
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
        thread::spawn(move || {
            let tcp: TcpListener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
            for stream in tcp.incoming() {
                let reoutes_clone = routes.clone();
                pool.execute(move || {
                    let mut stream: TcpStream = stream.unwrap();

                    println!("NEW CONNECTION");
                    let mut buffer = [0; 1024];
                    stream.read(&mut buffer).expect("Failed to read");
                    let string_req = String::from_utf8_lossy(&buffer);
                    let lines: Vec<&str> = string_req.lines().collect();

                    println!("{}", lines[0]);
                    let req_url = Url::parse(lines[0]).unwrap();
                    for route in reoutes_clone {
                        if route.path == req_url.path {
                            let response = "HTTP/1.1 200 OK\r\n\r\n Hi";
                            stream
                                .write_all(response.as_bytes())
                                .expect("Failed to write");
                        }
                    }

                    stream.flush().expect("Failed to flush");
                });
            }
        });
        Ok(())
    }
}
