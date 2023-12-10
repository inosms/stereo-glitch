import nipplejs from 'nipplejs';
import init, { load_level, set_eye_distance, set_size, joystick_input, action_button_pressed, action_button_released } from "../pkg/stereo_glitch.js";

var semi = nipplejs.create({
    zone: document.getElementById('game-controller-joystick'),
    mode: 'static',
    position: {left: '50%', top: '50%'},
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

init().then(() => {
    console.log("WASM Loaded");
    window.load_level = load_level;
    window.set_eye_distance = set_eye_distance;

    load_level(
        "N+W N+W N+W N+W N+W N+W N+W N+W N+W N+W\n" +
        "N+W N   N   N   N   N   N   N   N N+W\n" +
        "N+W N   N   N   N   N   N   N   N N+W\n" +
        "N+W N   N   N   N+P N   N   N   N N+W\n" +
        "N+W N   N   N   N   N   N   N   N N+W\n" +
        "N+W N   N   N   N   N   N   N   N N+W\n" +
        "N+W N   N   N   N   N   N   N   N N+W\n" +
        "N+W N+W   N+W   N+W   N+D(#t)+D(#t) N+D(#t)+D(#t)   N+W   N+W   N+W N+W\n" +
        "N+W N     N     N     N     N     N     N  N N+W\n" +
        "N+W N     N     N     N     N     N     N  N N+W\n" +
        "N+W N     N     N      N+T#t+X+X+B     N     N     N  N N+W\n" +
        "N+W N     N     N     N     N     N     N  N N+W\n" +
        "N+W N     N     N     N     N     N     N  N N+W\n" +
        "N+W N     N     N     N     N     N     N  N N+W\n" +
        "N+W N     N     N     N+C     N     N     N  N N+W\n" +
        "N+W N     N     N     N     N     N     N  N N+W\n" +
        "N+W N     N    _N    _N    _N     N     N  N N+W\n" +
        "N+W N     N    _N    _N    _N     N     N  N N+W\n" +
        "N+W N     N    _N    _N    _N     N     N  N N+W\n" +
        "N+W N     N    _N    _N    _N     N     N  N N+W\n" +
        "N+W N     N     N     N+C   N     N     N  N N+W\n" +
        "N+W N     N     N     N     N     N     N  N N+W\n"
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

let action_button = nipplejs.create({
    zone: document.getElementById('game-controller-action-button'),
    mode: 'static',
    position: {left: '50%', top: '50%'},
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