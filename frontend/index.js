const notes = document.getElementById("note-structure");
console.log(notes);

for (const element of notes.children) {
	console.log(element);
}

async function onPress(data) {
	const textbox = document.getElementById("add-note-input");
	console.log(textbox);
	const text = textbox.value;
	if (!text) {
		return;
	}
	console.log(text);
	const newTodo = document.createElement("div");
	newTodo.innerText = text;
	notes.insertBefore(newTodo, notes.children[1]);
}

const send = document.querySelector("#add-note-submit");
send.addEventListener("click", onPress);
