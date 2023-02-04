import * as wasm from './src/wasm';

export function setup() {
  return wasm.initPromise;
}

export function teardown() {}
