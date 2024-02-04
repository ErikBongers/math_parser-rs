import * as cloud from "./cloud.js"


class MenuState {
    #state = {
        theme: "",
        scriptId: "",
        showErrors: true,
        showLineNumbers: true,
    };
    constructor() {
        if(localStorage.menuState) {
            this.#state = JSON.parse(localStorage.menuState);
        } else {
            this.#state.scriptId = "script1";
            this.#state.showErrors = true;
            this.#state.showLineNumbers = true;
            this.#state.theme = "light";
        }
    }
    saveInLocalStorage() {
        localStorage.menuState = JSON.stringify(this.#state);
    }

    getTheme() { return this.#state.theme; }
    getScriptId() { return this.#state.scriptId; }
    getShowErrors() { return this.#state.showErrors; }
    getShowLineNumbers() { return this.#state.showLineNumbers; }

    setTheme(theme) {
        this.#state.theme = theme;
        this.saveInLocalStorage();
        //listeners
        updateTheme();
        updateMenu();
    }

    setScript(scriptId) {
        if ( this.#state.scriptId === scriptId ) {
            return;
        }
        this.#state.scriptId = scriptId;
        this.saveInLocalStorage();
        // listeners
        updateScript();
        updateMenu();
    }

    setShowErrors(show) {
        this.#state.showErrors = show;
        this.saveInLocalStorage();
        //listeners
        updateGutter();
        updateMenu();
    }

    setShowLineNumbers(show) {
        this.#state.showLineNumbers = show;
        this.saveInLocalStorage();
        //listeners
        updateGutter();
        updateMenu();
    }

    toggleLineNumbers() {
        this.setShowLineNumbers(!this.#state.showLineNumbers);
    }

    toggleErrors() {
        this.setShowErrors(!this.#state.showErrors);
    }
}

export let menuState = new MenuState();

//TODO: MVC:
// user clicks > controller (handler)
// controller > model (state/localStorage)
// model > view via listeners (menu items, codeMirror, html, ...)
// .
// but what about the startup code?

const ID_MENU_MAIN_SCRIPT = "viewMainScript";
const ID_MENU_START_SCRIPT = "viewStartupScript";
const ID_MENU_LIGHT_MODE = "setLightMode";
const ID_MENU_DARK_MODE = "setDarkMode";
const ID_MENU_SHOW_LINE_NUMBERWS = "hideShowLineNumbers";
const ID_MENU_SHOWW_ERRORS = "hideShowErrorColumn";

const CLASS_BULLET = "bullet";
const CLASS_CHECKED = "checked";

const ID_START_SCRIPT = "start";
const ID_SCRIPT1 = "script1";

//*****  SET MENU CLICK LISTENERS *****

document.getElementById(ID_MENU_MAIN_SCRIPT).addEventListener("click", function (e) {
    menuState.setScript(ID_SCRIPT1);
});

document.getElementById(ID_MENU_START_SCRIPT).addEventListener("click", function (e) {
    menuState.setScript(ID_START_SCRIPT);
});

document.getElementById(ID_MENU_DARK_MODE).addEventListener("click", function (e) {
    menuState.setTheme("dark");
})

document.getElementById(ID_MENU_LIGHT_MODE).addEventListener("click", function (e) {
    menuState.setTheme("light");
})

document.getElementById(ID_MENU_SHOW_LINE_NUMBERWS).addEventListener("click", function (e) {
    menuState.toggleLineNumbers();
});

document.getElementById(ID_MENU_SHOWW_ERRORS).addEventListener("click", function (e) {
    menuState.toggleErrors();
});

// Listeners, kind of...

export function updateGutter() {
    cm.showGutter(menuState.getShowLineNumbers(), menuState.getShowErrors());
}

export function updateTheme() {
    cm.setDarkTheme(menuState.getTheme() === "dark");
    document.body.classList.toggle("dark", menuState.getTheme() === "dark");
}

export function updateScript() {
    //TODO: these are 2 separate "views" and thus listeners
    cloud.promptAndUseServerFile(menuState.getScriptId());
    document.getElementById("script-name").innerHTML = getCurrentScriptName();

}

export function updateMenu() {
    menu_setScript(menuState.getScriptId());
    menu_setTheme(menuState.getTheme());
    document.getElementById(ID_MENU_SHOW_LINE_NUMBERWS).classList.toggle(CLASS_CHECKED, menuState.getShowLineNumbers());
    document.getElementById(ID_MENU_SHOWW_ERRORS).classList.toggle(CLASS_CHECKED, menuState.getShowErrors());
}

// Helpers

export function menu_setScript(scriptId) {
    switchRadioButton(radioButtonsScript, menuState.getScriptId() === ID_START_SCRIPT ? ID_MENU_START_SCRIPT :ID_MENU_MAIN_SCRIPT);
}

function menu_setTheme(dark) {
    switchRadioButton(radioButtonsTheme, menuState.getTheme() === "dark" ? ID_MENU_DARK_MODE :ID_MENU_LIGHT_MODE);
}

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

export function getCurrentScriptName() {
    if (menuState.getScriptId() === "start")
        return "start script";
    else
        return "main script";
}

