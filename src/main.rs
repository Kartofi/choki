use std::fs::{ self, File };
use std::io::{ BufReader, Read, Write };

use choki::utils::request::Request;
use choki::utils::response::Response;
use choki::utils::structs::ContentType;
use choki::Server;

fn main() {
    let mut server: Server<u8> = Server::new(None, None);
    server
        .get("/watch/[id]", |req: Request, mut res: Response, public_var: Option<u8>| {
            res.send_string("HI");
        })
        .unwrap();
    server
        .post("/", |req: Request, mut res: Response, public_var: Option<u8>| {
            res.send_string("ddd");
        })
        .unwrap();
    server
        .post("/filetest", |mut req: Request, mut res: Response, public_var: Option<u8>| {
            res.send_code(200);
            let body = req.body();
            println!("{}", String::from_utf8_lossy(body[0].data));
        })
        .unwrap();
    server
        .get("/", |req: Request, mut res: Response, public_var: Option<u8>| {
            res.send_string("123");
        })
        .unwrap();
    server.new_static("/images", "./tests/static").unwrap();
    server.listen(3000, None, Some(100)).unwrap();
    Server::<i32>::lock();
}
