A simple http server library built from scratch.
<br>
Using only the <a href="https://crates.io/crates/threadpool">threadpool</a>, <a href="https://crates.io/crates/num_cpus">num_cpus</a> and <a href="https://crates.io/crates/flate2">flate2</a>. (and ofc the built in std)

Using tikv-jemallocator allocator because the default one is making it eat ram.
<br>

<b>Heavily inspired by <a href="https://expressjs.com/">ExpressJs</a></b>

# 📂・Installation

```powershell
cargo add choki
```

or add it in your Cargo.toml

```powershell
choki = "1.1.0"
```

# 💡・Features

- Create GET and POST endpoints and use them like you are used to in express.js
  <br>
- Create Static endpoints

## Declare them in your file

```rust
use choki::structs::{Request, Response};
use choki::Server;
```

## Create a object from the class called `Server`

```rust
  let mut server: Server<u8> = Server::new(max_content_length: Some(1024),public_var: None);
```

You can set max request size and a public var that is in this case type u8 and it is cloned to every thread/request.

## Create `GET` endpoint

```rust
server.get("/".to_string(), |req: Request, mut res: Response, public_var: Option<u8>| {
    res.send_string("HI");
}).unwrap();
```

## Create `POST` endpoint

```rust
server.post("/".to_string(), |req: Request, mut res: Response, public_var: Option<u8>| {
    res.send_string("Letter!");
}).unwrap();
```

## Create `PUT` endpoint

```rust
server.put("/put".to_string(), |req: Request, mut res: Response, public_var: Option<u8>| {
    res.send_string("Hello world!");
}).unwrap();
```

## Create `DELETE` endpoint

```rust
server.delete("/delete".to_string(), |req: Request, mut res: Response, public_var: Option<u8>| {
    res.send_string("Boom!");
}).unwrap();
```

## Create `STATIC` endpoint

```rust
 server.new_static("/images", "./tests/images").unwrap(); // The first one is the path in the browser for example: example.com/images and the second one is the exposed path from the computer(local)
```

## Create endpoints with params

As of `1.0.8` choki supports params

```rust
 server.post("/search/[id]".to_string(), |req: Request, mut res: Response, public_var: Option<u8>| {
    println!("{}", req.params.get("id").unwrap()); // if i make request to /search/pizza this will print pizza
    res.send_string("HI");
}).unwrap();
```

Also queries and body are supported.

`req.body` is a `Vec<BodyItem>` which are the items in the body (if multipart-form and etc. (you can check it `req.content_type`));

`req.query` are the queries (/search?name=123 the thing after ?)

## Response

So they are four simple functions
There two types of responses:

1. Sending the data in one big chunk

```rust
res.send_bytes(&mut self, data: &[u8], content_type: Option<ContentType>) // sends raw bytes with content type you provide (you can provide ContentType::None and let the browser decide)
```

```rust
res.send_string(&mut self, data: &str) // sends string as response
```

```rust
res.send_json(&mut self, data: &str) // sends json as response
```

2. Sending it chunked

```rust
res.send_bytes_chunked(&mut self, data: &[u8], content_type: Option<ContentType>)
```

```rust
res.send_string_chunked(&mut self, data: &str)
```

```rust
res.send_json_chunked(&mut self, data: &str)
```

```rust
res.send_code(&mut self, code: usize) // sends a HTTP response code (404,200...)
```

Also you can send download bytes or streams

```rust
res.send_download_bytes(&mut self, data: &[u8], file_name: &str) // Sends bytes and the browser is goind to start to download it.
```

```rust
res.send_download_stream(
        &mut self,
        stream: BufReader<impl Read>,
        file_name: &str,
        file_size: Option<&u64>) // Pipes a stream and the browser is goind to start to download it.
```

And piping stream

```rust
res.pipe_stream(
        &mut self,
        mut stream: BufReader<impl Read>,
        content_type: Option<ContentType>,
        stream_size: Option<&u64>
    )
```

Sending raw code

```rust
res.send_code(&mut self, code: ResponseCode) // sends a HTTP response code (404,200...)
```

as of `1.0.3` you can set or delete cookies and ofc read them.

```rust
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: String,
    pub expires: String,
}
```

You can read cookies using req.cookies (stored as a vec)

You can set/delete them using

```rust
res.set_cookie(cookie: &Cookie);
res.delete_cookie(name: &str);
```

as of `1.0.6` you can set or delete headers and ofc read them.

```rust
pub struct Header {
    pub name: String,
    pub value: String,
}
```

You can set/delete them using

```rust
res.set_header(header: &Header);
res.delete_cookie(name: &str);
```

## Request

When you create an endpoint you have Request and Response.
<br>
The request holds info about the request.

```rust
pub struct Request {
    pub query: HashMap<String, String>, // for example in the url www.example.com/?name=Kartof the query will be ["name" => "Kartof"] as hashmap
    pub params: HashMap<String, String>, // a hashmap containing every param with name and value
    pub headers: Vec<Header>,
    pub cookies: Vec<Cookie>,

    // User data
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub content_encoding: Option<Vec<Encoding>>, // The compression it can accept (zlib...)
    pub content_length: usize,
    // BODY
    pub content_type: Option<ContentType>, // The content type of the body
}
```

To get the body use the function `body()`

```rust
let body: Vec<BodyItem<'_>> = req.body();
```

## The final

You need to make the server actually 'listen' for requests so use this method:

```rust
  server.listen(3000, None, None, || { println!("Server is listening on port 3000") }).unwrap();
```

And finally because you wanna keep the main thread running or else the server will close as soon as the code runs.
<br>
<br>
Add this at the end of your file

```rust
  Server::<i32>::lock();
```

Also in the src folder there is a `main.rs` file which can be used as an example.

## And thats it enjoy using it and keep in mind THIS IS NOT PRODUCTION READY!!!
