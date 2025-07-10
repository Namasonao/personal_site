use http::types::{HttpRequest, HttpResponse, Method, StatusCode};
use crate::note_db::{self, UserId};
use crate::{info, warn};
use crate::base64;
use std::fmt;
use std::hash::{self, Hash, Hasher};

#[derive(Debug)]
pub enum AuthenticationError {
    MissingInformation,
    MalformedInformation,
    IncorrectPasskey,
}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn generate_passkey() -> (Vec<u8>, i64) {
    let mut vec = Vec::new();
    vec.resize(64, 0);
    match getrandom::fill(vec.as_mut_slice()) {
        Err(e) => panic!("source of randomness failed: {}", e),
        Ok(()) => {}
    };

    let hash = generate_hash(&vec);
    (vec, hash)
}

pub fn authenticate_request(req: &HttpRequest) -> Result<(i64, String), AuthenticationError> {
    info!("authenticating request...");
    let mut passkey64: Option<&str> = None;
    for (key, value) in &req.fields {
        if key == "passkey" {
            passkey64 = Some(value);
            break;
        }
    }
    let Some(passkey64) = passkey64 else {
        return Err(AuthenticationError::MissingInformation);
    };

    let passkey = match base64::decode(passkey64) {
        Ok(pk) => pk,
        Err(e) => {
            warn!("invalid base64 for passkey: {}", e);
            return Err(AuthenticationError::MalformedInformation);
        }
    };

    let hash = generate_hash(&passkey);

    let Some(name) = note_db::get_user_by_passkey(hash) else {
        return Err(AuthenticationError::IncorrectPasskey);
    };

    info!("{} authenticated as {}", hash, name);
    Ok((hash, name))
}

fn generate_hash(bytes: &[u8]) -> i64 {
    // This is not a secure hash function, but it is a quick and simple solution
    let mut s = hash::DefaultHasher::new();
    bytes.hash(&mut s);
    s.finish() as i64
}
