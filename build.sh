npm --prefix interpreter_node ci
wasm-pack build --out-dir ../../interpreter_node/src/wasm/module --out-name index --target web interpreter_rust/interpreter
npm --prefix interpreter_node run build
