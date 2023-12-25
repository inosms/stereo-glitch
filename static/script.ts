import nipplejs from 'nipplejs';
import init, { load_level, set_eye_distance, set_size, joystick_input, action_button_pressed, action_button_released, compress_level_to_url, decompress_level_from_url, check_level } from "../pkg/stereo_glitch.js";
import { basicSetup, EditorView } from "codemirror"

// export the functions 
export { load_level, set_eye_distance, compress_level_to_url, decompress_level_from_url };

// make the function available to the window
window.load_level = load_level;
window.set_eye_distance = set_eye_distance;
window.compress_level_to_url = compress_level_to_url;
window.decompress_level_from_url = decompress_level_from_url;

// https://stackoverflow.com/questions/11381673/detecting-a-mobile-browser
window.mobileCheck = function () {
    let check = false;
    (function (a) { if (/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino/i.test(a) || /1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s\-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|\-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw\-(n|u)|c55\/|capi|ccwa|cdm\-|cell|chtm|cldc|cmd\-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc\-s|devi|dica|dmob|do(c|p)o|ds(12|\-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(\-|_)|g1 u|g560|gene|gf\-5|g\-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd\-(m|p|t)|hei\-|hi(pt|ta)|hp( i|ip)|hs\-c|ht(c(\-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i\-(20|go|ma)|i230|iac( |\-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc\-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|\-[a-w])|libw|lynx|m1\-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m\-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(\-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)\-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|\-([1-8]|c))|phil|pire|pl(ay|uc)|pn\-2|po(ck|rt|se)|prox|psio|pt\-g|qa\-a|qc(07|12|21|32|60|\-[2-7]|i\-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h\-|oo|p\-)|sdk\/|se(c(\-|0|1)|47|mc|nd|ri)|sgh\-|shar|sie(\-|m)|sk\-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h\-|v\-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl\-|tdg\-|tel(i|m)|tim\-|t\-mo|to(pl|sh)|ts(70|m\-|m3|m5)|tx\-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|\-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(\-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas\-|your|zeto|zte\-/i.test(a.substr(0, 4))) check = true; })(navigator.userAgent || navigator.vendor || window.opera);
    return check;
};

export const INITIAL_LEVEL = 
`N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+Wx3
N+W	N	N	N	N	N+Wx3
N+W	N	N+P	N	N	N+Wx3
N+W	N	N	N	N	N+Wx3	N+Wx3	N+Wx3
N+W	N	N	N	N	N+BX	N+T#t	N+Wx3
N+W	N+W	N+W	N+W	N+W	N+D(#t)	N+W	N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+Wx3
N+W	N+T#t2	N+W	N	N	N	N	N+Wx3	N	N	N	N+E1	N+Wx3
N+W	N	N+W	N	N	N	N	N+D(#t2)x2+W	N	N+S	N	N	N+Wx3
N+W	N	N+W	N	N	N	N	N+D(#t2)x2+W	N	N	N	N	N+Wx3
N+W	N+BY	N+W	N	N	N	N	N+Wx3	N+C	N+C	N+C	N+C	N+Wx3
N+W	N+W	N+W	N+W	N+W	N+W	N+W	N+Wx3	N+C	N+C	N+C	N+C	N+Wx3
N+W	N+W	N+W	N+W	N+W	N+W	N+W	N+Wx3	N+C	N+C	N+C	N+C	N+Wx3
_N+Wx3	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N+Wx3
_N+Wx3	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N+Wx3
_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N	_N	_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3
_N+Wx3	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N+Wx3
_N+Wx3	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N+Wx3
_N+Wx3	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N	_N+Wx3
_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N+Wx3	_N	_N	_N+Wx3
N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+Wx3	N+C	N+C	N+Wx3
N+Wx2	N+E1	N	N+E1	N	N	N+BRF	N+BRF	N+Wx3	N	N+C	N+C	N+Wx3
N+Wx2	N	N+E1	N	N	N	N	N	N+Wx3	N	N	N	N+Wx3
N+Wx2	N+Wx2	N+Wx2	N+D(#t3)x2	N+D(#t3)x2	N+Wx2	N+Wx2	N+Wx2	N+Wx2	N+W	N	N+W	N+Wx3
N+W	N	N+T#g1	N	N	N	N+T#g2	N	N+E1	N+W	N	N+T#t4	N+Wx3
N+W	N	N	N	N	N	N	N	N	N+W	N	N	N+Wx3
N+W	N	N	N	N	N	N	N	N	N+W	N	N	N+Wx3
N+W	N	N	N	N	N	N	N	N	N+W	N	N+BY	N+Wx3
N+W	N+E2X	N	N	N	N	N	N	N	N+D(#t4)	N+T#t3	N	N+Wx3
N+W	N	N	N	N	N	N	N	N	N+W	N	N	N+Wx3
N+W	N+W	N+W	N+W	N+D(#g1)	N+W	N+W	N+W	N+W	N+W	N	N	N+Wx3
N+W	N+W	N+W	N+W	N+D(#g2)	N+W	N+W	N+W	N+W	N+W	N	N	N+Wx3
N+W	N	N	N	N	N	N	N	N	N+W	N+W	N+W	N+Wx3
N+W	N	N	N	N	N	N	N	N	N+W
N+W	N	N	N	N+G(END)	N	N	N	N	N+W
N+W	N+W	N+W	N+W	N+W	N+W	N+W	N+W	N+W	N+W`;

// split the level into blocks by whitespace
function split_level(level: string): string[] {
    return level.split(/\s+/);
}

// get the width of the longest block in the level
function get_max_width(level: string): number {
    return Math.max(...split_level(level).map((line) => line.length));
}

// replace consecutive whitespace with tabs
// do not replace newlines
function replace_consecutive_whitespace_with_tabs(level: string): string {
    // split the level into lines
    let lines = level.split("\n");
    // replace consecutive whitespace with tabs
    lines = lines.map((line) => line.replace(/\s+/g, "\t"));
    // join the lines back together
    return lines.join("\n");
}


if (window.mobileCheck()) {
    document.getElementById("game")!.classList.add("is-mobile");

    document.getElementById("center-container")!.addEventListener("touchmove", (e) => {
        e.preventDefault();
    }, { passive: false });

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
}

import { parser } from "./parser.js"
import { styleTags, tags as t } from "@lezer/highlight"

let parserWithMetadata = parser.configure({
    props: [
        styleTags({
            Concat: t.operator,
            Glitch: t.invalid,
            NormalFloor: t.atom,
            Wall: t.typeName,
            Enemy: t.typeName,
            Goal: t.typeName,
            Charge: t.typeName,
            Trigger: t.typeName,
            Door: t.typeName,
            Player: t.typeName,
            Box: t.typeName,
            Id: t.controlKeyword,
            "( )": t.paren
        })
    ]
});

import { LRLanguage } from "@codemirror/language"
import { EditorState, StateEffect } from "@codemirror/state"
import { linter, Diagnostic } from "@codemirror/lint"


export const levelfileLanguage = LRLanguage.define({
    parser: parserWithMetadata,
})

const levelLinter = linter(view => {
    let level = view.state.doc.toString();
    let check_result = check_level(level);
    let diagnostics: Diagnostic[] = []
    let json = JSON.parse(check_result);

    if (json.result === "error") {
        let loadButton = document.getElementById("load-button")!;
        loadButton.classList.remove("is-active");

        let parsedFailedRest = json.contents?.ParseFailed?.rest;
        let validationError = json.contents?.ValidationError?.message;

        if (parsedFailedRest !== undefined) {

            // Find the start position in the edit buffer
            let startPos = level.indexOf(parsedFailedRest);
            // The end position is for now just one character after the start position
            let endPos = startPos + 1;

            diagnostics.push({
                from: startPos,
                to: endPos,
                severity: "error",
                message: "Failed to parse level",
                actions: [],
            })
        }

        if (validationError !== undefined) {
            let startPos = 0;
            let endPos = level.length;

            diagnostics.push({
                from: startPos,
                to: endPos,
                severity: "warning",
                message: validationError,
                actions: [],
            })
        }
    } else if (json.result === "ok") {
        let loadButton = document.getElementById("load-button")!;
        loadButton.classList.add("is-active");
    }
    return diagnostics
});

function update_listener(e: any) {
    if (e.docChanged === true) {
        let level = e.state.doc.toString();
        clean_and_set_level(level);
    }
}

let extensions = [
    basicSetup,
    levelfileLanguage,
    levelLinter,
    EditorView.updateListener.of(update_listener),
]

let view = new EditorView({
    doc: "", // level
    extensions: extensions,
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

function clean_and_set_level(level: string): string {
    let cleaned_level = replace_consecutive_whitespace_with_tabs(level);

    // set the tab size to the width of the longest line in the level
    let tab_size = get_max_width(cleaned_level) + 1;

    if (cleaned_level == level && view.state.doc.toString() == cleaned_level && view.state.tabSize == tab_size) {
        // the level is already clean
        return cleaned_level;
    }

    view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: cleaned_level },
        // Do not reset the selection when cleaning the level
        // as this will cause the cursor to jump to the start of the level
        selection: view.state.selection,
        effects: StateEffect.reconfigure.of([
            EditorState.tabSize.of(tab_size),
            ...extensions,
        ]),
    });

    return cleaned_level;
}

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
    level = replace_consecutive_whitespace_with_tabs(level);
    load_level(level);

    clean_and_set_level(level);

    // set the size of the canvas to the size of the game-container
    const gameContainer = document.getElementById("game-container");
    // listen on resize of gameContainer
    new ResizeObserver(() => {
        const gameCanvas = document.querySelector("#game-container > canvas");
        gameCanvas.width = gameContainer.clientWidth;
        gameCanvas.height = gameContainer.clientHeight;
        set_size(gameContainer.clientWidth, gameContainer.clientHeight, window.devicePixelRatio);
        // set the css width and height to the same as the canvas divided by the devicePixelRatio
        // this is needed to make the canvas appear sharp on high dpi screens
        gameCanvas.style.width = gameCanvas.width + "px";
        gameCanvas.style.height = gameCanvas.height + "px";
    }).observe(gameContainer);
});
