import nipplejs from 'nipplejs';
import init, { load_level, set_eye_distance, set_size, joystick_input } from "../pkg/stereo_glitch.js";

var semi = nipplejs.create({
    zone: document.getElementById('game-controller-joystick'),
    mode: 'static',
    position: {left: '50%', top: '50%'},
    catchDistance: 80,
    size: 70,
    threshold: 0.0,
    color: 'white',
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
}, 15);

init().then(() => {
    console.log("WASM Loaded");
    window.load_level = load_level;
    window.set_eye_distance = set_eye_distance;

    load_level(
        "N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N+P   N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N+T#t+X+X+B N N   N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+D(#t)+D(#t) N+D(#t)+D(#t) N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N    _N    _N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N    _N    _N    _N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N    _N    _N+B  _N     N     N     N     N+T#t2 N    N     N+W+W\n" +
        "N+W+W N     N     N     N    _N    _N    _N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N    _N    _N    _N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+D(#t2)+D(#t2) N+D(#t2)+D(#t2) N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     N     N     N     N     N     N+W+W\n" +
        "N+W+W N     N     N     N     N     N     N     N     X     X     X     X     X     X\n" +
        "N+W+W N     N     N     N     N     N     N     N     X     X     X     X     X     X\n" +
        "N+W+W N     N     N     N     N     N     N     N     X     X     X     X     X     X\n" +
        "N+W+W N     N     N     N     N     N     N    _N     X     X     X     X     X     X\n" +
        "N+W+W N     N     N     N     N     N    _N    _N     X     X     X     X     X     X\n" +
        "_N     N     N     N     N     N     N    _N    _N     N     N     N     N     N     N\n" +
        "_N     N     N     N     N     N     N    _N    _N    _N    _N     N     N     N     N\n" +
        "_N     N     N     N     N     N     N    _N    _N    _N    _N    _N     N     N     N\n" +
        "_N     N     N     N     N     N     N    _N    _N    _N    _N    _N     N     N     N\n" +
        "_N     N     N     N     N     N    _N    _N    _N    _N    _N    _N    _N    _N    _N\n" +
        "_N     N     N     N     N    _N    _N    _N    _X    _X    _N    _N    _N    _N    _N\n" +
        "_N     N     N     N     N    _N    _X    _X    _X    _X    _X    _X    _N    _N    _N\n" +
        "_N     N     N     N    _N    _N    _X    _X    _X    _X    _X    _X    _N    _N    _N\n" +
        "_N     N     N     N    _N    _N    _X    _X    _N    _N    _N    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _N    _N    _N    _N    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _N    _N    _N    _N    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _N    _N+B+B _N+B+B  _N  _X   _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _N    _N    _N    _N    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _N    _N    _N    _N    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _X    _N    _N    _N    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _X    _N    _N    _X    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _X    _N    _N    _X    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _X    _N    _N    _X    _X    _X    _X    _N\n" +
        "_N     N     N+T#t3     N    _N    _N    _X    _X    _N    _N    _X    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _X    _N    _N    _X    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _X    _N    _N    _X    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _X    _X    _N    _N    _X    _X    _X    _X    _N\n" +
        "_N     N     N     N    _N    _N    _N    _N    _N    _N    _N    _N    _N    _N    _N\n" +
        "_N     N     N     N    _N    _N     N    _N    _N    _N    _N    _N    _N    N     _N\n" +
        "_N     N     N     N    _N    _N    _N    _N     N    _N    _N    _N    _N    N     _N\n" +
        "_N     N     N     N    _N     N    _N    _N    _N    _N    _N    _N    _N    N     _N\n" +
        "_N     N     N     N    _N     N     N    _N     N    _N    _N    _N    _N    N     _N\n" +
        "_N     N     N     N    _N     N     N    _N     N    _N    _N    _N     N    N     _N\n" +
        "_N     N     N     N    _N     N     N    _N     N    _N    _N     N     N    N      N\n" +
        "_N     N     N     N    _N     N     N     N     N     N     N     N     N    N      N\n" +
        "_N     N     N     N     N     N     N     N     N     N     N     N     N    N      N\n" +
        "_N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W N+D(#t3)+D(#t3) N+D(#t3)+D(#t3) N+W+W N+W+W N+W+W N+W+W N+W+W N+W+W\n"
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

// TODO on click
nipplejs.create({
    zone: document.getElementById('game-controller-action-button'),
    mode: 'static',
    position: {left: '50%', top: '50%'},
    color: 'white',
    lockX: true,
    lockY: true,
    shape: 'square',
    size: 70,
});