use crate::note_db::{Note, NoteDB, NoteEntry, NoteId};

pub struct NoDB {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note_db::tests::*;

    #[test]
    fn test_nodb_saveget() {
        let db = NoDB {
            entries: Vec::new(),
            next_id: 10,
        };
        test_note_db_saveget(Box::new(db))
    }

    #[test]
    fn test_nodb_iter() {
        let db = NoDB {
            entries: Vec::new(),
            next_id: 10,
        };
        test_note_db_iter(Box::new(db));
    }
}
