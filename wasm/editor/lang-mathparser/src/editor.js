import { tags} from "@lezer/highlight"
import { HighlightStyle} from "@codemirror/language"
import { EditorView, highlightSpecialChars, drawSelection, dropCursor, highlightActiveLine, keymap } from '@codemirror/view';
import { EditorState, Compartment } from '@codemirror/state';
import { history, historyKeymap } from '@codemirror/commands';
import { foldGutter, foldKeymap } from '@codemirror/language';
import { lineNumbers, highlightActiveLineGutter } from '@codemirror/view';
import { defaultKeymap } from '@codemirror/commands';
import { bracketMatching, syntaxHighlighting } from '@codemirror/language';
import { closeBracketsKeymap } from '@codemirror/autocomplete';
import { highlightSelectionMatches, searchKeymap } from '@codemirror/search';
import { autocompletion, completionKeymap } from '@codemirror/autocomplete';
import { lintKeymap, linter, lintGutter } from '@codemirror/lint';

import {mathparser} from "./mathparser.ts"
import {mathparserLint} from "./mathparserlint.ts"

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
  syntaxHighlighting(mathParserStyle),
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


export let editor = new EditorView({
  state: EditorState.create({
    extensions: [
      basicSetup, 
      gutter.of([lineNumbers(),
            foldGutter(),
            lintGutter(),]),
      history(),
      autocompletion(),
      mathparser(),   
      linter( mathparserLint(), {delay: 200}),
      fontTheme
    ]
  }),
  parent: document.getElementById("txtInput")
})

export function showGutter(showLineNumbers, showErrors) {
    let gutter = [];
    if (showLineNumbers === true) {
        gutter.push(lineNumbers());
    }
    if (showErrors === true) {
        gutter.push(lintGutter());
    }
    editor.dispatch({
        effects: cm.gutter.reconfigure(gutter)
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
        extensions: [basicSetup, mathparser()]
    }),
    parent: document.getElementById("txtResult")
})
