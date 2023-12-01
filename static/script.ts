import nipplejs from 'nipplejs';
import init, { load_level, set_eye_distance, set_size, joystick_input } from "../pkg/stereo_glitch.js";

var semi = nipplejs.create({
    zone: document.getElementById('game-controller-joystick'),
    mode: 'dynamic',
    catchDistance: 80,
    threshold: 0.0,
    color: 'white'
});

var joystick_position = { x: 0, y: 0 };
semi.on('move', function (_evt, data) {
    joystick_position.x = data.vector.x;
    joystick_position.y = data.vector.y;
    joystick_input(joystick_position.x, joystick_position.y);
});

semi.on('end', function (_evt, _data) {
    joystick_position = { x: 0, y: 0 };
    joystick_input(joystick_position.x, joystick_position.y);
});

// periodically send joystick input to the game
// this is needed as the joystick will not emit events when it is not moved
setInterval(() => {
    joystick_input(joystick_position.x, joystick_position.y);
}, 100);

init().then(() => {
    console.log("WASM Loaded");
    window.load_level = load_level;
    window.set_eye_distance = set_eye_distance;

    load_level(
        "_N _N _N _N _N _N _N _N _N _N _N _N _N \n" +
        "_N _N _N _N _N _N _N _N _N _N _N _N _N \n" +
        "_N _N _N+P _N _N _N _N _N _N _N _N _N _N \n" +
        "_N _N _N _N _N _N _N _N _N _N _N _N _N \n" +
        "_N _N _N _N _N _N _N _N _N _N _N _N _N \n" +
        "_N _N _N _N _N _N _N _N _N _N _N _N _N \n" +
        "_N+W _N+W _N+W _N+W _N+W _N+W _N+W _N+W _N+W _N+W _N+W _N+W _N+W"
    );

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