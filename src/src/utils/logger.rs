use crate::src::structs::HttpServerError;

pub fn eprint(input: &HttpServerError) {
    eprintln!("[ERROR] {}", input.reason);
}
