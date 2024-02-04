// import * as mp from "./parser.js"
import * as cloud from "./cloud.js"


//TODO: MVC:
// user clicks > controller (handler)
// controller > model (localStorage)
// model > view (codeMirror)
// .
// but what about the startup code?

const ID_MENU_MAIN_SCRIPT = "viewMainScript";
const ID_MENU_START_SCRIPT = "viewStartupScript";
const ID_MENU_LIGHT_MODE = "setLightMode";
const ID_MENU_DARK_MODE = "setDarkMode";
const CLASS_BULLET = "bullet";
const CLASS_CHECKED = "checked";

let radioButtonsScript = [ID_MENU_MAIN_SCRIPT, ID_MENU_START_SCRIPT];
let radioButtonsTheme = [ID_MENU_LIGHT_MODE, ID_MENU_DARK_MODE];

function switchRadioButton(buttons, button) {
    for (const buttonId of buttons) {
        document.getElementById(buttonId).classList.remove(CLASS_BULLET);
    }
    if (typeof button === "string") {
        button = document.getElementById(button);
    }
    button.classList.add(CLASS_BULLET);
}

document.getElementById(ID_MENU_MAIN_SCRIPT).addEventListener("click", function (e) {
    if (this.classList.contains(CLASS_BULLET)) {
        return; //already at main script
    }
    this.classList.add(CLASS_BULLET);
    document.getElementById(ID_MENU_START_SCRIPT).classList.remove(CLASS_BULLET);
    cloud.switchToScript("script1")
});

document.getElementById(ID_MENU_START_SCRIPT).addEventListener("click", function (e) {
    if (this.classList.contains(CLASS_BULLET)) {
        return; //already at  start script
    }
    this.classList.add(CLASS_BULLET);
    document.getElementById(ID_MENU_MAIN_SCRIPT).classList.remove(CLASS_BULLET);
    cloud.switchToScript("start")
});

document.getElementById(ID_MENU_DARK_MODE).addEventListener("click", function (e) {
    setLocalStorageTheme(true);
    updateTheme();
    updateMenu();
})

//TODO: configure this to work with generic radiobuttons.
function setLocalStorageTheme(dark) {
    localStorage.darkTheme = dark;
}

document.getElementById(ID_MENU_LIGHT_MODE).addEventListener("click", function (e) {
    setLocalStorageTheme(false);
    updateTheme();
    updateMenu();
})

document.getElementById("hideShowLineNumbers").addEventListener("click", function (e) {
    toggleLocalStorage("showLineNumbers")
    updateGutter();
    updateMenu();
});

document.getElementById("hideShowErrorColumn").addEventListener("click", function (e) {
    toggleLocalStorage("showErrors")
    updateGutter();
    updateMenu();
});

export function menu_setScript(scriptId) {
    if(scriptId === "start") {
        document.getElementById(ID_MENU_MAIN_SCRIPT).classList.remove(CLASS_BULLET);
        document.getElementById(ID_MENU_START_SCRIPT).classList.add(CLASS_BULLET);
    } else {
        document.getElementById(ID_MENU_MAIN_SCRIPT).classList.add(CLASS_BULLET);
        document.getElementById(ID_MENU_START_SCRIPT).classList.remove(CLASS_BULLET);
    }
}

function menu_setTheme(dark) {
    switchRadioButton(radioButtonsTheme, localStorage.darkTheme === "true" ? ID_MENU_DARK_MODE :ID_MENU_LIGHT_MODE);
}

function toggleLocalStorage(id) {
    localStorage.setItem(id, localStorage.getItem(id) === "true" ? "false" : "true");
}

export function updateGutter() {
    cm.showGutter(localStorage.showLineNumbers === "true", localStorage.showErrors === "true");
}

function updateTheme() {
    cm.setDarkTheme(localStorage.darkTheme === "true");
}

export function updateMenu() {
    menu_setScript(localStorage.lastScript);
    menu_setTheme(localStorage.darkTheme==="true");
    document.getElementById("hideShowLineNumbers").classList.toggle(CLASS_CHECKED, localStorage.showLineNumbers==="true");
    document.getElementById("hideShowErrorColumn").classList.toggle(CLASS_CHECKED, localStorage.showErrors==="true");
}