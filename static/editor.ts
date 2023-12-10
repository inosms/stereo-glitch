import {basicSetup, EditorView} from "codemirror"
import { INITIAL_LEVEL } from "./script"

// The Markdown parser will dynamically load parsers
// for code blocks, using @codemirror/language-data to
// look up the appropriate dynamic import.
let view = new EditorView({
  doc: INITIAL_LEVEL,
  extensions: [
    basicSetup,
  ],
  parent: document.getElementById("editor")!,
});

// on pressing load button get the content of the editor and load it
document.getElementById("load-button")!.addEventListener("click", () => {
  const level = view.state.doc.toString();
  window.load_level(level);
});