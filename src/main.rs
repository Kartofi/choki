use std::fs::{self, File};
use std::io::{BufReader, Write};

use choki::structs::{ContentType, Cookie, Header, Request, Response};
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
            },
        )
        .unwrap();
    server
        .get(
            "/",
            |req: Request, mut res: Response, public_var: Option<u8>| {
                let file = File::open("tests/static/test/image.gif").expect("Failed to open file");
                let reader = BufReader::new(file);
                res.pipe_stream(reader, Some(ContentType::Png));
            },
        )
        .unwrap();
    server.new_static("/images", "./tests/static").unwrap();
    server.listen(3000, None).unwrap();
    Server::<i32>::lock();
}
