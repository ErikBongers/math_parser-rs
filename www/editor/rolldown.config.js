import { defineConfig } from 'rolldown'

export default defineConfig({
    input:  "./lang-mathparser/src/editor.js",
    output: {
        file: "./../dist/editor.bundle.js",
        format: "iife",
        name: "cm"
    }
})