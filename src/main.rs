use std::fs::{ self, File };
use std::io::{ BufReader, Read, Seek, SeekFrom, Write };

use choki::utils::request::Request;
use choki::utils::response::Response;
use choki::utils::structs::{ ContentType, Header, ResponseCode };
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
        .get("/video", |req: Request, mut res: Response, public_var: Option<u8>| {
            // Open the file
            let mut file = File::open("./tests/static/large.mp4").unwrap();
            let size = file.metadata().unwrap().len(); // Get the file size
            let mut buf_reader = BufReader::new(file);

            let mut start: u64 = 0;
            let mut end: u64 = size; // Default to the end of the file if no Range is provided

            // Parse the Range header if it exists
            for header in req.headers.iter() {
                if header.name == "Range" {
                    let range_str = header.value.replace("bytes=", "");
                    let mut parts: Vec<&str> = range_str.split('-').collect();
                    parts.retain(|item| !item.is_empty());

                    if parts.len() == 2 {
                        start = parts[0].parse::<u64>().unwrap_or(0); // Start position
                        end = parts[1].parse::<u64>().unwrap_or(size - 1); // End position
                    }
                    break;
                }
            }

            // Seek to the starting byte position
            buf_reader.seek(SeekFrom::Start(start)).unwrap();

            // Set the headers for range response
            res.set_header(&Header::new("Accept-Ranges", "bytes"));
            res.set_header(
                &Header::new("Content-Range", &format!("bytes {}-{}/{}", start, end - 1, size - 1))
            );
            let chunk_size = (end - start + 1) as usize;

            // Calculate the length of the chunk we're sending

            res.set_status(&ResponseCode::PartialContent);
            // Send the chunk as     a response
            res.pipe_stream(buf_reader, Some(ContentType::Mp4), Some(&(chunk_size as u64)));
        })
        .unwrap();
    server
        .get("/", |req: Request, mut res: Response, public_var: Option<u8>| {
            let mut file = File::open("./tests/index.html").unwrap();
            let size = file.metadata().unwrap().len();
            let mut buf_reader = BufReader::new(file);
            let mut data: Vec<u8> = Vec::new();
            buf_reader.read_to_end(&mut data).unwrap();
            res.send_bytes_chunked(&data, Some(ContentType::Html));
        })
        .unwrap();
    server.new_static("/images", "./tests/static").unwrap();
    server.new_static("/images2", "./tests/static").unwrap();
    server.listen(3000, None, None).unwrap();
    Server::<i32>::lock();
}
