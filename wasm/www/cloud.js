import * as mp from "./parser.js"
import init, { parse_direct as parseDirect, parse as parseMath, upload_source as uploadSource, get_math_version as getMathVersion } from './pack/wasm.js';

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

async function promptAndUseServerFile(scriptId) {
    if (!userSession)
        return;
    if (!userSession.sessionId)
        return;
    let cloudText = await downloadScript(scriptId);
    if (cloudText == undefined)
        return;
    let localText = getLocalScript(scriptId);
    if (cloudText === localText)
        return;
    if (confirm("Use server file : " + scriptId + "?")) {
        let transaction = cm.editor.state.update({ changes: { from: 0, to: cm.editor.state.doc.length, insert: cloudText} });
        cm.editor.update([transaction]);
        }
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

export function startUp() {
    window.document.title = "Math Parser " + getMathVersion();
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
        uploadSource("start", localStorage.savedStartCode);
        let theText = cm.editor.state.doc.toString();
        sourceIndex = uploadSource(scriptId, theText);
        sourceIndex = 0;//TODO: remove this line when source_index is implemented.
        // result = parseMath("start", scriptId);
        result = parseDirect(theText);
    } else {
        sourceIndex = uploadSource(scriptId, cm.editor.state.doc.toString());
        result = parseMath("", scriptId);
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

