use std::io::{Error, Write};
use std::net::TcpStream;

pub type Field = (String, String);

#[derive(Debug, PartialEq, Clone)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub fields: Vec<Field>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum StatusCode {
    OK,
    BadRequest,
    NotFound,
    InternalError,
    NotImplemented,
}

#[derive(Debug)]
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

    pub fn respond(&self, stream: &mut TcpStream) -> Result<(), Error> {
        let mut response = self.version.clone();
        response += " ";
        use StatusCode::*;
        response += match &self.status_code {
            OK => "200 OK",
            NotFound => "404 NOT FOUND",
            NotImplemented => "501 NOT IMPLEMENTED",
            BadRequest => "400 BAD REQUEST",
            InternalError => "500 INTERNAL SERVER ERROR",
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
