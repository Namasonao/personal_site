use crate::http::server::HttpHandler;
use crate::http::types::{HttpRequest, HttpResponse, Method, StatusCode};
use crate::note_db::{self, Note, NoteId};
use crate::{info, warn};
use base64::{prelude::BASE64_STANDARD, Engine};
use getrandom;
use serde_json::{self, json};
use std::hash::{self, Hash, Hasher};

pub struct ApiHandler {}
impl HttpHandler for ApiHandler {
    fn handle(&self, req: HttpRequest) -> HttpResponse {
        return handle_api(req);
    }
}

fn bad_request() -> HttpResponse {
    HttpResponse::new(StatusCode::BadRequest, None)
}

fn get_string(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn get_id(v: &serde_json::Value) -> Option<NoteId> {
    let n = match v {
        serde_json::Value::Number(s) => s,
        _ => return None,
    };
    return n.as_i64();
}

fn api_add_note(request: HttpRequest) -> HttpResponse {
    info!("Request to add note");
    let body_bytes = match request.body {
        Some(b) => b,
        None => return bad_request(),
    };

    let body: serde_json::Value = match serde_json::from_slice(&body_bytes) {
        Ok(c) => c,
        Err(_) => return bad_request(),
    };
    let text = match get_string(&body["note"]) {
        Some(t) => t,
        None => return bad_request(),
    };
    let id = note_db::save(&Note::new(text.clone()));

    info!("Stored note {}", id);

    let entry = match note_db::get(&id) {
        Some(e) => e,
        None => return HttpResponse::new(StatusCode::InternalError, None),
    };

    let note = match stringify_note(entry) {
        Some(s) => s,
        None => return HttpResponse::new(StatusCode::InternalError, None),
    };
    HttpResponse::new(StatusCode::OK, Some(note.into_bytes()))
}

fn stringify_note(entry: note_db::NoteEntry) -> Option<String> {
    let note_json = json!({
        "text": entry.note.text,
        "id": entry.id,
        "date": entry.note.date,
    });
    match serde_json::to_string(&note_json) {
        Ok(n) => Some(n),
        Err(e) => {
            warn!("{}", e);
            None
        }
    }
}

fn api_get_notes(request: HttpRequest) -> HttpResponse {
    if request.method != Method::Get {
        return bad_request();
    }
    info!("Request to get notes");
    let note_entries = note_db::all();
    let mut resp = "[".to_string();
    for entry in note_entries.into_iter() {
        let note = match stringify_note(entry) {
            Some(s) => s,
            None => return HttpResponse::new(StatusCode::InternalError, None),
        };
        resp += &note;
        resp += ",";
    }
    if resp.len() > 1 {
        resp.pop();
    }
    resp += "]";

    HttpResponse::new(StatusCode::OK, Some(resp.into_bytes()))
}

fn api_delete_note(request: HttpRequest) -> HttpResponse {
    info!("Request to delete notes");
    let body_bytes = match request.body {
        Some(b) => b,
        None => return bad_request(),
    };

    let body: serde_json::Value = match serde_json::from_slice(&body_bytes) {
        Ok(c) => c,
        Err(_) => return bad_request(),
    };

    let id = match get_id(&body["id"]) {
        Some(s) => s,
        None => return bad_request(),
    };
    note_db::delete(&id);

    HttpResponse::new(StatusCode::OK, None)
}

fn api_create_account(request: HttpRequest) -> HttpResponse {
    if request.method != Method::Post {
        return bad_request();
    }
    info!("Request to create user");
    let body_bytes = match request.body {
        Some(b) => b,
        None => return bad_request(),
    };
    let body: serde_json::Value = match serde_json::from_slice(&body_bytes) {
        Ok(c) => c,
        Err(_) => return bad_request(),
    };
    let name = match get_string(&body["name"]) {
        Some(s) => s,
        None => return bad_request(),
    };
    let (passkey, hash) = generate_passkey();
    let time = note_db::now();
    let id = note_db::create_user(&name, time, hash);

    let result = json!({
        "id": id,
        "passkey": &BASE64_STANDARD.encode(passkey),
    });
    let body = match serde_json::to_string(&result) {
        Ok(n) => Some(n.into_bytes()),
        Err(e) => {
            warn!("{}", e);
            None
        }
    };
    HttpResponse::new(StatusCode::OK, body)
}

fn generate_passkey() -> (Vec<u8>, i64) {
    let mut vec = Vec::new();
    vec.resize(64, 0);
    getrandom::fill(vec.as_mut_slice());

    let mut s = hash::DefaultHasher::new();
    vec.hash(&mut s);
    let hash = s.finish() as i64;
    (vec, hash)
}

fn hello_world() -> HttpResponse {
    HttpResponse::new(StatusCode::OK, Some("hello world!".as_bytes().to_vec()))
}

fn handle_api(request: HttpRequest) -> HttpResponse {
    let path: Vec<&str> = request.path.split('/').collect();

    return match path[2] {
        "add-note" => api_add_note(request),
        "get-notes" => api_get_notes(request),
        "delete-note" => api_delete_note(request),
        "create-account" => api_create_account(request),
        "hello" => hello_world(),
        "not-implemented" => HttpResponse::new(StatusCode::NotImplemented, None),
        _ => HttpResponse::new(StatusCode::NotFound, None),
    };
}
