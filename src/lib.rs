use std::thread;
use std::{io::Write, net::*};

use std::io::Read;
use threadpool::ThreadPool;
pub struct Server {}

impl Server {
    ///Starts the http server on the given port.
    pub fn listen(port: u8) -> Server {
        let pool: ThreadPool = ThreadPool::new(6);
        thread::spawn(move || {
            let tcp: TcpListener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
            for stream in tcp.incoming() {
                pool.execute(move || {
                    let mut stream: TcpStream = stream.unwrap();

                    println!("NEW CONNECTION");
                    let mut buffer = [0; 1024];
                    stream.read(&mut buffer).expect("Failed to read");

                    let response = "HTTP/1.1 200 OK\r\n\r\n Hi";
                    stream
                        .write_all(response.as_bytes())
                        .expect("Failed to write");

                    stream.flush().expect("Failed to flush");
                });
            }
        });

        return Server {};
    }
}
