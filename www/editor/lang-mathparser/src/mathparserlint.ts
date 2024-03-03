import {Diagnostic} from "@codemirror/lint" 
import {Text} from "@codemirror/state"
import {EditorView} from "@codemirror/view"

let _lintSource: (view: EditorView) => Diagnostic[] = undefined;

export function setLintSource(lintSource: (view: EditorView) => Diagnostic[]){
    _lintSource = lintSource;
}

export function mathparserLint(): (view: EditorView) => Diagnostic[] {
//return a function that returns a Diagnosic[] array, containing all the errors for the given view.
//NOTE: this can return a Promise<readonly Diagnostic[]> !!!
    return (view: EditorView) => {
        if(_lintSource)
            return _lintSource(view);
        else
            return [];
    };
}

function mapPos(line: number, col: number, doc: Text, offset: {line: number, col: number, pos: number}) {
  return doc.line(line + offset.line).from + col + (line == 1 ? offset.col - 1 : -1);
}
