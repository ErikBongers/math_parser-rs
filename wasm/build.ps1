wasm-pack.exe build --release --target web --out-dir pack -- --color=always
if ($?)
{
    Copy-Item ".\pack\wasm.js" -Destination "..\www\dist\wasm.js"
    Copy-Item ".\pack\wasm_bg.wasm" -Destination "..\www\dist\wasm_bg.wasm"
}