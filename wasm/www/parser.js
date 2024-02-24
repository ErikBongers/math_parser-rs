export let errorsForLint = [];
export let activeDocumentIndex = -1;
export let sources = [];
export function clearErrorList() {
    errorsForLint = [];
}
function getFirstErrorForLine(lineNo, errors) {
    for (let e of errors) {
        if (e.range.startLine === lineNo && e.type === "E") {
            return e.msg;
        }
    }
    return "";
}
function getFirstWarningForLine(lineNo, errors) {
    for (let e of errors) {
        if (e.range.startLine === lineNo && e.type === "W") {
            return e.msg;
        }
    }
    return "";
}
function convertErrorToCodeMirror(e, doc) {
    let start = 0;
    let end = 0;
    try {
        if (e.range.sourceIndex == activeDocumentIndex) {
            start = doc.line(e.range.startLine + 1).from + e.range.startPos;
            end = doc.line(e.range.endLine + 1).from + e.range.endPos;
        }
        else {
            start = doc.line(1).from;
            end = doc.line(1).from;
        }
        let hint = {
            message: prefixErrorMessage(e),
            severity: "error",
            from: start,
            to: end
        };
        if (e.type === "W")
            hint.severity = "warning";
        return hint;
    }
    catch (err) {
        console.error(e);
        throw err;
    }
}
function addErrorsToLint(errors) {
    for (let e of errors) {
        errorsForLint.push(convertErrorToCodeMirror(e, cm.editor.state.doc));
        if (e.stackTrace)
            addErrorsToLint(e.stackTrace);
    }
}
function formatNumber(numb) {
    let strFormatted = numb.fmtd;
    strFormatted += numb.u;
    return strFormatted;
}
function formatList(values) {
    let strFormatted = "";
    let strComma = "";
    values.forEach(value => {
        strFormatted += strComma + formatResult(value);
        strComma = ", ";
    });
    return strFormatted;
}
function formatResult(line) {
    let strFormatted = "";
    if (line.type === "Number" || line.type === "N") {
        strFormatted = formatNumber(line.number);
    }
    else if (line.type === "Timepoint" || line.type === "T")
        strFormatted = line.date.formatted;
    else if (line.type === "Duration" || line.type === "D") {
        strFormatted = line.duration.years + " years, " + line.duration.months + " months, " + line.duration.days + " days";
    }
    else if (line.type === "List" || line.type === "L") {
        strFormatted = "(" + formatList(line.list) + ")";
    }
    else if (line.type === "Last") {
        strFormatted = "'last'";
    }
    else if (line.type === "FunctionDef") {
        strFormatted = "Function";
    }
    return strFormatted;
}
function prefixErrorMessage(e) {
    let sourcePrefix = "";
    if (e.range.sourceIndex !== activeDocumentIndex) {
        sourcePrefix = "[" + sources[e.range.sourceIndex] + "]: ";
    }
    return sourcePrefix + e.msg;
}
function linetoResultString(line, errors, showErrors) {
    if (line.type === "Comment") {
        return "[comment:" + line.comment + "]";
    }
    let strLine = "";
    if (window.innerWidth > 880) {
        strLine += (line.id ? line.id + "=" : "") + formatResult(line);
    }
    else
        strLine += formatResult(line);
    return strLine;
}
function getErrorsAndWarningsForLine(lineNo, errors) {
    let strError = getFirstErrorForLine(lineNo, errors);
    if (strError !== "")
        strError = "[error:" + strError + "]";
    else {
        let strWarning = getFirstWarningForLine(lineNo, errors);
        if (strWarning !== "")
            strError = "[warning:" + strWarning + "]";
    }
    return strError;
}
export function outputResult(resultString, sourceIndex, showErrors) {
    activeDocumentIndex = sourceIndex;
    console.debug(resultString);
    clearErrorList();
    var strOutput = "";
    var strResult = "";
    try {
        let result = JSON.parse(resultString); //may throw...
        let lineCnt = 0;
        let lineAlreadyFilled = false;
        let strLine = "";
        for (let line of result.result) {
            if (line.src !== sourceIndex)
                continue;
            //goto the next line in output
            while (lineCnt < line.line) {
                let strError = "";
                if (window.innerWidth > 880 && showErrors === true) {
                    strError = getErrorsAndWarningsForLine(lineCnt, result.errors);
                }
                strResult += strError + strLine + "\n";
                strLine = "";
                lineCnt++;
                lineAlreadyFilled = false;
            }
            let strValue = linetoResultString(line, result.errors, showErrors);
            if (lineAlreadyFilled) {
                if (line.type === "Comment") {
                    strLine = strValue + " " + strLine; //comment to the left and no separator.
                }
                else {
                    if (strValue !== "") {
                        strLine += (strLine !== "" ? " | " : "") + strValue;
                    }
                }
            }
            else {
                strLine += strValue;
            }
            lineAlreadyFilled = true;
        }
        strResult += strLine; //ad the last line.
        addErrorsToLint(result.errors);
    }
    catch (e) {
        strOutput = e.message + "\n";
        strOutput += e.name + "\n";
        strOutput += e.stack + "\n";
        strResult = e.message + "\n";
    }
    let transaction = cm.cmOutput.state.update({ changes: { from: 0, to: cm.cmOutput.state.doc.length, insert: strOutput } });
    cm.cmOutput.update([transaction]);
    transaction = cm.cmResult.state.update({ changes: { from: 0, to: cm.cmResult.state.doc.length, insert: strResult } });
    cm.cmResult.update([transaction]);
}
