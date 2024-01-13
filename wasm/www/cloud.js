import * as mp from "./parser.js"
import { MathParser } from './pack/wasm.js';

let userSession = {};

export function onSignedIn(googleUserToken) {
    userSession = JSON.parse(getCookie("mathparserSession"));
    let params = "";
    if (userSession && userSession.sessionId)
        params = "?sessionId=" + encodeURIComponent(userSession.sessionId);
    fetch("https://europe-west1-ebo-tain.cloudfunctions.net/get-session" + params, {
        method: "POST",
        mode: "cors",
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(googleUserToken)
    }).then(res => res.json())
        .then(jsonUserSession => {
            console.debug("Request complete! response:", jsonUserSession);
            document.getElementById("userName").innerHTML = jsonUserSession.user.name;
            userSession = jsonUserSession;
            setCookie("mathparserSession", JSON.stringify(userSession), 1);
            let scriptId = document.getElementById("scriptSelector").value;
            promptAndUseServerFile(scriptId);
        });
}

async function promptServerFile(scriptId) {
    let localText = getLocalScript(scriptId);
    if (!userSession)
        return localText;
    if (!userSession.sessionId)
        return localText;
    let cloudText = await downloadScript(scriptId);
    if (cloudText === undefined)
        return localText;
    if (cloudText === localText)
        return localText;
    if (confirm("Use server file : " + scriptId + "?")) {
        return cloudText;
        }
    return localText;
}

function promptAndUseServerFile(scriptId) {
    promptServerFile(scriptId).then( text => {
        let transaction = cm.editor.state.update({ changes: { from: 0, to: cm.editor.state.doc.length, insert: text} });
        cm.editor.update([transaction]);
    });
}

function uploadScript(scriptId, text) {
    if (!userSession) {
        console.log("Not uploading: no session.");
        return;
    }

    if (!userSession.sessionId) {
        console.log("Not uploading: no sessionId.");
        return;
    }

    let url = new URL("https://europe-west1-ebo-tain.cloudfunctions.net/cloud-script");
    url.searchParams.append("sessionId", userSession.sessionId);
    url.searchParams.append("scriptId", scriptId);
    fetch(url, {
        method: "POST",
        mode: "cors",
        headers: { 'Content-Type': 'plain/text' },
        body: text
    }).then(res => res.json())
        .then(jsonResult => {
            console.log("Upload complete!");
            console.debug("response:", jsonResult);
        });

}

async function downloadScript(scriptId) {
    if (!userSession.sessionId)
        throw new Error("Can't upload: no session.");

    let url = new URL("https://europe-west1-ebo-tain.cloudfunctions.net/cloud-script");
    url.searchParams.append("sessionId", userSession.sessionId);
    url.searchParams.append("scriptId", scriptId);

    const response = await fetch(url, {
        method: "GET",
        mode: "cors",
        headers: { 'Content-Type': 'plain/text' }
    });

    const text = await response.text();
    console.log("download complete!");
    console.debug("response:", text);
    return text;
}

function getLocalScript(scriptId) {
    let txt = "";
    if (scriptId == "start")
        txt = localStorage.savedStartCode;

    else
        txt = localStorage.savedCode;
    return txt;
}


export function saveScript(scriptId) {
    uploadScript(scriptId, cm.editor.state.doc.toString());
    if (scriptId == "start") {
        localStorage.savedStartCode = cm.editor.state.doc.toString();
    } else {
        localStorage.savedCode = cm.editor.state.doc.toString();
    }
}

export function onScriptSwitch() {
    let scriptId = document.getElementById("scriptSelector").value
    if (scriptId == "start") {
        promptAndUseServerFile("start");
        localStorage.lastScript = "start";
    } else {
        promptAndUseServerFile("script1");
        localStorage.lastScript = "script1";
    }
}

let parserInstance = {}

export function startUp() {
    parserInstance = MathParser.new();
    window.document.title = "Math Parser " + MathParser.get_math_version(); //getMathVersion();
    mp.sources[0] = "start";
    mp.sources[1] = "script1";

    cm.setLintSource((view) => {
        afterEditorChange();
        return mp.errorsForLint;
    });

    if (window.innerWidth > 480)
        cm.showGutter();
    else
        cm.hideGutter();

    let startScript = "script1";
    if (localStorage.lastScript)
        startScript = localStorage.lastScript;
    document.getElementById("scriptSelector").value = startScript;
    let txt = getLocalScript(startScript);
    let transaction = cm.editor.state.update({ changes: { from: 0, to: cm.editor.state.doc.length, insert: txt } });
    cm.editor.update([transaction]);
}

export function afterEditorChange() {
    let scriptId = document.getElementById("scriptSelector").value
    saveScript(scriptId);
    parseAfterChange(scriptId);
}

function parseAfterChange(scriptId) {
    let result = {};
    let sourceIndex = -1;
    if (scriptId !== "start") {
        if (!localStorage.savedStartCode)
            localStorage.savedStartCode = "";
        parserInstance.add_source("start", localStorage.savedStartCode);
        sourceIndex = parserInstance.add_source(scriptId, cm.editor.state.doc.toString());//TODO: move this line up and de-duplicate it
        result = parserInstance.parse("start", scriptId);
    } else {
        sourceIndex = parserInstance.add_source(scriptId, cm.editor.state.doc.toString());
        result = parserInstance.parse("", scriptId);
    }
    mp.outputResult(result, sourceIndex);
}

function setCookie(name, value, days) {
    var expires = "";
    if (days) {
        let date = new Date();
        date.setTime(date.getTime() + (days * 24 * 60 * 60 * 1000));
        expires = "; expires=" + date.toUTCString();
    }
    document.cookie = name + "=" + (value || "") + expires + "; path=/";
}
function getCookie(name) {
    var nameEQ = name + "=";
    var ca = document.cookie.split(';');
    for (let i = 0; i < ca.length; i++) {
        let c = ca[i];
        while (c.charAt(0) == ' ') c = c.substring(1, c.length);
        if (c.indexOf(nameEQ) == 0) return c.substring(nameEQ.length, c.length);
    }
    return null;
}

