import { readFileSync } from 'fs';

// If you see an error here, make sure to build the Rust module first!
import init from './wasm-module';
export * from './wasm-module';

// Node.js requires a crypto polyfill. Importing it directly inlines it in the
// browser too, but we don't need it there.
if (typeof window === 'undefined') {
  eval("globalThis.crypto = require('crypto').webcrypto;");
}

// WASM module is inlined in the browser and referenced in the CLI version.
const buffer = readFileSync(__dirname + '/wasm-module/index_bg.wasm');
export const initPromise = init(buffer);
