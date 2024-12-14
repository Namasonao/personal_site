use crate::http::Http;
use crate::info;
use crate::note_db::{self, Note};
use crate::TcpStream;
use serde_json;
use std::io::Write;

type Response = String;
enum APIError {
    NotImplemented,
    NotFound,
    BadRequest,
}

fn get_string(v: &serde_json::Value) -> Result<String, APIError> {
    match v {
        serde_json::Value::String(s) => Ok(s.clone()),
        _ => Err(APIError::BadRequest),
    }
}

fn api_add_note(request: Http) -> Result<Response, APIError> {
    info!("Request to add note");
    let body: serde_json::Value = match serde_json::from_slice(&request.body) {
        Ok(c) => c,
        Err(e) => return Err(APIError::BadRequest),
    };
    let text: String = get_string(&body["note"])?;
    let id = note_db::save(&Note::new(text.clone()));

    info!("Stored note {} with text:\n{}", id, text);

    return Ok(id.to_string());
}
fn hello_world() -> Result<Response, APIError> {
    return Ok("Hello world!".to_string());
}

fn respond_error(mut stream: TcpStream, e: APIError) {
    use APIError::*;
    let response = match e {
        NotFound => "HTTP/1.1 404 NOT FOUND\r\n",
        NotImplemented => "HTTP/1.1 501 NOT IMPLEMENTED\r\n",
        BadRequest => "HTTP/1.1 400 BAD REQUEST\r\n",
    };
    stream.write_all(response.as_bytes()).unwrap();
}

pub fn handle_api(mut stream: TcpStream, request: Http) {
    let path: Vec<&str> = request.header.path.split('/').collect();

    let resp_res = match path[2] {
        "add-note" => api_add_note(request),
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
