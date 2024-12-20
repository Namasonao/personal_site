const notes = document.getElementById("note-structure");

async function addNoteDb(text) {
	fetch("/api/add-note", {
		method: "POST",
		body: JSON.stringify({
			note: text,
		}),
		headers: {},
	});
}

async function onPress(data) {
	const textbox = document.getElementById("add-note-input");
	console.log(textbox);
	const text = textbox.value;
	if (!text) {
		return;
	}
	const newTodo = document.createElement("div");
	newTodo.innerText = text;
	notes.insertBefore(newTodo, notes.children[1]);
	addNoteDb(text);
}

async function getNotesFromDb() {
	const response = await fetch("/api/get-notes", {
		method: "GET",
	});
	if (!response.ok) {
		return null;
	}
	console.log(response.body);
	return await response.json();
}

async function renderNotes() {
	const notes_json = await getNotesFromDb();
	console.log("notes json:");
	console.log(notes_json);
	for (let i = 0; i < notes_json.length; i++) {
		const note = notes_json[i];
		const newNote = document.createElement("div");
		newNote.innerText = note.text;
		notes.insertBefore(newNote, notes.children[1]);

	}
}

const send = document.querySelector("#add-note-submit");
send.addEventListener("click", onPress);
renderNotes();
