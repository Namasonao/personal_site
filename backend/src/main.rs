mod api;
mod config;
mod http;
mod my_logger;
mod note_db;
mod socket;
mod sqlite_db;
mod authenticator;
use crate::api::ApiHandler;
use crate::config::*;
use crate::http::server::*;
use crate::http::types::*;
use crate::my_logger::*;
use std::env;
use std::fs;

struct MyHandler<'a> {
    config: &'a Config,
}

impl<'a> HttpHandler for MyHandler<'a> {
    fn handle(&self, request: HttpRequest) -> HttpResponse {
        if request.path.starts_with("/api") {
            let handler = ApiHandler {};
            return handler.handle(request);
        }

        match request.method {
            Method::Get => {}
            Method::Post => {}
        }

        let path_bytes = request.path.as_bytes();
        if path_bytes[path_bytes.len() - 1] == b'/' {
            return http_respond_file(
                &(self.config.frontend_dir.clone() + &request.path + "index.html"),
            );
        } else {
            return http_respond_file(&(self.config.frontend_dir.clone() + &request.path));
        }
    }
}

fn http_respond_file(fp: &str) -> HttpResponse {
    // TODO: Check permissions
    let result = fs::read(fp);
    let contents = match result {
        Ok(c) => c,
        Err(e) => {
            warn!("fs::read({}) - {}", fp, e);
            return HttpResponse::new(StatusCode::NotFound, None);
        }
    };

    return HttpResponse::new(StatusCode::OK, Some(contents));
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
        Err(e) => panic!("Config error: {}", e),
    };
    my_logger::init();
    note_db::init(&cfg.database);
    println!("{:#?}", cfg);

    let http_handler = MyHandler { config: &cfg };
    let http_server = match HttpServer::new(&cfg, Box::new(http_handler)) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };

    http_server.listen();
}
