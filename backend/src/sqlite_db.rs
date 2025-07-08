use crate::note_db::{Note, NoteDB, NoteEntry, NoteId};
use base64::{prelude::BASE64_STANDARD, Engine};
use sqlite::{open, Connection, State, Statement};
use std::path::Path;

pub enum DBError {
    Fail,
}
pub struct SqliteDB {
    connection: Option<Connection>,
}

impl SqliteDB {
    pub const fn new() -> SqliteDB {
        SqliteDB { connection: None }
    }
    pub fn init(&mut self, path: &str) -> Result<(), DBError> {
        self.connection = match open(Path::new(path)) {
            Ok(c) => Some(c),
            Err(_) => return Err(DBError::Fail),
        };

        Ok(())
    }
}

fn statement_to_entry(statement: &Statement<'_>) -> Option<NoteEntry> {
    match statement_to_entry_err(statement) {
        Ok(e) => Some(e),
        Err(_) => None,
    }
}

fn statement_to_entry_err(statement: &Statement<'_>) -> Result<NoteEntry, sqlite::Error> {
    let id: i64 = statement.read::<i64, _>("id")?;
    let author: i64 = statement.read::<i64, _>("author")?;
    let time: i64 = statement.read::<i64, _>("time")?;
    let contents = from_sql_string(&statement.read::<String, _>("contents")?);
    let entry = NoteEntry {
        id: id,
        note: Note {
            text: contents,
            date: time,
            author,
        },
    };
    Ok(entry)
}

fn into_sql_string(string: &str) -> String {
    return "'".to_string() + &BASE64_STANDARD.encode(string) + "'";
}

fn from_sql_string(string: &str) -> String {
    return String::from_utf8(BASE64_STANDARD.decode(string).unwrap()).unwrap();
}

impl NoteDB for SqliteDB {
    fn save(&mut self, n: &Note) -> NoteId {
        let connection = match &self.connection {
            Some(c) => c,
            None => return -1,
        };

        let author = n.author;
        let time = &n.date;
        let contents = into_sql_string(&n.text);
        let query = format!(
            "
        INSERT INTO notes (author, time, contents) 
        VALUES ({}, {}, {})
        RETURNING id
        ",
            author, time, contents
        );

        //println!("{}", query);
        let mut statement = connection.prepare(query).unwrap();
        if let Ok(State::Row) = statement.next() {
            let id: i64 = statement.read::<i64, _>("id").unwrap();
            return id;
        }
        return 0;
    }

    fn get(&self, id: &NoteId) -> Option<NoteEntry> {
        let connection = match &self.connection {
            Some(c) => c,
            None => return None,
        };

        let query = format!(
            "
        SELECT * FROM notes WHERE id IS {}
        ",
            id
        );
        let mut statement = connection.prepare(query).unwrap();
        if let Ok(State::Row) = statement.next() {
            statement_to_entry(&statement)
        } else {
            None
        }
    }

    fn delete(&mut self, id: &NoteId) {
        let connection = match &self.connection {
            Some(c) => c,
            None => return,
        };
        let query = format!(
            "
        DELETE FROM notes WHERE id IS {}
        ",
            id
        );

        connection.execute(query).unwrap();
    }

    fn delete_if_user(&mut self, id: &NoteId, passkey: i64) {
        let connection = match &self.connection {
            Some(c) => c,
            None => return,
        };
        let query = format!(
            "
        DELETE FROM notes WHERE id={} AND author={}
        ",
            id, passkey
        );

        connection.execute(query).unwrap();
    }

    fn all(&self) -> Vec<NoteEntry> {
        let connection = match &self.connection {
            Some(c) => c,
            None => return Vec::new(),
        };
        let query = "SELECT * FROM notes";
        let mut statement = connection.prepare(query).unwrap();
        //statement.bind((1)).unwrap();
        let mut entries: Vec<NoteEntry> = Vec::new();
        while let Ok(State::Row) = statement.next() {
            if let Some(entry) = statement_to_entry(&statement) {
                entries.push(entry);
            }
        }

        entries
    }

    fn by_passkey(&self, passkey: i64) -> Vec<NoteEntry> {
        let connection = match &self.connection {
            Some(c) => c,
            None => panic!("no connection"),
        };
        let query = format!("SELECT * FROM notes WHERE author={}", passkey);

        let mut statement = connection.prepare(query).unwrap();
        let mut entries: Vec<NoteEntry> = Vec::new();
        while let Ok(State::Row) = statement.next() {
            if let Some(entry) = statement_to_entry(&statement) {
                entries.push(entry);
            }
        }
        entries
    }

    fn create_user(&mut self, name: &str, time: i64, passkey: i64) {
        let connection = match &self.connection {
            Some(c) => c,
            None => panic!("no connection"),
        };

        let name = into_sql_string(name);
        let query = format!(
            "
        INSERT INTO users (passkey, name, time) 
        VALUES ({}, {}, {})
        ",
            passkey, name, time
        );

        connection.execute(query).unwrap();
    }

    fn get_user_by_passkey(&mut self, passkey: i64) -> Option<String> {
        let connection = match &self.connection {
            Some(c) => c,
            None => panic!("no connection"),
        };

        let query = format!(
            "
            SELECT COUNT(*),name FROM users WHERE passkey={}
            ",
            passkey
        );
        let mut statement = connection.prepare(query).unwrap();
        while let Ok(State::Row) = statement.next() {
            let count: i64 = statement.read::<i64, _>(0).unwrap();
            let name = from_sql_string(&statement.read::<String, _>(1).unwrap());
            if count == 1 {
                return Some(name);
            }
        }
        None
    }
}
