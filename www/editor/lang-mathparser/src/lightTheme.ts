import {Extension} from "@codemirror/state"
import {HighlightStyle, syntaxHighlighting} from "@codemirror/language"
import {tags} from "@lezer/highlight"
import {EditorView} from "@codemirror/view";
import {svg, underline} from "./themeHelpers";

const lightStyle = HighlightStyle.define([
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

export const lightTheme = EditorView.theme({
  ".cm-lintRange-error": { backgroundImage: /*@__PURE__*/underline("#f88") },
  ".cm-lintRange-warning": { backgroundImage: /*@__PURE__*/underline("#ff8") },
  ".cm-lint-marker-warning": {
    content: /*@__PURE__*/svg(`<path fill="#ff8" stroke="#888" stroke-width="1" stroke-linejoin="round" d="M20 6L37 35L3 35Z"/>`),
  },
  ".cm-lint-marker-error": {
    content: /*@__PURE__*/svg(`<circle cx="20" cy="20" r="13" fill="#f55" stroke="#f55" stroke-width="2"/>`)
  },
}, {dark: false})


export const oneLight: Extension = [lightTheme, syntaxHighlighting(lightStyle)]
