const notes = document.getElementById("note-structure");
const textbox = document.getElementById("add-note-input");
const creation_username = document.getElementById("create-account-name");
let passkey = get_passkey();

function get_passkey() {
   try {
      const acc = JSON.parse(localStorage.account);
      return acc.passkey;
   } catch (e) {
      return "";
   }
}

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
		headers: {
         passkey: passkey,
      },
	});
   if (response.status !== 200) {


      domNote.children[0].innerText = "FAILED: " + response.status;
      const deleteButton = document.createElement("input");
      deleteButton.type = "button";
      deleteButton.classList.add("delete-button");
      deleteButton.addEventListener("click", (data) => {
         domNote.remove();
      });
      domNote.children[0].appendChild(deleteButton);
      return;
   }
	const apiNote = await response.json();
	domNote.remove();
	addNoteToDom(apiNote);
	console.log('Received response, updating note');
	console.log(apiNote);
}

async function onCreateAccountPress(data) {
   console.log(creation_username);
   const username = creation_username.value;
   if (!username) {
      return;
   }

   const response = await fetch("/api/create-account", {
      method: "POST",
      body: JSON.stringify({
         name: username,
      }),
      headers: {
      },
   });
   console.log(response);
   const login_info = await response.json();
   console.log(login_info);
   localStorage.setItem("account", JSON.stringify({
      name: username,
      passkey: login_info.passkey,
   }));
}

async function onDeleteNotePress(data, root) {
   if (passkey === "") {
      return;
   }
	const nId = root.apiNote.id;
	console.log(nId);
	const response = await fetch("/api/delete-note", {
		method: "POST",
		body: JSON.stringify({
			id: nId,
		}),
		headers: {
         passkey: passkey,
      },
	});
	console.log("Delete response:");
	console.log(response);
	if (response.status === 200) {
		root.remove();
	}
}

async function getNotesFromDb() {
   if (passkey === "") {
      return {};
   }
	const response = await fetch("/api/get-notes", {
		method: "GET",
      headers: {
         passkey: passkey,
      },
	});
	if (!response.ok) {
		return null;
	}
	console.log(response.body);
	return await response.json();
}

function noteHeader(root) {
	console.log(root.apiNote);
	if (root.apiNote.id === undefined) {
      const header = document.createElement("div");
      header.innerText = "Submitting...";
      header.classList.add("note-header");
      return header;
   }
	const deleteButton = document.createElement("input");
	deleteButton.type = "button";
	deleteButton.classList.add("delete-button");
	deleteButton.addEventListener("click", (data) => {
		onDeleteNotePress(data, root)
	});

	const dateDiv = document.createElement("div");
	const timeDiv = document.createElement("div");
	const fullDate = new Date(root.apiNote.date);
	const date = fullDate.toLocaleDateString("en-UK");
	const hh = fullDate.getHours();
	const mm = fullDate.getMinutes();
	const ss = fullDate.getSeconds();

	time = hh + ":" + mm + ":" + ss;
	dateDiv.innerText = date + "\t" + time;

	const header = document.createElement("div");
	header.appendChild(dateDiv);
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
	root.classList.add("whole-note");
	root.appendChild(header);
	root.appendChild(noteText);

	notes.insertBefore(root, notes.children[1]);

	return root;
}

async function checkLogin() {
	const response = await fetch("/api/who-am-i", {
		method: "GET",
      headers: {
         passkey: passkey,
      },
	});
	const body = await response.json();
   const login_div = document.getElementById("login-div");
   if (body.authenticated !== true) {
      login_div.children[0].hidden = false;
   } else {
      const welcome_div = document.createElement("div");
      welcome_div.innerText = "Welcome, " + body.username + "!";
      login_div.appendChild(welcome_div);
   }
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
const account_create = document.querySelector("#create-account-submit");
account_create.addEventListener("click", onCreateAccountPress);
checkLogin();
renderNotes();
console.log(localStorage.account);
