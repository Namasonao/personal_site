mod nodb;
mod note_db;
use std::net::TcpListener;

fn main() {
    println!("Hello");
    let listener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(l) => l,
        Err(e) => panic!("{}", e),
    };
}
