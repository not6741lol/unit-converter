# Build the WASM web app into web/dist.
# Requires: rustup target add wasm32-unknown-unknown
#           cargo install wasm-bindgen-cli --version 0.2.129
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

cargo build --release --target wasm32-unknown-unknown -p convert-web
wasm-bindgen --target web --out-dir "$PSScriptRoot/dist" --out-name convert_web `
    "$root/target/wasm32-unknown-unknown/release/convert_web.wasm"
Copy-Item "$PSScriptRoot/index.html" "$PSScriptRoot/dist/index.html" -Force
New-Item -ItemType File -Path "$PSScriptRoot/dist/.nojekyll" -Force | Out-Null

# Optional: shrink the .wasm with Binaryen's wasm-opt, if it is on PATH.
# https://github.com/WebAssembly/binaryen/releases  (version 133)
$wasmOpt = Get-Command wasm-opt -ErrorAction SilentlyContinue
if ($wasmOpt) {
    $target = "$PSScriptRoot/dist/convert_web_bg.wasm"
    & $wasmOpt.Source `
        --enable-bulk-memory --enable-bulk-memory-opt `
        --enable-nontrapping-float-to-int --enable-reference-types `
        --enable-mutable-globals `
        -Oz $target -o "$target.opt"
    Move-Item "$target.opt" $target -Force
    Write-Host "Optimized wasm with wasm-opt"
}
else {
    Write-Host "wasm-opt not found - skipping optimization (install Binaryen to shrink the .wasm)"
}

Write-Host "Built web/dist"
