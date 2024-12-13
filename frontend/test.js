const notes = document.getElementById("note-structure");
console.log(notes);

const myElement = document.createElement("hi");
myElement.innerText = "HAHAHA this is dynamic";
notes.append(myElement);
for (const element of notes.children) {
	console.log(element);
}
