use std::fs::{ self, File };
use std::io::{ BufReader, Read, Seek, SeekFrom, Write };

use choki::src::request::Request;
use choki::src::response::Response;
use choki::src::structs::{ ContentType, Header, RequestType, ResponseCode, Url };
use choki::Server;

fn main() {
    let mut server: Server<u8> = Server::new(None, None);
    server
        .on(
            RequestType::Other("Custom".to_string()),
            "/",
            |req: Request, mut res: Response, public_var: Option<u8>| {
                res.send_string("HI custom one");
            }
        )
        .unwrap();
    server.use_middleware(|url: &Url, req: &Request, res: &mut Response, public_var: &Option<u8>| {
        println!("Ip {}", req.ip.clone().unwrap_or_default());
        return true;
    });
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

    server.listen(3000, None, None, || { println!("Server is listening on port 3000") }).unwrap();
    Server::<i32>::lock();
}
