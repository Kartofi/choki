use std::fs;
use std::io::Write;

use choki::structs::{Cookie, Header, Request, Response};
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
                res.send_string("111111111111111111112222222222222222222dddddddddddddddddddddddddddddddddddddddd");
            },
        )
        .unwrap();
    server.new_static("/images", "./tests/static").unwrap();
    server.listen(3000, None).unwrap();
    Server::<i32>::lock();
}
