use bumpalo::Bump;
use src::utils::logger;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::write;
use std::fs::File;
use std::hash::Hash;
use std::path::{ self, Path };
use std::time::{ Duration, Instant };
use std::{ fs, io, thread, vec };
use std::{ io::Write, net::* };

use std::io::{ BufRead, BufReader, Error, Read };
use structs::*;
use threadpool::ThreadPool;

extern crate num_cpus;

pub mod src;

use src::request::Request;
use src::response::Response;
use src::*;

pub struct Server<T: Clone + std::marker::Send + 'static> {
    active: bool,
    pub max_content_length: usize,
    pub endpoints: Vec<EndPoint<T>>,
    pub static_endpoints: HashMap<String, String>,

    pub public_var: Option<T>,

    middleware: Option<
        fn(url: &Url, req: &Request, res: &mut Response, public_var: &Option<T>) -> bool
    >,
    logger: Option<fn(input: &HttpServerError)>,
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
            middleware: None,
            logger: None,
        };
    }
    ///Add function as middleware (just before sending response). The response is a bool. If it's true, the request will continue; if it's false, it will stop.
    pub fn use_middleware(
        &mut self,
        handle: fn(url: &Url, req: &Request, res: &mut Response, public_var: &Option<T>) -> bool
    ) {
        self.middleware = Some(handle);
    }
    ///Add your own custom error logger function
    pub fn use_logger(&mut self, handle: fn(input: &HttpServerError)) {
        self.logger = Some(handle);
    }
    ///Creates a new static url
    /// For example a folder named "images" on path /images every image in that folder will be exposed like "/images/example.png"
    pub fn new_static(&mut self, path: &str, folder: &str) -> Result<(), HttpServerError> {
        if self.active == true {
            return Err(HttpServerError::new("Server is already running!"));
        }
        let path_: &Path = Path::new(&folder);

        if path_.is_dir() == false || path_.exists() == false {
            return Err(
                HttpServerError::new("Folder does not exist or the path provided is a file!")
            );
        }
        let mut path = path.to_owned();
        if
            (self.endpoints.len() > 0 &&
                self.endpoints.iter().any(|x| x.path == path && x.req_type == RequestType::Get)) ||
            (self.static_endpoints.len() > 0 && self.static_endpoints.iter().any(|x| x.0 == &path))
        {
            return Err(HttpServerError::new("Endpoint already exists!"));
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
        handle: fn(
            req: Request,
            res: Response,
            public_var: Option<T>
        ) -> Result<(), HttpServerError>
    ) -> Result<(), HttpServerError> {
        if self.active == true {
            return Err(HttpServerError::new("Server is already running!"));
        }
        let mut path = path.to_owned();
        if
            (self.endpoints.len() > 0 &&
                self.endpoints.iter().any(|x| x.path == path && x.req_type == req_type)) ||
            (self.static_endpoints.len() > 0 && self.static_endpoints.iter().any(|x| x.0 == &path))
        {
            return Err(HttpServerError::new("Endpoint already exists!"));
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
        handle: fn(
            req: Request,
            res: Response,
            public_var: Option<T>
        ) -> Result<(), HttpServerError>
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, RequestType::Get, handle)
    }

    ///Creates a new POST endpoint
    pub fn post(
        &mut self,
        path: &str,
        handle: fn(
            req: Request,
            res: Response,
            public_var: Option<T>
        ) -> Result<(), HttpServerError>
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, RequestType::Post, handle)
    }
    ///Creates a new PUT endpoint
    pub fn put(
        &mut self,
        path: &str,
        handle: fn(
            req: Request,
            res: Response,
            public_var: Option<T>
        ) -> Result<(), HttpServerError>
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, RequestType::Put, handle)
    }
    ///Creates a new DELETE endpoint
    pub fn delete(
        &mut self,
        path: &str,
        handle: fn(
            req: Request,
            res: Response,
            public_var: Option<T>
        ) -> Result<(), HttpServerError>
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, RequestType::Delete, handle)
    }

    ///Universal endpoint creator
    pub fn on(
        &mut self,
        req_type: RequestType,
        path: &str,
        handle: fn(
            req: Request,
            res: Response,
            public_var: Option<T>
        ) -> Result<(), HttpServerError>
    ) -> Result<(), HttpServerError> {
        self.new_endpoint(path, req_type, handle)
    }
    ///Starts listening on the given port.
    /// If no provided threads will use cpu threads as value. The higher the value the higher the cpu usage.
    /// The on_complete function is executed after the listener has started.
    pub fn listen(
        &mut self,
        port: u32,
        address: Option<&str>,
        threads: Option<usize>,
        on_complete: fn()
    ) -> Result<(), HttpServerError> {
        if port > 65_535 {
            return Err(HttpServerError::new("Invalid port: port must be 0-65,535"));
        }
        if self.active == true {
            return Err(HttpServerError::new("The server is already running!"));
        }
        self.active = true;

        let pool: ThreadPool = ThreadPool::new(threads.unwrap_or(num_cpus::get()));
        let mut routes = self.endpoints.clone();
        let static_routes = self.static_endpoints.clone();

        let middleware = self.middleware.clone();
        let logger = self.logger.unwrap_or(logger::eprint);

        let max_content_length = self.max_content_length.clone();
        let public_var = self.public_var.clone();

        let address = address.unwrap_or("0.0.0.0").to_owned();

        order_routes(&mut routes); // Order them so the first one are without params

        thread::spawn(move || {
            let tcp: TcpListener = TcpListener::bind(format!("{}:{}", address, port)).unwrap();

            for stream in tcp.incoming() {
                let routes_clone = routes.clone();
                let static_routes_clone = static_routes.clone();
                let max_content_length_clone = max_content_length.clone();
                let public_var_clone = public_var.clone();

                pool.execute(move || {
                    let stream = stream.unwrap();
                    let res = Self::handle_request(
                        stream,
                        max_content_length_clone,
                        routes_clone,
                        static_routes_clone,
                        middleware,

                        public_var_clone
                    );
                    if res.is_err() {
                        logger(&res.unwrap_err());
                    }
                });
            }
        });

        on_complete();

        Ok(())
    }
    fn handle_request(
        stream: TcpStream,
        max_content_length: usize,
        routes: Vec<EndPoint<T>>,
        static_routes: HashMap<String, String>,
        middleware: Option<
            fn(url: &Url, req: &Request, res: &mut Response, public_var: &Option<T>) -> bool
        >,
        public_var: Option<T>
    ) -> Result<(), HttpServerError> {
        let bump = Bump::new(); // Allocator

        let mut bfreader: BufReader<TcpStream> = BufReader::new(
            stream.try_clone().expect("Failed to create buffer reader")
        );

        let mut headers_string: String = "".to_string();

        let mut line = "".to_owned();
        loop {
            match bfreader.read_line(&mut line) {
                Ok(size) => {
                    if size <= 2 {
                        break;
                    }
                }
                Err(e) => {
                    return Err(HttpServerError::new("Error reading request headers!"));
                }
            }

            headers_string.push_str(&line);

            line = "".to_string();
        }

        let lines: Vec<&str> = headers_string.lines().collect();

        if lines.len() == 0 {
            return Err(HttpServerError::new("No headers!"));
        }
        let req_url = Url::parse(lines[0]).unwrap();

        let mut req = Request::parse(&lines, Some(req_url.query), None)?;

        if let Some(socket) = stream.peer_addr().ok() {
            req.ip = Some(socket.ip().to_string());
        }
        let content_encoding = req.content_encoding.clone();
        let mut res = Response::new(stream.try_clone().unwrap(), content_encoding.clone());
        // Check if supported req type
        let content_type = req.content_type.clone().unwrap_or(ContentType::None);

        let has_body = content_type != ContentType::None && req.content_length > 0;

        // Check if body in GET or HEAD
        if
            has_body &&
            (req_url.req_type == RequestType::Get || req_url.req_type == RequestType::Head)
        {
            req.read_only_body(&mut bfreader);
            res.send_code(ResponseCode::BadRequest)?;
            return Err(HttpServerError::new("Bad request!"));
        }
        //Check if over content length
        if max_content_length > 0 && req.content_length > max_content_length && has_body {
            req.read_only_body(&mut bfreader);
            res.send_code(ResponseCode::ContentTooLarge)?;
            return Err(HttpServerError::new("Content too large!"));
        }
        // Middleware
        if middleware.is_some() {
            let result = middleware.unwrap()(
                &(Url {
                    path: req_url.path.clone(),
                    req_type: req_url.req_type.clone(),
                    query: HashMap::new(),
                }),
                &req,
                &mut res,
                &public_var
            );
            if result == false {
                return Ok(());
            }
        }
        //
        let mut matching_routes: Vec<EndPoint<T>> = Vec::new();
        let mut params: HashMap<String, String> = HashMap::new();
        // Check for matching pattern
        for route in routes {
            let match_pattern = Url::match_patern(&req_url.path.clone(), &route.path.clone());
            if match_pattern.0 == true {
                matching_routes.push(route);
                if params.is_empty() {
                    params = match_pattern.1;
                }
            }
        }

        if matching_routes.len() > 0 {
            let routes: Vec<EndPoint<T>> = matching_routes
                .into_iter()
                .filter(|route| route.req_type == req_url.req_type)
                .collect();

            if routes.len() == 0 {
                if has_body {
                    req.read_only_body(&mut bfreader);
                }

                res.send_code(ResponseCode::MethodNotAllowed)?;
                return Err(HttpServerError::new("Method not allowed!"));
            }
            let route = &routes[0];

            req.params = params;

            if has_body {
                req.extract_body(&mut bfreader, bump)?;
            }

            (route.handle)(req, res, public_var)?;

            return Ok(());
        }

        let mut sent = false;
        for route in static_routes {
            if req_url.path.starts_with(&route.0) {
                let parts: Vec<&str> = req_url.path.split(&route.0).collect();
                if parts.len() == 0 {
                    continue;
                }
                let path_str = route.1 + parts[1];
                let path = Path::new(&path_str);

                if path.exists() && path.is_file() {
                    match File::open(path) {
                        Ok(file) => {
                            let metadata = file.metadata();

                            let bfreader = BufReader::new(file);

                            let mut size: Option<u64> = None;
                            if metadata.is_ok() {
                                size = Some(metadata.unwrap().len());
                            }
                            let extension = path.extension().unwrap_or_default();

                            let content_type = ContentType::from_extension(
                                extension.to_str().unwrap_or_default()
                            );

                            res.pipe_stream(bfreader, content_type, size.as_ref())?;
                        }
                        Err(_err) => {
                            res.send_code(ResponseCode::NotFound)?;
                        }
                    }
                } else {
                    res.send_code(ResponseCode::NotFound)?;
                }

                sent = true;
                break;
            }
        }
        if sent == false {
            res.send_code(ResponseCode::NotFound)?;
            return Err(HttpServerError::new("Not found!"));
        }
        return Ok(());
    }
    ///Locks the thread from stoping (put it in the end of the main file to keep the server running);
    pub fn lock() {
        let dur = Duration::from_secs(5);
        loop {
            std::thread::sleep(dur);
        }
    }
}

fn order_routes<T: Clone + Send + 'static>(routes: &mut Vec<EndPoint<T>>) {
    routes.sort_by(|a, b| {
        let path_a = &a.path;
        let path_b = &b.path;
        if path_a.contains("[") && path_a.contains("]") && !path_b.contains("[") {
            return Ordering::Greater;
        }
        if path_b.contains("[") && path_b.contains("]") && !path_a.contains("[") {
            return Ordering::Less;
        }
        if path_a.len() > path_b.len() {
            return Ordering::Greater;
        }
        if path_a.len() < path_b.len() {
            return Ordering::Less;
        }
        return Ordering::Equal;
    });
}
