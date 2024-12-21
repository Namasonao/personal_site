use crate::http::{Http, Method};
use crate::info;
use crate::note_db::{self, Note, NoteId};
use crate::TcpStream;
use serde_json::{self, json};
use std::io::Write;

type Response = String;
enum APIError {
    NotImplemented,
    NotFound,
    BadRequest,
    InternalError,
}

fn get_string(v: &serde_json::Value) -> Result<String, APIError> {
    match v {
        serde_json::Value::String(s) => Ok(s.clone()),
        _ => Err(APIError::BadRequest),
    }
}

fn get_id(v: &serde_json::Value) -> Result<NoteId, APIError> {
    let n = match v {
        serde_json::Value::Number(s) => s,
        _ => return Err(APIError::BadRequest),
    };
    match n.as_u64() {
        Some(s) => Ok(s),
        _ => Err(APIError::BadRequest),
    }
}

fn api_add_note(request: Http) -> Result<Response, APIError> {
    info!("Request to add note");
    let body: serde_json::Value = match serde_json::from_slice(&request.body) {
        Ok(c) => c,
        Err(_) => return Err(APIError::BadRequest),
    };
    let text: String = get_string(&body["note"])?;
    let id = note_db::save(&Note::new(text.clone()));

    info!("Stored note {} with text:\n{}", id, text);

    return Ok(id.to_string());
}

fn api_get_notes(request: Http) -> Result<Response, APIError> {
    if request.header.method != Method::Get {
        return Err(APIError::BadRequest);
    }
    info!("Request to get notes");
    let note_entries = note_db::all();
    let mut resp = "[".to_string();
    for entry in note_entries.into_iter() {
        let note_json = json!({
            "text": entry.note.text,
            "id": entry.id,
            "date": entry.note.date,
        });
        let note = match serde_json::to_string(&note_json) {
            Ok(n) => n,
            Err(_) => return Err(APIError::InternalError),
        };
        resp += &note;
        resp += ",";
    }
    if resp.len() > 1 {
        resp.pop();
    }
    resp += "]";
    return Ok(resp);
}

fn api_delete_note(request: Http) -> Result<Response, APIError> {
    info!("Request to delete notes");

    let body: serde_json::Value = match serde_json::from_slice(&request.body) {
        Ok(c) => c,
        Err(_) => return Err(APIError::BadRequest),
    };

    let id = get_id(&body["id"])?;
    note_db::delete(&id);

    let resp = "".to_string();
    return Ok(resp);
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
        InternalError => "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n",
    };
    stream.write_all(response.as_bytes()).unwrap();
}

pub fn handle_api(mut stream: TcpStream, request: Http) {
    let path: Vec<&str> = request.header.path.split('/').collect();

    let resp_res = match path[2] {
        "add-note" => api_add_note(request),
        "get-notes" => api_get_notes(request),
        "delete-note" => api_delete_note(request),
        "hello" => hello_world(),
        "not-implemented" => Err(APIError::NotImplemented),
        _ => Err(APIError::NotFound),
    };

    let response = match resp_res {
        Ok(s) => s,
        Err(e) => {
            respond_error(stream, e);
            return;
        }
    };

    info!("{}", response);
    stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
    stream.write_all(response.as_bytes()).unwrap();
}
