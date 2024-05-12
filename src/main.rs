use std::time::Duration;

use choki::Server;

fn main() {
    let mut server = Server::new();
    server.get("/".to_string()).unwrap();
    server.listen(3000).unwrap();

    let dur = Duration::from_secs(2);
    loop {
        std::thread::sleep(dur);
    }
}
