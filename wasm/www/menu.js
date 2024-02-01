// import * as mp from "./parser.js"
import * as cloud from "./cloud.js"

document.getElementById("viewMainScript").addEventListener("click", function (e) {
    if (this.classList.contains("bullet") === true) {
        console.log("already at  main script");
    } else {
        console.log("switching to main script.");
        this.classList.add("bullet");
        document.getElementById("viewStartupScript").classList.remove("bullet");
        cloud.switchToScript("script1")
    }
});

document.getElementById("viewStartupScript").addEventListener("click", function (e) {
    if (this.classList.contains("bullet") === true) {
        console.log("already at  start script");
    } else {
        console.log("switching to start script.");
        this.classList.add("bullet");
        document.getElementById("viewMainScript").classList.remove("bullet");
        cloud.switchToScript("start")
    }
});

export function setScript(scriptId) {
    if(scriptId === "start") {
        document.getElementById("viewMainScript").classList.remove("bullet");
        document.getElementById("viewStartupScript").classList.add("bullet");
    } else {
        document.getElementById("viewMainScript").classList.add("bullet");
        document.getElementById("viewStartupScript").classList.remove("bullet");
    }
}

function toggleLocalStorage(id) {
    localStorage.setItem(id, localStorage.getItem(id) === "true" ? "false" : "true");
}

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

export function updateGutter() {
    cm.showGutter(localStorage.showLineNumbers === "true", localStorage.showErrors === "true");
}

export function updateMenu() {
    setScript(localStorage.lastScript);
    document.getElementById("hideShowLineNumbers").classList.toggle("checked", localStorage.showLineNumbers==="true");
    document.getElementById("hideShowErrorColumn").classList.toggle("checked", localStorage.showErrors==="true");
}