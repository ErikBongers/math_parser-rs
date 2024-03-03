import {nodeResolve} from "@rollup/plugin-node-resolve"
import typescript from '@rollup/plugin-typescript';

export default {
  input:  "./lang-mathparser/src/editor.js",
  output: {
    file: "./../dist/editor.bundle.js",
    format: "iife",
    name: "cm"
  },
  plugins: [nodeResolve(), typescript()]
}