A simple http server library built from scratch.
<br>
Using only the <a href="https://crates.io/crates/threadpool">threadpool</a> crate and <a href="https://crates.io/crates/num_cpus">num_cpus</a>. (and ofc the built in std)
<br>
Heavily inspired by express.js

<a href="https://expressjs.com/">https://expressjs.com/

# 📂・Installation

```powershell
cargo add choki
```

or add it in your Cargo.toml

```powershell
choki = "1.0.0"
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
  let mut server: Server = Server::new(Some(1024)); // you can also type None if you dont want any restrictions
```

The number in the () is the max content length of the request or in simple terms the max size of the request sent from the client.

## Create GET endpoint

```rust
server.get("/".to_string(), |req: Request, mut res: Response| {
    res.send_string("HI");
}).unwrap();
```

## Create POST endpoint

```rust
server.post("/".to_string(), |req: Request, mut res: Response| {
    res.send_string("HI");
}).unwrap();
```

## Create STATIC endpoint

```rust
 server.new_static("/images".to_string(), "./tests/images".to_string()).unwrap(); // The first one is the path in the browser for example: example.com/images and the second one is the exposed path from the computer(local)
```

## Response

So they are four simple functions

```rust
res.send_bytes(&mut self, data: &[u8], content_type: Option<ContentType>) // sends raw bytes with content type you provide (you can provide ContentType::None and let the browser decide)
```

```rust
res.send_string(&mut self, data: &str) // sends string as response
```

```rust
res.send_json(&mut self, data: &str) // sends json as response
```

```rust
res.send_code(&mut self, code: usize) // sends a HTTP response code (404,200...)
```

## Request

When you create an endpoint you have Request and Response.
<br>
The request holds info about the request.

```rust
pub struct Request {
    pub query: HashMap<String, String>, // for example in the url www.example.com/?name=Kartof the query will be ["name" => "Kartof"] as hashmap
    pub user_agent: Option<String>, // this is the user agent from which the user accesses the website
    pub content_length: usize, // the length of the request (they are no implementations for multy form thingy so its not so useful)
}
```

## The final

You need to make the server actually 'listen' for requests so use this method:

```rust
server.listen(port: u32, address: Option<String>) -> Result<(), HttpServerError>
```

And finally because you wanna keep the main thread running or else the server will close as soon as the code runs.
<br>
<br>
Add this at the end of your file

```rust
Server::lock();
```

Also in the src folder there is a main.rs file which can be used as an example.

## And thats it enjoy using it and keep in mind THIS IS NOT PRODUCTION READY!!!
