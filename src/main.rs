use std::fs::{ self, File };
use std::io::{ BufReader, Read, Write };

use choki::utils::request::Request;
use choki::utils::response::Response;
use choki::utils::structs::{ ContentType, ResponseCode };
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
        .post("/filetest", |req: Request, mut res: Response, public_var: Option<u8>| {
            res.send_code(ResponseCode::Ok);
            let body = req.body();
            println!("{}", body.len());
        })
        .unwrap();
    server
        .get("/filetest", |mut req: Request, mut res: Response, public_var: Option<u8>| {
            res.send_code(ResponseCode::Ok);
        })
        .unwrap();
    server
        .get("/", |req: Request, mut res: Response, public_var: Option<u8>| {
            let mut file = File::open("./tests/static/video.mkv").unwrap();
            let size = file.metadata().unwrap().len();
            let mut buf_reader = BufReader::new(file);

            res.pipe_stream(buf_reader, Some(ContentType::Mkv), Some(&size));
        })
        .unwrap();
    server.new_static("/images", "./tests/static").unwrap();
    server.new_static("/images2", "./tests/static").unwrap();
    server.listen(3000, None, None).unwrap();
    Server::<i32>::lock();
}
