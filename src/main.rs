use std::time::{SystemTime, UNIX_EPOCH};

type NoteId = u64;
#[derive(Debug, PartialEq, Clone)]
struct Note {
    text: String,
    date: u64,
}

impl Note {
    fn new_str(s: &str) -> Note {
        Note {
            text: s.to_string(),
            date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64,
        }
    }
    fn new(s: String) -> Note {
        Note {
            text: s,
            date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct NoteEntry {
    note: Note,
    id: NoteId,
}

trait NoteDB {
    fn save(&mut self, n: &Note) -> NoteId;
    fn get(&self, id: &NoteId) -> Option<NoteEntry>;
    fn edit(&mut self, id: &NoteId, n: &Note);
    fn iter(&self) -> Vec<NoteEntry>;
}

struct NoDB {
    entries: Vec<NoteEntry>,
    next_id: NoteId,
}
impl NoteDB for NoDB {
    fn save(&mut self, n: &Note) -> NoteId {
        let id = self.next_id;
        let entry = NoteEntry {
            note: n.clone(),
            id: id,
        };
        self.next_id += 1;
        self.entries.push(entry.clone());
        id
    }
    fn get(&self, id: &NoteId) -> Option<NoteEntry> {
        for entry in self.entries.iter() {
            if entry.id == *id {
                return Some(entry.clone());
            }
        }
        return None;
    }
    fn edit(&mut self, id: &NoteId, n: &Note) {
        for entry in self.entries.iter_mut() {
            if entry.id == *id {
                entry.note = n.clone();
                return;
            }
        }
    }

    fn iter(&self) -> Vec<NoteEntry> {
        return self.entries.clone();
    }
}

fn main() {
    println!("Hello");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goodtest() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn test_nodb_saveget() {
        let db = NoDB {
            entries: Vec::new(),
            next_id: 10,
        };
        test_note_db_saveget(Box::new(db))
    }

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

    #[test]
    fn test_nodb_iter() {
        let db = NoDB {
            entries: Vec::new(),
            next_id: 10,
        };
        test_note_db_iter(Box::new(db));
    }

    fn test_note_db_iter(mut db: Box<dyn NoteDB>) {
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

    fn test_note_db_saveget(mut db: Box<dyn NoteDB>) {
        let my_note1 = Note {
            text: "This is an example note.\n".to_string(),
            date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64,
        };
        let my_note2 = Note {
            text: "This note stores some other random info".to_string(),
            date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64,
        };

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
}
