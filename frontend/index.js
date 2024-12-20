const notes = document.getElementById("note-structure");
const textbox = document.getElementById("add-note-input");

async function addNoteDb(text) {
	fetch("/api/add-note", {
		method: "POST",
		body: JSON.stringify({
			note: text,
		}),
		headers: {},
	});
}

async function onSubmitNotePress(data) {
	console.log(textbox);
	const text = textbox.value;
	if (!text) {
		return;
	}
	const note = {};
	note.text = text;
	addNoteToDom(note);
	addNoteDb(text);
}

async function onDeleteNotePress(data) {
	const note = data.srcElement.parentElement;
	const nId = note.apiId;
	console.log(nId);
	const response = await fetch("/api/delete-note", {
		method: "POST",
		body: JSON.stringify({
			id: nId,
		}),
		headers: {},
	});
	console.log("Delete response:");
	console.log(response);
	if (response.status === 200) {
		note.remove();
	}
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

function addNoteToDom(note) {
	const deleteButton = document.createElement("input");
	deleteButton.type = "button";
	deleteButton.addEventListener("click", onDeleteNotePress);

	const newNote = document.createElement("div");
	newNote.innerText = note.text;
	newNote.apiId = note.id;
	newNote.appendChild(deleteButton);
	notes.insertBefore(newNote, notes.children[1]);
}

async function renderNotes() {
	const notes_json = await getNotesFromDb();
	console.log("notes json:");
	console.log(notes_json);
	for (let i = 0; i < notes_json.length; i++) {
		addNoteToDom(notes_json[i]);
	}
}

const send = document.querySelector("#add-note-submit");
send.addEventListener("click", onSubmitNotePress);
renderNotes();
