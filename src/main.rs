use std::time::Duration;

use choki::Server;

fn main() {
    let server = Server::listen(100);

    let dur = Duration::from_secs(2);
    loop {
        std::thread::sleep(dur);
    }
}
