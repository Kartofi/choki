use std::{io::Write, time::Duration};

use choki::structs::{Request, Response};
use choki::Server;

fn main() {
    let mut server = Server::new();
    server
        .get("/".to_string(), |req: Request, mut res: Response| {
            let str = req.user_agent.unwrap();
            res.send_string(&str);
        })
        .unwrap();
    server.listen(3000).unwrap();

    let dur = Duration::from_secs(2);
    loop {
        std::thread::sleep(dur);
    }
}
