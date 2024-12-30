use crate::{info, warn, config::Config};
use std::io::{BufRead, BufReader, Error, Read, Write};
use std::net::{TcpStream, TcpListener};
use std::str;

type HttpHandlerT<'a> = Box<dyn HttpHandler + 'a>;
pub trait HttpHandler {
    fn handle(&self, request: HttpRequest) -> HttpResponse;
}

pub struct HttpServer<'a> {
    listener: TcpListener,
    default_handler: HttpHandlerT<'a>,
    config: &'a Config,
}

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
}

type Field = (String, String);

#[derive(Debug)]
pub struct HttpRequest {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub fields: Vec<Field>,
    pub body: Option<Vec<u8>>,
}

pub enum StatusCode {
    OK,
    BadRequest,
    NotFound,
    InternalError,
    NotImplemented,
}

pub struct HttpResponse {
    pub version: String,
    pub status_code: StatusCode,
    pub fields: Vec<Field>,
    pub body: Option<Vec<u8>>,
}

impl HttpResponse {
    pub fn new(code: StatusCode, body: Option<Vec<u8>>) -> HttpResponse {
        HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: code,
            fields: Vec::new(),
            body: body,
        }
    }

    fn respond(&self, stream: &mut TcpStream) -> Result<(), Error> {
        let mut response = self.version.clone();
        response += " ";
        use StatusCode::*;
        response += match self.status_code {
            OK => "200 OK",
            NotFound => "404 NOT FOUND",
            NotImplemented => "501 NOT IMPLEMENTED",
            BadRequest => "400 BAD REQUEST",
            InternalError => "500 INTERNAL SERVER ERROR",
            _ => todo!(),
        };
        response += "\r\n";

        for (left, right) in self.fields.iter() {
            response += &left;
            response += ": ";
            response += &right;
            response += "\r\n";
        }
        response += "\r\n";
        stream.write(response.as_bytes())?;

        if let Some(body) = &self.body {
            stream.write(&body)?;
        }

        Ok(())
    }
}

impl<'a> HttpServer<'a> {
    pub fn new(config: &'a Config, default_handler: HttpHandlerT<'a>) -> Result<HttpServer<'a>, Error> {
        let listener = match TcpListener::bind(&config.address) {
            Ok(l) => l,
            Err(e) => return Err(e),
        };
        Ok(HttpServer {
            listener: listener,
            default_handler: default_handler,
            config: config,
        })
    }

    pub fn listen(&self) {
        for stream in self.listener.incoming() {
            let mut stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    warn!("Invalid stream: {}", e);
                    continue;
                }
            };
            info!(
                "Connection established with {}",
                stream.peer_addr().unwrap()
            );
            let buf_reader = BufReader::new(&mut stream);
            let http_request = match parse_http(buf_reader) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Invalid HTTP: {}", e);
                    continue;
                }
            };
            info!("Parsed HTTP successfully");

            let http_response = self.default_handler.handle(http_request);
            if let Err(e) = http_response.respond(&mut stream) {
                warn!("Error responding: {}", e);
                continue;
            }
            info!("Responded HTTP");
        }
    }
}

fn parse_field(line: String) -> Option<Field> {
    let mut it = line.split(':');
    let key = it.next()?.to_string();
    let mut val = it.next()?.to_string();
    val.remove(0);
    Some((key, val))
}

pub fn parse_http(mut buf_reader: BufReader<&mut TcpStream>) -> Result<HttpRequest, &'static str> {
    let mut first_line = String::new();
    if let Err(_) = buf_reader.read_line(&mut first_line) {
        return Err("Empty request");
    }
    let l1: Vec<_> = first_line.split(' ').collect();
    if l1.len() != 3 {
        return Err("Error parsing first line");
    }
    let method = match l1[0] {
        "GET" => Method::Get,
        "POST" => Method::Post,
        _ => return Err("Unknown method"),
    };
    let path = l1[1].to_string();
    let version = l1[2].to_string();
    let mut fields: Vec<Field> = Vec::new();
    let mut l = String::new();
    while let Ok(_) = buf_reader.read_line(&mut l) {
        let _ = l.pop();
        let _ = l.pop();
        if l.is_empty() {
            break;
        }
        if let Some(f) = parse_field(l.clone()) {
            fields.push(f);
        }
        l.clear();
    }

    info!("{:?} {} {}", method, path, version);

    let mut http_body = Vec::new();
    if method == Method::Post {
        let mut exp_len = 0;
        for f in &fields {
            if f.0 == "Content-Length" {
                exp_len = match f.1.parse::<usize>() {
                    Ok(n) => n,
                    Err(_) => {
                        info!("Unexpected Content-Length");
                        0
                    }
                };
                break;
            }
        }
        http_body.reserve(exp_len);
        unsafe {
            http_body.set_len(exp_len);
        }
        if let Err(e) = buf_reader.read_exact(&mut http_body) {
            warn!("{}", e);
        }
    }

    return Ok(HttpRequest {
        method: method,
        path: path,
        version: version,
        fields: fields,
        body: Some(http_body),
    });
}


