//use crate::nodb::NoDB;
use crate::my_logger::warn;
use crate::sqlite_db::SqliteDB;
use std::time::{SystemTime, UNIX_EPOCH};

pub type NoteId = i64;
pub type UserId = i64;

static mut DATABASE: SqliteDB = SqliteDB::new();

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
    pub text: String,
    pub date: i64,
    pub author: i64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NoteEntry {
    pub note: Note,
    pub id: NoteId,
}

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

impl Note {
    pub fn new_str(s: &str) -> Note {
        Note {
            text: s.to_string(),
            date: now(),
            author: 0,
        }
    }
    pub fn new(s: String, author: i64) -> Note {
        Note {
            text: s,
            date: now(),
            author: author,
        }
    }
}

pub trait NoteDB {
    fn save(&mut self, n: &Note) -> NoteId;
    fn get(&self, id: &NoteId) -> Option<NoteEntry>;
    fn delete(&mut self, id: &NoteId);
    fn delete_if_user(&mut self, id: &NoteId, passkey: i64);
    fn all(&self) -> Vec<NoteEntry>;
    fn by_passkey(&self, passkey: i64) -> Vec<NoteEntry>;

    fn create_user(&mut self, name: &str, time: i64, passkey: i64);
    fn get_user_by_passkey(&mut self, passkey: i64) -> Option<String>;
}

pub fn save(n: &Note) -> NoteId {
    unsafe { DATABASE.save(n) }
}

pub fn get(id: &NoteId) -> Option<NoteEntry> {
    unsafe { DATABASE.get(id) }
}

pub fn delete(id: &NoteId) {
    unsafe { DATABASE.delete(id) }
}

pub fn delete_if_user(id: &NoteId, passkey: i64) {
    unsafe { DATABASE.delete_if_user(id, passkey) }
}

pub fn all() -> Vec<NoteEntry> {
    unsafe { DATABASE.all() }
}

pub fn by_passkey(passkey: i64) -> Vec<NoteEntry> {
    unsafe { DATABASE.by_passkey(passkey) }
}

pub fn init(path: &str) {
    if let Err(_) = unsafe { DATABASE.init(path) } {
        warn!("Could not initialise database {}", path);
    }
}

pub fn create_user(name: &str, time: i64, passkey: i64) {
    unsafe { DATABASE.create_user(name, time, passkey) }
}

pub fn get_user_by_passkey(passkey: i64) -> Option<String> {
    unsafe { DATABASE.get_user_by_passkey(passkey) }
}
