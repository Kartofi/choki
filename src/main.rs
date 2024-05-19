use std::fs;
use std::{io::Write, time::Duration};

use choki::structs::{Request, Response};
use choki::Server;

fn main() {
    let mut server = Server::new(None);
    server
        .get("/".to_string(), |req: Request, mut res: Response| {
            let str = req.user_agent.unwrap();
            let file = fs::read("./tests/static/image.gif").unwrap();
            res.send_bytes(&file);
        })
        .unwrap();
    server
        .post("/".to_string(), |req: Request, mut res: Response| {
            let str = req.user_agent.unwrap();
            println!("{}", req.content_length);
            res.send_string("ddd");
        })
        .unwrap();
    server
        .new_static("/images".to_string(), "./tests/static".to_string())
        .unwrap();
    server.listen(3000).unwrap();

    let dur = Duration::from_secs(2);
    loop {
        std::thread::sleep(dur);
    }
}
