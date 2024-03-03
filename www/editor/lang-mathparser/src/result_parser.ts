import {parser} from "./lezer_generated/resultparser.lzr.js"
import {LRLanguage, LanguageSupport} from "@codemirror/language"
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
})

export function resultparser() {
  let lang = resultParserLanguage
  return new LanguageSupport(lang, resultParserLanguage.data.of({}))
}
