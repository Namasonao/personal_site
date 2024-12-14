use crate::http::HttpHeader;
use crate::info;
use crate::TcpStream;
use std::io::Write;

type Response = String;
enum APIError {
    NotImplemented,
    NotFound,
}

fn api_add_note() -> Result<Response, APIError> {
    info!("Request to add note");
    return Err(APIError::NotImplemented);
}
fn hello_world() -> Result<Response, APIError> {
    return Ok("Hello world!".to_string());
}

fn respond_error(mut stream: TcpStream, e: APIError) {
    use APIError::*;
    match e {
        NotFound => {
            let response = "HTTP/1.1 404 NOT FOUND\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
        NotImplemented => {
            let response = "HTTP/1.1 501 NOT IMPLEMENTED\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    };
}

pub fn handle_api(mut stream: TcpStream, header: HttpHeader) {
    let path: Vec<&str> = header.path.split('/').collect();

    let resp_res = match path[2] {
        "add-note" => api_add_note(),
        "hello" => hello_world(),
        _ => Err(APIError::NotFound),
    };

    let response = match resp_res {
        Ok(s) => s,
        Err(e) => {
            respond_error(stream, e);
            return;
        }
    };

    stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
    stream.write_all(response.as_bytes()).unwrap();
}
