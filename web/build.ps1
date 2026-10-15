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
Write-Host "Built web/dist"
