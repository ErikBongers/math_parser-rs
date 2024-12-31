.PHONY: all
.PHONY: run
.PHONY: release
.PHONY: clean
.PHONY: test
.PHONY: wasm

rust_deps := $(wildcard math_parser/src/*.rs) $(wildcard math_parser/src/*/*.rs) $(wildcard wasm/src/*.rs)
parser_deps := $(wildcard www/typescript/*.ts)
editor_deps := $(wildcard www/editor/lang-mathparser/src/*.ts www/editor/lang-mathparser/src/*.js www/editor/lang-mathparser/src/lezer_generated/*.js)
#wasm is only build when version.no is changed. Thus assuming a bin build is always done prior to a wasm build.
wasm_deps := math_parser/version.no
lezer_grammar := www/editor/lang-mathparser/src/mathparser.grammar
lezer_result_grammar := www/editor/lang-mathparser/src/result.grammar

all_deps := www\editor\lang-mathparser\src\lezer_generated\resultparser.lzr.js
all_deps += www\editor\lang-mathparser\src\lezer_generated\parser.lzr.js
all_deps += www\dist\parser.js
all_deps += www\dist\editor.bundle.js
all_deps += www\dist\wasm_bg.wasm

all: $(all_deps)

www\editor\lang-mathparser\src\lezer_generated\resultparser.lzr.js: $(lezer_result_grammar)
	powershell lezer-generator $(lezer_result_grammar) -o www\editor\lang-mathparser\src\lezer_generated\resultparser.lzr

www\editor\lang-mathparser\src\lezer_generated\parser.lzr.js: $(lezer_grammar)
	powershell lezer-generator $(lezer_grammar) -o www\editor\lang-mathparser\src\lezer_generated\parser.lzr

www\dist\parser.js: $(parser_deps)
	powershell tsc www\typescript\parser.ts -t es2018 --outDir www\dist ; rm www\dist\result.js

www\dist\editor.bundle.js: $(editor_deps)
	powershell cd www\editor ; rollup -c

wasm: www\dist\wasm_bg.wasm

www\dist\wasm_bg.wasm: $(wasm_deps) $(rust_deps)
	 powershell cd wasm ; ./build.ps1

run:
	powershell cd main ; cargo run --package math_parser_main --bin math_parser_main --features print_nodes

release:
	powershell cd main ; cargo run --package math_parser_main --release --bin math_parser_main

deepclean: clean
	powershell cargo clean-recursive

clean:
	powershell try { remove-item www/dist/editor.bundle.js -erroraction stop } catch [System.Management.Automation.ItemNotFoundException] { $$null }
	powershell try { remove-item www/dist/parser.js -erroraction stop } catch [System.Management.Automation.ItemNotFoundException] { $$null }
	powershell try { remove-item www/dist/wasm.js -erroraction stop } catch [System.Management.Automation.ItemNotFoundException] { $$null }
	powershell try { remove-item www/dist/wasm_bg.wasm -erroraction stop } catch [System.Management.Automation.ItemNotFoundException] { $$null }
	powershell try { remove-item wasm/pack/* -erroraction stop } catch [System.Management.Automation.ItemNotFoundException] { $$null }

incversion:
	powershell inc_version math_parser/version.no

test:
	powershell cargo test --workspace
