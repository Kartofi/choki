use std::fs::{ self, File };
use std::io::{ BufReader, Read, Seek, SeekFrom, Write };

use choki::src::request::Request;
use choki::src::response::Response;
use choki::src::structs::{ ContentType, Header, ResponseCode };
use choki::Server;

fn main() {
    let mut server: Server<u8> = Server::new(None, None);
    server
        .get("/watch/[id]", |req: Request, mut res: Response, public_var: Option<u8>| {
            res.send_string("HI");
        })
        .unwrap();
    server
        .get("/test", |req: Request, mut res: Response, public_var: Option<u8>| {
            res.send_code(ResponseCode::Ok);
        })
        .unwrap();
    server
        .post("/filetest", |req: Request, mut res: Response, public_var: Option<u8>| {
            res.send_code(ResponseCode::Ok);
            let body: Vec<choki::src::structs::BodyItem<'_>> = req.body();
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
            let mut file = File::open("./tests/static/index.html").unwrap();
            let size = file.metadata().unwrap().len();
            let mut buf_reader = BufReader::new(file);
            let mut data: Vec<u8> = Vec::new();
            buf_reader.read_to_end(&mut data).unwrap();
            res.send_bytes_chunked(&data, Some(ContentType::Html));
        })
        .unwrap();
    server.new_static("/images", "./tests/static").unwrap();
    server.new_static("/images2", "./tests/static").unwrap();
    server.listen(3000, None, None, || { println!("Server is listening on port 3000") }).unwrap();
    Server::<i32>::lock();
}
