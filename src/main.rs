use std::fs::{self, File};
use std::io::{BufReader, Write};

use choki::utils::request::Request;
use choki::utils::response::Response;
use choki::utils::structs::ContentType;
use choki::Server;

fn main() {
    let mut server: Server<u8> = Server::new(Some(10_000_000), None);
    server
        .get(
            "/watch/[id]",
            |req: Request, mut res: Response, public_var: Option<u8>| {
                res.send_string("HI");
            },
        )
        .unwrap();
    server
        .post(
            "/",
            |req: Request, mut res: Response, public_var: Option<u8>| {
                let str = req.user_agent.unwrap();

                res.send_string("ddd");
            },
        )
        .unwrap();
    server
        .post(
            "/filetest",
            |req: Request, mut res: Response, public_var: Option<u8>| {
                let str = req.user_agent.unwrap();
                res.send_code(200);
                //println!("{}", String::from_utf8_lossy(&req.body.unwrap()));
            },
        )
        .unwrap();
    server
        .get(
            "/",
            |req: Request, mut res: Response, public_var: Option<u8>| {
                let stream = File::open("./image.png").unwrap();
                let reader = BufReader::new(stream);
                res.pipe_stream(reader, Some(ContentType::Png));
            },
        )
        .unwrap();
    server.new_static("/images", "./tests/static").unwrap();
    server.listen(3000, None, Some(100)).unwrap();
    Server::<i32>::lock();
}
