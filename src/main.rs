use std::fs;

use choki::structs::{Cookie, Header, Request, Response};
use choki::Server;

fn main() {
    let mut server: Server<u8> = Server::new(Some(1024), None);
    server
        .get(
            "/watch/[id]".to_string(),
            |req: Request, mut res: Response, public_var: Option<u8>| {
                println!("{}", req.params.get("id").unwrap());

                res.send_string("HI");
            },
        )
        .unwrap();
    server
        .post(
            "/".to_string(),
            |req: Request, mut res: Response, public_var: Option<u8>| {
                let str = req.user_agent.unwrap();

                res.send_string("ddd");
            },
        )
        .unwrap();
    server
        .get(
            "/".to_string(),
            |req: Request, mut res: Response, public_var: Option<u8>| {
                println!("{}", req.query.len());
                res.set_header(&Header::new("naruto".to_string(), "value".to_string()));
                res.send_string("ddd");
            },
        )
        .unwrap();
    server
        .new_static("/images".to_string(), "./tests/static".to_string())
        .unwrap();
    server.listen(3000, None).unwrap();
    Server::<i32>::lock();
}
