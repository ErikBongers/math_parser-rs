// import * as mp from "./parser.js"
import * as cloud from "./cloud.js"


//TODO: MVC:
// user clicks > controller (handler)
// controller > model (localStorage)
// model > view (codeMirror)
// .
// but what about the startup code?

let radioButtonsScript = ["viewMainScript", "viewStartupScript"];
let radioButtonsTheme = ["setLightMode", "setDarkMode"];

function switchRadioButton(buttons, button) {
    for (const buttonId of buttons) {
        document.getElementById(buttonId).classList.remove("bullet");
    }
    if (typeof button === "string") {
        button = document.getElementById(button);
    }
    button.classList.add("bullet");
}

document.getElementById("viewMainScript").addEventListener("click", function (e) {
    if (this.classList.contains("bullet")) {
        return; //already at main script
    }
    this.classList.add("bullet");
    document.getElementById("viewStartupScript").classList.remove("bullet");
    cloud.switchToScript("script1")
});

document.getElementById("viewStartupScript").addEventListener("click", function (e) {
    if (this.classList.contains("bullet")) {
        return; //already at  start script
    }
    this.classList.add("bullet");
    document.getElementById("viewMainScript").classList.remove("bullet");
    cloud.switchToScript("start")
});

document.getElementById("setDarkMode").addEventListener("click", function (e) {
    setLocalStorageTheme(true);
    updateTheme();
    updateMenu();
})

//TODO: configure this to work with generic radiobuttons.
function setLocalStorageTheme(dark) {
    localStorage.darkTheme = dark;
}

document.getElementById("setLightMode").addEventListener("click", function (e) {
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
        document.getElementById("viewMainScript").classList.remove("bullet");
        document.getElementById("viewStartupScript").classList.add("bullet");
    } else {
        document.getElementById("viewMainScript").classList.add("bullet");
        document.getElementById("viewStartupScript").classList.remove("bullet");
    }
}

function menu_setTheme(dark) {
    switchRadioButton(radioButtonsTheme, localStorage.darkTheme === "true" ? "setDarkMode" :"setLightMode");
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
    document.getElementById("hideShowLineNumbers").classList.toggle("checked", localStorage.showLineNumbers==="true");
    document.getElementById("hideShowErrorColumn").classList.toggle("checked", localStorage.showErrors==="true");
}