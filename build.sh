npm --prefix interpreter_node ci
wasm-pack build --out-dir ../../interpreter_node/src/wasm/interpreter_module --out-name index --target web interpreter_rust/interpreter
wasm-pack build --out-dir ../../interpreter_node/src/wasm/lsp_module --out-name index --target web lsp_browser
npm --prefix interpreter_node run build
