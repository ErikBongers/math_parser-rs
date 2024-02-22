import {EditorView, Decoration} from "@codemirror/view"
import {syntaxTree} from "@codemirror/language"
import {ViewUpdate, ViewPlugin, DecorationSet} from "@codemirror/view"

function hideWrapperDecoSet(view: EditorView) {
    let replacements = []
    for (let {from, to} of view.visibleRanges) {
        syntaxTree(view.state).iterate({
            from, to,
            enter: (node) => {
                if (    node.name == "CommentMarkerStart"
                    ||  node.name == "MarkerEnd"
                    ||  node.name == "ErrorMarkerStart"
                    ||  node.name == "WarningMarkerStart"
                    )
                {
                    let deco = Decoration.replace({});
                    replacements.push(deco.range(node.from, node.to))
                }
            }
        })
    }
    return Decoration.set(replacements)
}


export const hideWrapperPlugin = ViewPlugin.fromClass(class {
    decorations: DecorationSet

    constructor(view: EditorView) {
        this.decorations = hideWrapperDecoSet(view)
    }

    update(update: ViewUpdate) {
        if (update.docChanged || update.viewportChanged ||
            syntaxTree(update.startState) != syntaxTree(update.state))
            this.decorations = hideWrapperDecoSet(update.view)
    }
}, {
    decorations: v => v.decorations
})