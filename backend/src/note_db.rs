use crate::nodb::NoDB;
use std::time::{SystemTime, UNIX_EPOCH};

pub type NoteId = u64;

static mut DATABASE: NoDB = NoDB::new();

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
    pub text: String,
    pub date: u64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NoteEntry {
    pub note: Note,
    pub id: NoteId,
}

impl Note {
    pub fn new_str(s: &str) -> Note {
        Note {
            text: s.to_string(),
            date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64,
        }
    }
    pub fn new(s: String) -> Note {
        Note {
            text: s,
            date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64,
        }
    }
}

pub trait NoteDB {
    fn save(&mut self, n: &Note) -> NoteId;
    fn get(&self, id: &NoteId) -> Option<NoteEntry>;
    fn delete(&mut self, id: &NoteId);
    fn iter(&self) -> Vec<NoteEntry>;
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

pub fn iter() -> Vec<NoteEntry> {
    unsafe { DATABASE.iter() }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn populate_db(db: &mut Box<dyn NoteDB>) -> i32 {
        let notes = vec![
            Note::new_str("Nothing!"),
            Note::new_str("This is an example note."),
            Note::new_str("In this note, there are\nMULTIPLE\nlines!"),
            Note::new_str("Even more interesting text."),
            Note::new_str("Even more interesting text."),
        ];

        let mut i = 0;
        for n in notes.into_iter() {
            db.save(&n);
            i += 1;
        }

        i
    }

    pub fn test_note_db_iter(mut db: Box<dyn NoteDB>) {
        let l = populate_db(&mut db);
        let mut seen: Vec<NoteEntry> = Vec::new();
        let mut i = 0;
        for entry in db.iter().iter() {
            i += 1;
            for s in seen.iter() {
                assert_ne!(entry, s);
            }
            seen.push(entry.clone());
        }
        assert_eq!(i, l);
    }

    pub fn test_note_db_saveget(mut db: Box<dyn NoteDB>) {
        let my_note1 = Note::new_str("This is an example note.\n");
        let my_note2 = Note::new_str("This note stores some other random info");

        let id1 = db.save(&my_note1);
        let id2 = db.save(&my_note2);
        assert_ne!(id1, id2);

        let get1 = if let Some(get) = db.get(&id1) {
            get
        } else {
            panic!("get1 not found");
        };
        let get2 = if let Some(get) = db.get(&id2) {
            get
        } else {
            panic!("get2 not found");
        };
        let get3 = if let Some(get) = db.get(&id1) {
            get
        } else {
            panic!("get3 not found")
        };
        assert_eq!(get1.note, my_note1);
        assert_eq!(get2.note, my_note2);
        assert_eq!(get3.note, my_note1);
    }

    pub fn test_note_db_delete(mut db: Box<dyn NoteDB>) {
        let my_note = Note::new_str("This is an example note.\n");
        let id1 = db.save(&my_note);
        db.delete(&id1);
        if let Some(get) = db.get(&id1) {
            panic!("not deleted");
        }
    }
}
