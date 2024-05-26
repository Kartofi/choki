use std::fs;

use choki::structs::{Cookie, Request, Response};
use choki::Server;

fn main() {
    let mut server: Server = Server::new(Some(1024));
    server
        .get("/".to_string(), |req: Request, mut res: Response| {
            println!("{}", req.cookies[0].as_str());

            res.send_string("HI");
        })
        .unwrap();
    server
        .post("/".to_string(), |req: Request, mut res: Response| {
            let str = req.user_agent.unwrap();

            res.send_string("ddd");
        })
        .unwrap();
    server
        .new_static("/images".to_string(), "./tests/static".to_string())
        .unwrap();
    server.listen(3000, None).unwrap();
    Server::lock();
}
