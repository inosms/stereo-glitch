import nipplejs from 'nipplejs';
import init, { load_level, set_eye_distance, set_size, joystick_input, action_button_pressed, action_button_released, compress_level_to_url, decompress_level_from_url } from "../pkg/stereo_glitch.js";
import { basicSetup, EditorView } from "codemirror"

// export the functions 
export { load_level, set_eye_distance, compress_level_to_url, decompress_level_from_url };

// make the function available to the window
window.load_level = load_level;
window.set_eye_distance = set_eye_distance;
window.compress_level_to_url = compress_level_to_url;
window.decompress_level_from_url = decompress_level_from_url;

export const INITIAL_LEVEL = "N+W N+W N+W N+W N+W N+W\n" +
    "N+W N   N   N   N   N+W\n" +
    "N+W N   N+P N   N   N+W\n" +
    "N+W N   N   N   N   N+W   N+W   N+W\n" +
    "N+W N   N   N   N   N+BX  N+T#t   N+W\n" +
    "N+W N+W N+W N+W N+W N+D(#t)   N+W   N+W\n" +
    "N+W N     N+W     N     N     N     N   N+W\n" +
    "N+W N     N+W     N     N     N     N   N+W\n" +
    "N+W N+BY  N+W     N     N     N     N   N+W\n" +
    "N+W N     N+W     N     N     N     N   N+W\n" +
    "N+W N     N+W     N     N     N     N   N+W\n" +
    "N+W N+T#t2 N+W    N     N     N     N   N+W\n" +
    "N+W N+W   N+W     N+W   N+D(#t2) N+W   N+W N+W\n" +
    "N+W _N   _N    _N     N     N     N+C N+W\n" +
    "N+W _N   _N    _N     N     N     N   N+W\n" +
    "N+W _N   _N    _N    _N    _N    _N   N+W\n" +
    "N+W _N   _N    _N    _N    _N    _N   N+W\n" +
    "N+W _N   _N    _N    _N    _N    _N   N+W\n" +
    "_N+W _N   _N    _N    _N    _N    _N   _N+W\n" +
    "_N+W _N   _N    _N    _N    _N    _N   _N+W\n" +
    "_N+W _N   _N    _N    _N    _N    _N   _N+W\n" +
    "_N+W+W+W _N+W+W+W   _N+W+W+W    _N+W+W+W    _N+W+W+W    _N    _N   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N+W+W+W    _N+W+W+W    _N+W+W+W    _N    _N   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N+W+W+W    _N+W+W+W    _N+W+W+W    _N    _N   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N+W+W+W    _N+W+W+W    _N    _N    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N+W+W+W    _N+W+W+W    _N    _N    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N+W+W+W    _N+W+W+W    _N    _N    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N   _N    _N    _N    _N    _N   _N+W+W+W\n" +
    "_N+W+W+W _N   _N    _N    _N    _N    _N   _N+W+W+W\n" +
    "_N+W+W+W _N   _N    _N    _N    _N+W+W+W    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N   _N    _N    _N    _N+W+W+W    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N   _N    _N    _N+W+W+W  _N+W+W+W    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N   _N    _N    _N+W+W+W  _N+W+W+W    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N   _N    _N    _N+W+W+W  _N+W+W+W    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N    _N    _N    _N    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N    _N    _N    _N    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N    _N    _N    _N    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N+W+W+W    _N+W+W+W    _N    _N    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N+W+W+W    _N+W+W+W    _N    _N    _N+W+W+W   _N+W+W+W\n" +
    "_N+W+W+W _N+W+W+W   _N+W+W+W    _N+W+W+W    _N    _N    _N+W+W+W   _N+W+W+W\n" +
    "_N+W _N     _N    _N    _N    _N     _N   _N+W\n" +
    "_N+W _N     _N    _N    _N    _N     _N   _N+W\n" +
    "_N+W _N     _N    _N    _N    _N     _N     _N  _N _N+W\n" +
    "N+W N     N     N     N     N     N     N  N N+W\n" +
    "N+W N     N     N     N     N     N     N  N+C N+W\n" +
    "N+W N     N     N     N     N     N     N+C  N+C N+W\n" +
    "N+W N+W   N+W     N+W   N+W N+W   N+W N+W N+C N+W\n" +
    "N+W N     N     N     N     N     N     N  N N+W\n" +
    "N+W N     N+BY     N     N     N     N     N+E1  N+E1 N+W\n" +
    "N+W N     N     N     N     N     N     N  N N+W\n" +
    "N+W N+E2X N     N     N+W   N     N     N  N N+W\n" +
    "N+W N     N+E2X N     N+W   N     N     N  N N+W\n" +
    "N+W N     N     N+E2X N+W   N     N     N  N N+W\n" +
    "N+W N+E2X N     N     N+W   N     N     N  N N+W\n" +
    "N+W N+E2X N     N     N+W   N     N     N  N N+W\n" +
    "N+W N     N+E2X N     N+W   N     N     N  N N+W\n" +
    "N+W N     N     N+E2X N+W   N     N     N  N N+W\n" +
    "N+W N+E2X N     N     N+W   N     N     N  N N+W\n" +
    "N+W N     N     N     N+W   N     N     N  N N+W\n" +
    "N+W N     N+T#t3 N     N+W  N+W   N+D(#t3)   N+W N+W N+W\n" +
    "N+W N+W   N+W     N+W   N+W     N+C     N+C     N+C  N N+W\n" +
    "N+W N+W   N+W     N+W   N+W     N+C     N+C     N+C  N N+W\n" +
    "_N+W _N     _N     _N     _N     _N     _N     _N   _N _N+W\n" +
    "_N+W _N     _N     _N+E2X     _N     _N     _N     _N   _N _N+W\n" +
    "_N+W _N     _N     _N     _N     _N+E2X     _N     _N   _N _N+W\n" +
    "_N+W _N     _N+E2X     _N     _N     _N     _N     _N   _N _N+W\n" +
    "_N+W _N     _N     _N     _N     _N     _N     _N   _N _N+W\n" +
    "_N+W _N     _N     _N     _N     _N     _N     _N   _N _N+W\n" +
    "_N+W _N     _N     _N     _N     _N+E2X     _N     _N   _N _N+W\n" +
    "_N+W _N     _N     _N     _N     _N     _N     _N   _N _N+W\n" +
    "_N+W _N     _N     _N     _N+E2X     _N     _N     _N   _N _N+W\n" +
    "_N+W _N     _N+E2X     _N     _N     _N     _N     _N   _N _N+W\n" +
    "_N+W _N     _N     _N     _N     _N     _N     _N   _N _N+W\n" +
    "N+W N     N     N     N     N     N     N  N N+W\n" +
    "N+W N     N     N     N     N     N     N  N N+W\n" +
    "N+W N     N     N     N     N     N     N  N N+W\n" +
    "N+W N     N     N     N+G     N+G     N     N  N N+W\n" +
    "N+W N     N     N     N+G     N+G     N     N  N N+W\n" +
    "N+W N+W     N+W     N+W     N+W     N+W     N+W     N+W  N+W N+W\n";

