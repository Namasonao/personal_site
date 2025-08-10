const notes = document.getElementById("note-structure");
const textbox = document.getElementById("add-note-input");
const creation_username = document.getElementById("create-account-name");
let accInfo = getAccInfo();

function getAccInfo() {
   try {
      const acc = JSON.parse(localStorage.account);
      return acc;
   } catch (e) {
      return null;
   }
}

function setAccInfo(name, passkey) {
   localStorage.setItem("account", JSON.stringify({
      name: name,
      passkey: passkey,
   }));
   accInfo = getAccInfo();
   showLoggedIn();
}

async function onSubmitNotePress(data) {
	const text = textbox.value;
	if (!text) {
		return;
	}
   if (!accInfo) {
      console.log("Login before you add notes!");
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
         passkey: accInfo.passkey,
      },
	});
   if (response.status !== 200) {
      domNote.children[0].innerText = "FAILED: " + response.status;
      const deleteButton = document.createElement("button");
      deleteButton.classList.add("delete-button");
      deleteButton.addEventListener("click", (data) => {
         domNote.remove();
      });
      const icon = document.createElement("i");
      domNote.children[0].appendChild(deleteButton);
      return;
   }
   textbox.value = "";
	const apiNote = await response.json();
	domNote.remove();
	addNoteToDom(apiNote);
	console.log('Received response, updating note');
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
   if (response.status !== 200) {
      console.log("Invalid USERNAME");
      return;
   }
   const login_info = await response.json();
   setAccInfo(username, login_info.passkey);
}

async function onDeleteNotePress(data, root) {
   if (!accInfo.passkey) {
      return;
   }
	const nId = root.apiNote.id;
	const response = await fetch("/api/delete-note", {
		method: "POST",
		body: JSON.stringify({
			id: nId,
		}),
		headers: {
         passkey: accInfo.passkey,
      },
	});
	console.log("Delete response:");
	console.log(response);
	if (response.status === 200) {
		root.remove();
	}
}

async function getNotesFromDb() {
   if (!accInfo) {
      return {};
   }
	const response = await fetch("/api/get-notes", {
		method: "GET",
      headers: {
         passkey: accInfo.passkey,
      },
	});
	if (!response.ok) {
		return null;
	}
	return await response.json();
}

function fmt2digit(n) {
   if (n < 10) {
      return "0" + n;
   }
   return n;
}

function noteHeader(root) {
	if (root.apiNote.id === undefined) {
      const header = document.createElement("div");
      header.innerText = "Submitting...";
      header.classList.add("note-header");
      return header;
   }
	const deleteButton = document.createElement("button");
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

	time = fmt2digit(hh) + ":" + fmt2digit(mm) + ":" + fmt2digit(ss);
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
   if (!accInfo) {
      showLoggedOut();
      return;
   }
	const response = await fetch("/api/who-am-i", {
		method: "GET",
      headers: {
         passkey: accInfo.passkey,
      },
	});
	const body = await response.json();
   if (body.authenticated !== true) {
      showLoggedOut();
   } else {
      showLoggedIn();
   }
}

const login_div = document.getElementById("login-div");
const username_box = document.getElementById("username-box");
function showLoggedOut() {
   login_div.children[0].hidden = false;
   login_div.children[1].hidden = true;
}

function showLoggedIn() {
   login_div.children[0].hidden = true;
   login_div.children[1].hidden = false;
   username_box.innerText = accInfo.name;
}

async function renderNotes() {
	const notes_json = await getNotesFromDb();
	for (let i = 0; i < notes_json.length; i++) {
		addNoteToDom(notes_json[i]);
	}
}

function copyPasskeyToClipboard() {
   navigator.clipboard.writeText(accInfo.passkey);
}

function logout() {
   console.log("Logged out of: `" + accInfo.passkey + "`"); 
   while (notes.children.length > 1) {
      notes.removeChild(notes.lastChild);
   }
   localStorage.removeItem("account");
   accInfo = null;
   showLoggedOut();
}

const loginKeyBox = document.getElementById("login-passkey-box");
async function login() {
   const passkey = loginKeyBox.value;
	const response = await fetch("/api/who-am-i", {
		method: "GET",
      headers: {
         passkey: passkey,
      },
	});
   if (response.status !== 200) {
      console.log("login failed! please retry");
      return;
   }

	const body = await response.json();
   if (body.authenticated !== true) {
      showLoggedOut();
   } else {
      setAccInfo(body.username, passkey);
      renderNotes();
   }
}

const send = document.querySelector("#add-note-submit");
send.addEventListener("click", onSubmitNotePress);
const account_create = document.querySelector("#create-account-submit");
account_create.addEventListener("click", onCreateAccountPress);
checkLogin();
renderNotes();
console.log(localStorage.account);
