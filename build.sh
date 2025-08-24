npm --prefix interpreter_node ci
wasm-pack build --out-dir ../../interpreter_node/src/wasm/cli --out-name index --target web interpreter_rust/cli
wasm-pack build --out-dir ../../interpreter_node/src/wasm/lsp --out-name index --target web interpreter_rust/lsp_browser
npm --prefix interpreter_node run build
