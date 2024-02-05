import {EditorView} from "@codemirror/view"
import {Extension} from "@codemirror/state"
import {HighlightStyle, syntaxHighlighting} from "@codemirror/language"
import {tags, tags as t} from "@lezer/highlight"
import {svg, underline} from "./themeHelpers";

// Using https://github.com/one-dark/vscode-one-dark-theme/ as reference for the colors

const chalky = "#aba18f",
  coral = "#a6676c",
  cyan = "#56b6c2",
  invalid = "#ffffff",
  ivory = "#abb2bf",
  stone = "#727883", // Brightened compared to original to increase contrast
  malibu = "#6292b7",
  sage = "#98c379",
  whiskey = "#9a8776",
  violet = "#c678dd",
  darkBackground = "#202025",
  highlightBackground = "#2c313a",
  background = "#25252b",
  tooltipBackground = "#353a42",
  selection = "#3E4451",
  cursor = "#528bff"

/// The colors used in the theme, as CSS color strings.
export const color = {
  chalky,
  coral,
  cyan,
  invalid,
  ivory,
  stone,
  malibu,
  sage,
  whiskey,
  violet,
  darkBackground,
  highlightBackground,
  background,
  tooltipBackground,
  selection,
  cursor
}

/// The editor theme styles for One Dark.
export const oneDarkTheme = EditorView.theme({
  "&": {
    color: ivory,
    backgroundColor: background
  },

  ".cm-content": {
    caretColor: cursor
  },

  ".cm-cursor, .cm-dropCursor": {borderLeftColor: cursor},
  "&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection": {backgroundColor: selection},

  ".cm-panels": {backgroundColor: darkBackground, color: ivory},
  ".cm-panels.cm-panels-top": {borderBottom: "2px solid black"},
  ".cm-panels.cm-panels-bottom": {borderTop: "2px solid black"},

  ".cm-searchMatch": {
    backgroundColor: "#72a1ff59",
    outline: "1px solid #457dff"
  },
  ".cm-searchMatch.cm-searchMatch-selected": {
    backgroundColor: "#6199ff2f"
  },

  ".cm-activeLine": {backgroundColor: "#6699ff0b"},
  ".cm-selectionMatch": {backgroundColor: "#aafe661a"},

  "&.cm-focused .cm-matchingBracket, &.cm-focused .cm-nonmatchingBracket": {
    backgroundColor: "#bad0f847"
  },

  ".cm-gutters": {
    backgroundColor: background,
    color: stone,
    border: "none"
  },

  ".cm-activeLineGutter": {
    backgroundColor: highlightBackground
  },

  ".cm-foldPlaceholder": {
    backgroundColor: "transparent",
    border: "none",
    color: "#ddd"
  },
  ".cm-lintRange-error": { backgroundImage: /*@__PURE__*/underline("#f88") },
  ".cm-lintRange-warning": { backgroundImage: /*@__PURE__*/underline("#ff8") },
  ".cm-lint-marker-warning": {
    content: /*@__PURE__*/svg(`<path fill="#ff8" stroke="#ff8" stroke-width="0" stroke-linejoin="round" d="M20 6L37 35L3 35Z"/>`),
  },
  ".cm-lint-marker-error": {
    content: /*@__PURE__*/svg(`<circle cx="20" cy="20" r="13" fill="#f55" stroke="#f55" stroke-width="2"/>`)
  },

  ".cm-tooltip": {
    border: "none",
    backgroundColor: tooltipBackground
  },
  ".cm-tooltip .cm-tooltip-arrow:before": {
    borderTopColor: "transparent",
    borderBottomColor: "transparent"
  },
  ".cm-tooltip .cm-tooltip-arrow:after": {
    borderTopColor: tooltipBackground,
    borderBottomColor: tooltipBackground
  },
  ".cm-tooltip-autocomplete": {
    "& > ul > li[aria-selected]": {
      backgroundColor: highlightBackground,
      color: ivory
    }
  }
}, {dark: true})

/// The highlighting style for code in the One Dark theme.
export const oneDarkHighlightStyle = HighlightStyle.define([
  {tag: t.keyword, color: violet},
  {tag: [t.name, t.deleted, t.character, t.propertyName, t.macroName], color: ivory},
  {tag: [t.function(t.variableName), t.labelName], color: malibu},
  {tag: [t.color, t.constant(t.name), t.standard(t.name)], color: whiskey},
  {tag: [t.definition(t.name), t.separator], color: ivory},
  {tag: [t.typeName, t.className, t.number, t.changed, t.annotation, t.modifier, t.self, t.namespace], color: chalky},
  {tag: [t.operator, t.operatorKeyword, t.url, t.escape, t.regexp, t.link, t.special(t.string)], color: cyan},
  {tag: [t.meta, t.comment], color: stone},
  {tag: t.strong, fontWeight: "bold"},
  {tag: t.emphasis, fontStyle: "italic"},
  {tag: t.strikethrough, textDecoration: "line-through"},
  {tag: t.link, color: stone,  textDecoration: "underline"},
  {tag: t.heading, fontWeight: "bold", color: coral},
  {tag: [t.atom, t.bool, t.special(t.variableName)], color: whiskey },
  {tag: [t.processingInstruction, t.inserted], color: stone},
  {tag: t.string, color: malibu},
  {tag: t.invalid,  color: invalid},
  { tag: tags.definitionKeyword, color: "#6d9a6d" },

])

/// Extension to enable the One Dark theme (both the editor theme and
/// the highlight style).
export const oneDark: Extension = [oneDarkTheme, syntaxHighlighting(oneDarkHighlightStyle)]