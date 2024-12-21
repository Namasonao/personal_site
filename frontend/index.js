const notes = document.getElementById("note-structure");
const textbox = document.getElementById("add-note-input");

async function onSubmitNotePress(data) {
	console.log(textbox);
	const text = textbox.value;
	if (!text) {
		return;
	}
	const note = {};
	note.text = text;
	const domNote = addNoteToDom(note);
	const response = await fetch("/api/add-note", {
		method: "POST",
		body: JSON.stringify({
			note: text,
		}),
		headers: {},
	});
	const id = await response.json();
	domNote.apiNote.id = id;
	
}

async function onDeleteNotePress(data, note) {
	const nId = note.apiNote.id;
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

function noteHeader(root) {
	const deleteButton = document.createElement("input");
	deleteButton.type = "button";
	deleteButton.classList.add("delete-button");
	deleteButton.addEventListener("click", (data) => {
		onDeleteNotePress(data, root)
	});

	const time = document.createElement("div");
	time.innerText = "12:57:18";

	const header = document.createElement("div");
	header.appendChild(time);
	header.appendChild(deleteButton);
	header.classList.add("note-header");

	return header;
}

function addNoteToDom(note) {
	const root = document.createElement("div");
	root.apiNote = note;

	const noteText = document.createElement("div");
	noteText.innerText = note.text;
	noteText.classList.add("note-text");

	const header = noteHeader(root);

	root.classList.add("block-div");
	root.appendChild(header);
	root.appendChild(noteText);

	notes.insertBefore(root, notes.children[1]);

	return root;
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