var semi = nipplejs.create({
    zone: document.getElementById('game-controller-joystick'),
    mode: 'static',
    position: { left: '50%', top: '50%' },
    catchDistance: 80,
    size: 50,
    threshold: 0.0,
    color: 'white',
});

var joystick_position = { x: 0, y: 0 };
var is_active = false;

semi.on('start', function (_evt, _data) {
    is_active = true;
});

semi.on('move', function (_evt, data) {
    joystick_position.x = data.vector.x;
    joystick_position.y = data.vector.y;
    joystick_input(joystick_position.x, joystick_position.y);
});

semi.on('end', function (_evt, _data) {
    joystick_position = { x: 0, y: 0 };
    joystick_input(joystick_position.x, joystick_position.y);
    is_active = false;
});

// periodically send joystick input to the game
// this is needed as the joystick will not emit events when it is not moved
setInterval(() => {
    if (is_active) {
        joystick_input(joystick_position.x, joystick_position.y);
    }
}, 1000 / 80);

// The Markdown parser will dynamically load parsers
// for code blocks, using @codemirror/language-data to
// look up the appropriate dynamic import.
let view = new EditorView({
    doc: "", // level
    extensions: [
      basicSetup,
    ],
    parent: document.getElementById("editor")!,
  });
  
  // on pressing load button get the content of the editor and load it
  document.getElementById("load-button")!.addEventListener("click", () => {
    const level = view.state.doc.toString();
    load_level(level);
  
    // set it to the url with ?level=... so that it can be shared
    const url = compress_level_to_url(level)
    window.history.replaceState({}, "", "?level=" + url);
  });

init().then(() => {
    console.log("WASM Loaded");

    // get level from ?level=... url parameter if it exists otherwise use INITIAL_LEVEL
    const urlParams = new URLSearchParams(window.location.search);
    // decompress the level from the url or fall back to INITIAL_LEVEL
    var level = INITIAL_LEVEL;
    try {
        level = decompress_level_from_url(urlParams.get('level'));
    }
    catch (e) {
        console.log("Could not decompress level from url: " + e);
    }
    load_level(level);

    // set the level to the editor
    view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: level },
    });

    // set the size of the canvas to the size of the game-container
    const gameContainer = document.getElementById("game-container");
    // listen on resize of gameContainer
    new ResizeObserver(() => {
        const gameCanvas = document.querySelector("#game-container > canvas");
        gameCanvas.width = gameContainer.clientWidth / 2.0;
        gameCanvas.height = gameContainer.clientHeight / 2.0;
        set_size(gameContainer.clientWidth, gameContainer.clientHeight);
    }).observe(gameContainer);
});

let action_button = nipplejs.create({
    zone: document.getElementById('game-controller-action-button'),
    mode: 'static',
    position: { left: '50%', top: '50%' },
    color: 'white',
    lockX: true,
    lockY: true,
    shape: 'square',
    size: 50,
});

action_button.on('start', function (_evt, _data) {
    action_button_pressed();
});

action_button.on('end', function (_evt, _data) {
    action_button_released();
});