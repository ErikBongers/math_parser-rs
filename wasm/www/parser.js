export let errorsForLint = [];
export let activeDocumentIndex = -1;
export let sources = [];
export function clearErrorList() {
	errorsForLint = [];
}

function convertErrorToCodeMirror(e, doc) {
	let start, end = 0;
	let sourcePrefix = "";
	try {
		if (e.range.sourceIndex == activeDocumentIndex) {
			start = doc.line(e.range.startLine + 1).from + e.range.startPos;
			end = doc.line(e.range.endLine + 1).from + e.range.endPos;
		} else {
			start = doc.line(1).from;
			end = doc.line(1).from;
		}
		let hint = {
			message: prefixErrorMessage(e),
			severity: "error",
			from: start,
			to: end
		}
		if (e.type == "W")
			hint.severity = "warning";
		return hint;
	}
	catch (err) {
		console.error(e);
		throw err;
	}
}

function addErrorsToLint (errors) {
	for (let e of errors) {
		errorsForLint.push(convertErrorToCodeMirror(e, cm.editor.state.doc));
		if (e.stackTrace)
			addErrorsToLint(e.stackTrace);
	}
}

function formatNumber (numb) {
	let strFormatted = "";
	if (numb.fmt == "Dec")
		strFormatted = formatFloatString(numb.sig, numb.exp);
	else
		strFormatted = numb.fmtd;
	strFormatted += numb.u;
	return strFormatted;
}

function formatList (values) {
	let strFormatted = "";
	let strComma = "";
	values.forEach(value => {
		strFormatted += strComma + formatResult(value);
		strComma = ", ";
	});
	return strFormatted;
}

function formatResult (line) {
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
	return strFormatted;
}

function prefixErrorMessage(e) {
	let sourcePrefix = "";
	if (e.range.sourceIndex !== activeDocumentIndex) {
		sourcePrefix = "[" + sources[e.range.sourceIndex] + "]: ";
	}
	return sourcePrefix + e.msg;
}

function lineToOutputString (line) {
	if (line.onlyComment === true) {
		return "//" + line.comment;
	}
	let strComment = "";
	if (line.comment)
		strComment = " //" + line.comment;
	let strNL = "";
	let strText = "";
	if (line.text)
		strText = " " + line.text;
	let strLine = "";
	let strFormatted = formatResult(line);

	if (!line.mute)
		strLine += (line.id ? line.id + "=" : "") + strFormatted;
	strLine += strText;
	strLine += strComment;
	return strLine;
}

export function linetoResultString (line) {
	if (line.type === "Comment") {
		return line.comment;
	}
	let strComment = "";
	if (line.comment)
		strComment = line.comment;
	let strLine = "";
	if (window.innerWidth > 880) {
		strLine += strComment;
		strLine += (line.id ? line.id + "=" : "") + formatResult(line);
	}
	else
		strLine += formatResult(line);

	return strLine;
}

function formatFloatString (floatString, exponent) {
	var sFixed = parseFloat(floatString).toFixed(5);
	if (sFixed.search(".") == -1)
		return sFixed;
	var sResult = "";
	var removeLastZero = true;
	[...sFixed].slice().reverse().forEach(function (c) {
		if (c === '.' && removeLastZero) //all decimals are zero > remove dot
		{
			removeLastZero = false;
			return;
		}
		if (c !== '0' || !removeLastZero) {
			removeLastZero = false;
			sResult = c + sResult;
		}
	});
	if (exponent !== 0) {
		sResult += "E" + exponent;
	}
	return sResult;
}

export function outputResult(result, sourceIndex) {
	activeDocumentIndex = sourceIndex;
	console.debug(result);
	clearErrorList();
	var strOutput = "";
	var strResult = "";
	try {
		result = JSON.parse(result); //may throw...
		for (let line of result.result) {
			let strLine = lineToOutputString(line);
			if (strLine.length > 0)
				strOutput += strLine + "\n";
		}
		let lineCnt = 0;
		let lineAlreadyFilled = false;
		for (let line of result.result) {
			if (line.src !== sourceIndex)
				continue;
			//goto the next line in output
			while (lineCnt < line.line) {
				strResult += "\n";
				lineCnt++;
				lineAlreadyFilled = false;
			}
			let strValue = linetoResultString(line);
			if (lineAlreadyFilled)
				strResult += " | ";
			strResult += strValue;
			lineAlreadyFilled = true;
		}
        addErrorsToLint(result.errors);

    } catch (e) {
		strOutput = e.message + "\n";
		strOutput += e.name + "\n";
		strOutput += e.stack+ "\n";
		strResult = e.message + "\n";
	}
	let transaction = cm.cmOutput.state.update({ changes: { from: 0, to: cm.cmOutput.state.doc.length, insert: strOutput } });
	cm.cmOutput.update([transaction]);
	transaction = cm.cmResult.state.update({ changes: { from: 0, to: cm.cmResult.state.doc.length, insert: strResult } });
	cm.cmResult.update([transaction]);
}
