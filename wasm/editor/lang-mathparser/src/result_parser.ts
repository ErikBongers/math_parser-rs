import {parser} from "./resultparser.js"
import {LRLanguage, LanguageSupport, foldNodeProp, foldInside} from "@codemirror/language"
import {styleTags, tags as t} from "@lezer/highlight"

export const resultParserLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      styleTags({
        BooleanLiteral: t.bool,
        Number: t.number,
        String: t.string,
        Equals: t.definitionOperator,

        ErrorMarkerStart: t.meta,
        CommentMarkerStart: t.meta,
        WarningMarkerStart: t.meta,
        MarkerEnd: t.meta,

        ErrorMarkerText: t.invalid,
        WarningMarkerText: t.annotation,
        CommentMarkerText: t.comment,
      })
    ]
  }),
  languageData: {
    closeBrackets: {brackets: ["(", "[", "{", "'", '"', "`"]},
    commentTokens: {line: "//", block: {open: "/*", close: "*/"}},
    indentOnInput: /^\s*(?:case |default:|\{|\}|<\/)$/,
    wordChars: "$"
  }
})

export function resultparser() {
  let lang = resultParserLanguage
  return new LanguageSupport(lang, resultParserLanguage.data.of({}))
}
