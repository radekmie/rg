import { readFileSync } from 'fs';

import { initSync, parseRg, perfRg, runRg } from './module';

// Node.js requires a crypto polyfill. Importing it directly inlines it in the
// browser too, but we don't need it there. Yep, this is a nasty `eval` trick.
if (typeof crypto === 'undefined') {
  eval("globalThis.crypto = require('crypto').webcrypto;");
}

// WASM module is inlined in the browser and referenced in the CLI version.
initSync(readFileSync(__dirname + '/module/index_bg.wasm'));

const methods = { parseRg, perfRg, runRg };
self.addEventListener('message', ({ data }) => {
  try {
    self.postMessage({
      done: true,
      // @ts-expect-error Check `index.ts` for details.
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access -- Check `index.ts` for details.
      value: methods[data.fn](...data.args, (...value) => {
        self.postMessage({ done: false, value });
      }),
    });
  } catch (error) {
    self.postMessage({ error });
  }
});
