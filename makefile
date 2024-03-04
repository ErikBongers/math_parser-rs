.PHONY: all

parser_deps := $(wildcard www/typescript/*.ts)
editor_deps := $(wildcard www/editor/lang-mathparser/src/*.ts www/editor/lang-mathparser/src/*.js)

all: www\dist\parser.js www\dist\editor.bundle.js www\dist\wasm_bg.wasm

www\dist\parser.js: $(parser_deps)
	powershell tsc www\typescript\parser.ts -t es2018 --outDir www\dist ; rm www\dist\result.js

www\dist\editor.bundle.js: $(editor_deps)
	powershell cd www\editor ; rollup -c

www\dist\wasm_bg.wasm:
	 powershell cd wasm ; ./build.ps1