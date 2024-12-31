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
}
