use crate::{info, warn, config::Config};
use std::io::{BufReader, Error};
use std::net::{TcpListener};
use crate::http::types::{HttpResponse, HttpRequest};
use crate::http::internal::*;

type HttpHandlerT<'a> = Box<dyn HttpHandler + 'a>;
pub trait HttpHandler {
    fn handle(&self, request: HttpRequest) -> HttpResponse;
}

pub struct HttpServer<'a> {
    listener: TcpListener,
    default_handler: HttpHandlerT<'a>,
    config: &'a Config,
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

