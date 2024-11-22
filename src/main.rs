use std::time::{SystemTime, UNIX_EPOCH};

type NoteId = u64;
#[derive(Debug)]
struct Note {
    text: String,
    date: u64,
}

#[derive(Debug)]
struct NoteEntry {
    note: Note,
    id: NoteId,
}

fn save_note(n: Note) -> NoteEntry {
    println!("{:?}", n);
    let id = 0;
    return NoteEntry{note: n, id: id};
}

trait NoteDB {
    fn save(&mut self, n: Note) -> NoteEntry;
    fn get_by_id(&self, id: &NoteId) -> NoteEntry;
    fn edit(&mut self, id: &NoteId, n: Note) -> NoteEntry;
}

fn main() {
    let my_note = Note {
        text: "This is an example note.\n".to_string(),
        date: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64,
    };

    let entry = save_note(my_note);
    println!("{:?}", entry);
}
