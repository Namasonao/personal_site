use crate::parser::Future;
use crate::socket::Stream;
use std::fmt::Display;
use std::io::{Error, Write};
use std::os::fd::{AsFd, BorrowedFd};

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

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
        }
    }
}

impl Display for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{} {} {}", self.method, self.path, self.version)?;
        if let Some(b) = &self.body {
            write!(f, "\t{} bytes", b.len())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum StatusCode {
    OK,
    BadRequest,
    Unauthorized,
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

pub struct Responder {
    bytes: Vec<u8>,
    sent: usize,
    stream: Stream,
}

impl Responder {
    pub fn as_fd(&self) -> BorrowedFd {
        return self.stream.as_fd();
    }
    pub fn from_http_response(r: HttpResponse, stream: Stream) -> Responder {
        let mut response = r.version;
        response += " ";
        use StatusCode::*;
        response += match r.status_code {
            OK => "200 OK",
            NotFound => "404 NOT FOUND",
            NotImplemented => "501 NOT IMPLEMENTED",
            BadRequest => "400 BAD REQUEST",
            Unauthorized => "401 UNAUTHORIZED",
            InternalError => "500 INTERNAL SERVER ERROR",
        };
        response += "\r\n";

        for (left, right) in r.fields.into_iter() {
            response += &left;
            response += ": ";
            response += &right;
            response += "\r\n";
        }
        response += "\r\n";

        let mut bytes = response.into_bytes();
        if let Some(body) = r.body {
            bytes.extend(body);
        }
        Responder {
            bytes,
            sent: 0,
            stream,
        }
    }

    pub fn respond(&mut self) -> Future<()> {
        let n = match self.stream.write(&self.bytes[self.sent..]) {
            Ok(n) => n,
            Err(e) => return Future::Fail("there was an error during writing"),
        };
        self.sent += n;
        if self.sent == self.bytes.len() {
            return Future::Done(());
        }
        Future::Wait
    }
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

    /*
    pub fn respond(&self, stream: &mut Stream) -> Result<(), Error> {
        let mut response = self.version.clone();
        response += " ";
        use StatusCode::*;
        response += match &self.status_code {
            OK => "200 OK",
            NotFound => "404 NOT FOUND",
            NotImplemented => "501 NOT IMPLEMENTED",
            BadRequest => "400 BAD REQUEST",
            Unauthorized => "401 UNAUTHORIZED",
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
    */
}
