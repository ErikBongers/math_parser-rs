.PHONY: all
.PHONY: run
.PHONY: release
.PHONY: clean
.PHONY: test

parser_deps := $(wildcard www/typescript/*.ts)
editor_deps := $(wildcard www/editor/lang-mathparser/src/*.ts www/editor/lang-mathparser/src/*.js)
#wasm is only build when version.no is changed. Thus assuming a bin build is always done prior to a wasm build.
wasm_deps := math_parser/version.no

all: www\dist\parser.js www\dist\editor.bundle.js www\dist\wasm_bg.wasm

www\dist\parser.js: $(parser_deps)
	powershell tsc www\typescript\parser.ts -t es2018 --outDir www\dist ; rm www\dist\result.js

www\dist\editor.bundle.js: $(editor_deps)
	powershell cd www\editor ; rollup -c

www\dist\wasm_bg.wasm: $(wasm_deps)
	 powershell cd wasm ; ./build.ps1

run:
	powershell cd main ; cargo run --package math_parser_main --bin math_parser_main --features print_nodes

release:
	powershell cd main ; cargo run --package math_parser_main --release --bin math_parser_main

clean:
	powershell cargo clean-recursive

test:
	powershell cargo test --workspace
