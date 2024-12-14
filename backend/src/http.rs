use crate::info;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::str;

fn parse_field(line: String) -> Option<Field> {
    let mut it = line.split(':');
    let key = it.next()?.to_string();
    let mut val = it.next()?.to_string();
    val.remove(0);
    Some((key, val))
}

pub fn parse_http(mut buf_reader: BufReader<&mut TcpStream>) -> Result<Http, &'static str> {
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

    let header = HttpHeader {
        method: method,
        path: path,
        version: version,
        fields: fields,
    };

    let mut http_body = Vec::new();
    if header.method == Method::Post {
        let mut exp_len = 0;
        for f in &header.fields {
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
        buf_reader.read_exact(&mut http_body);
    }

    // info!("Parsed http.\nHeader: {:#?}\nBody: {:#?}", header, str::from_utf8(&http_body));
    return Ok(Http {
        header: header,
        body: http_body,
    });
}

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
}

type Field = (String, String);

#[derive(Debug)]
pub struct HttpHeader {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub fields: Vec<Field>,
}

pub struct Http {
    pub header: HttpHeader,
    pub body: Vec<u8>,
}
