use std::time::{SystemTime, UNIX_EPOCH};

type NoteId = u64;
#[derive(Debug, PartialEq)]
struct Note {
    text: String,
    date: u64,
}

#[derive(Debug, PartialEq)]
struct NoteEntry {
    note: Note,
    id: NoteId,
}


trait NoteDB {
    fn save(&mut self, n: Note) -> NoteEntry;
    fn get(&self, id: &NoteId) -> Option<NoteEntry>;
    fn edit(&mut self, id: &NoteId, n: Note) -> NoteEntry;
    fn iter(&self) -> dyn Iterator<Item=NoteEntry>;
}

fn main() {
    println!("Hello");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goodtest() {
        assert_eq!(1+1,2);
    }

    fn test_note_db(mut db: Box<dyn NoteDB>) {
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

        let entry1 = db.save(my_note1);
        let entry2 = db.save(my_note2);
        assert_ne!(entry1.id, entry2.id);

        
        let get1 = if let Some(get) = db.get(&entry1.id) {get} else {panic!("get1 not found");};
        let get2 = if let Some(get) = db.get(&entry2.id) {get} else {panic!("get2 not found");};
        let get3 = if let Some(get) = db.get(&entry1.id) {get} else {panic!("get3 not found")};
        assert_eq!(get1, entry1);
        assert_eq!(get2, entry2);
        assert_eq!(get3, entry1);
    }
}
