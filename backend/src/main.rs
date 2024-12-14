//mod nodb;
//mod note_db;
mod api;
mod config;
mod http;
mod my_logger;
use crate::api::handle_api;
use crate::config::*;
use crate::http::parse_http_header;
use crate::my_logger::*;
use std::env;
use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream, frontend_dir: String) {
    let buf_reader = BufReader::new(&mut stream);
    let header = match parse_http_header(buf_reader) {
        Ok(h) => h,
        Err(e) => {
            warn!("HTTP Header error: {}", e);
            http_respond_error(stream);
            return;
        }
    };

    if header.path.starts_with("/api") {
        handle_api(stream, header);
        return;
    }
    match header.method.as_str() {
        "GET" => {}
        "POST" => {}
        _ => {
            warn!("Unsupported method: {}", header.method);
            http_respond_error(stream);
            return;
        }
    }

    let path_bytes = header.path.as_bytes();
    if path_bytes[path_bytes.len() - 1] == b'/' {
        http_respond_file(stream, frontend_dir + &header.path + "index.html");
    } else {
        http_respond_file(stream, frontend_dir + &header.path);
    }
}

fn http_respond_error(mut stream: TcpStream) {
    let response = "HTTP/1.1 404 NOT FOUND\r\n";
    stream.write_all(response.as_bytes()).unwrap();
}

fn http_respond_file(mut stream: TcpStream, fp: String) {
    // TODO: Check permissions
    let result = fs::read(fp.clone());
    let contents = match result {
        Ok(c) => c,
        Err(e) => {
            warn!("fs::read({}) - {}", fp, e);
            http_respond_error(stream);
            return;
        }
    };
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() <= 1 {
        println!("Expected <config.json> argument");
        return;
    }
    let config_fp = args[1].clone();
    let cfg = match parse_config_file(&config_fp) {
        Ok(c) => c,
        Err(e) => panic!("{}", e),
    };
    my_logger::init();
    println!("{:#?}", cfg);
    let listener = match TcpListener::bind(&cfg.address) {
        Ok(l) => l,
        Err(e) => panic!("{}", e),
    };

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                warn!("{}", e);
                continue;
            }
        };
        info!(
            "Connection established with {}!",
            stream.peer_addr().unwrap()
        );
        handle_connection(stream, cfg.frontend_dir.clone());
    }
}
