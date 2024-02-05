import { tags} from "@lezer/highlight"
import { HighlightStyle} from "@codemirror/language"
import { EditorView, highlightSpecialChars, drawSelection, dropCursor, highlightActiveLine, keymap } from '@codemirror/view';
import { EditorState, Compartment } from '@codemirror/state';
import { history, historyKeymap } from '@codemirror/commands';
import { foldGutter, foldKeymap } from '@codemirror/language';
import { lineNumbers, highlightActiveLineGutter } from '@codemirror/view';
import { defaultKeymap } from '@codemirror/commands';
import { bracketMatching } from '@codemirror/language';
import { closeBracketsKeymap } from '@codemirror/autocomplete';
import { highlightSelectionMatches, searchKeymap } from '@codemirror/search';
import { autocompletion, completionKeymap } from '@codemirror/autocomplete';
import { lintKeymap, linter, lintGutter } from '@codemirror/lint';

import {mathparser} from "./mathparser.ts"
import {mathparserLint} from "./mathparserlint.ts"

import { oneDark } from "./darkTheme.js"
import {oneLight} from "./lightTheme.js";

export {setLintSource} from "./mathparserlint";


let mathParserStyle = HighlightStyle.define([
  { tag: tags.strong, fontWeight: "bold" },
  { tag: tags.keyword, color: "#708" },
  { tag: [tags.atom, tags.bool, tags.url, tags.contentSeparator, tags.labelName], color: "#219" },
  { tag: [tags.literal, tags.inserted], color: "#164" },
  { tag: [tags.string, tags.deleted], color: "#446" },
  { tag: [tags.regexp, tags.escape, tags.special(tags.string)], color: "#e40" },
  { tag: tags.definition(tags.variableName), color: "#00f" },
  { tag: tags.local(tags.variableName), color: "#30a" },
  { tag: [tags.typeName, tags.namespace], color: "#085" },
  { tag: tags.className, color: "#167" },
  { tag: [tags.special(tags.variableName), tags.macroName], color: "#256" },
  { tag: tags.definition(tags.propertyName), color: "#00c" },
  { tag: tags.comment, color: "#666" },
  { tag: tags.meta, color: "#7a757a" },
  { tag: tags.invalid, color: "#f00" },
  { tag: tags.definitionKeyword, color: "#464" },

  //custom styles:
  {tag: tags.processingInstruction, color: "#bbb"}
]);

const basicSetup = [
  highlightActiveLineGutter(),
  highlightSpecialChars(),
  drawSelection(),
  dropCursor(),
  EditorState.allowMultipleSelections.of(true),
  bracketMatching(),
  highlightActiveLine(),
  highlightSelectionMatches(),
  keymap.of([
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...completionKeymap,
      ...lintKeymap
  ])
];

const fontTheme = EditorView.theme({
    "&": {
        fontFamily: "monospace"
    }
});

export let gutter = new Compartment;
export let editorTheme = new Compartment;
export let resultTheme = new Compartment;
export let undoRedo = new Compartment; //TODO: need to be exported?


export let editor = new EditorView({
    state: EditorState.create({
        extensions: [
            basicSetup,
            gutter.of([lineNumbers(),
                foldGutter(),
                lintGutter(),]),
            autocompletion(),
            mathparser(),
            linter( mathparserLint(), {delay: 200}),
            fontTheme,
            editorTheme.of([]),
            undoRedo.of([history()]),
        ]
    }),
    parent: document.getElementById("txtInput")
})

export function resetUndoRedo() {
    editor.dispatch({
        effects: undoRedo.reconfigure([])
    });
    editor.dispatch({
        effects: undoRedo.reconfigure([history()])
    });
}

export function setEditorText(text) {
    let transaction = editor.state.update({ changes: { from: 0, to: editor.state.doc.length, insert: text} });
    editor.update([transaction]);
    resetUndoRedo();
}

export function showGutter(showLineNumbers, showErrors) {
    let gutters = [];
    if (showLineNumbers === true) {
        gutters.push(lineNumbers());
    }
    if (showErrors === true) {
        gutters.push(lintGutter());
    }
    editor.dispatch({
        effects: gutter.reconfigure(gutters)
    });
}

export function setDarkTheme(dark) {
    let theme1 = [];
    theme1.push(dark === true ? oneDark : oneLight)
    editor.dispatch({
        effects: editorTheme.reconfigure(theme1)
    });
    let theme2 = [];
    theme2.push(dark === true ? oneDark : oneLight)
    cmResult.dispatch({
        effects: resultTheme.reconfigure(theme2)
    });
}

export let cmOutput = new EditorView({
  state: EditorState.create({
    extensions: [basicSetup, mathparser()]
  }),
  parent: document.getElementById("txtOutput")
})

export let cmResult = new EditorView({
    state: EditorState.create({
        extensions: [basicSetup,
            mathparser(),
            EditorState.readOnly.of(true),
            resultTheme.of([])
        ]
    }),
    parent: document.getElementById("txtResult")
})
