import { readFile } from 'fs/promises';

import init, {
  parseRg,
  perfRg,
  runRg,
  serializeRg,
  transformExpandGeneratorNodes,
  transformSkipSelfAssignments,
  validateCheckReachabilities,
} from './module';

// Node.js requires a crypto polyfill. Importing it directly inlines it in the
// browser too, but we don't need it there. Yep, this is a nasty `eval` trick.
if (typeof crypto === 'undefined') {
  eval("globalThis.crypto = require('crypto').webcrypto;");
}

const url = new URL('./module/index_bg.wasm', import.meta.url);
const response = url.protocol === 'file:' ? readFile(url.pathname) : fetch(url);
const initPromise = init(response);
initPromise.catch(console.error);

const methods = {
  parseRg,
  perfRg,
  runRg,
  serializeRg,
  transformExpandGeneratorNodes,
  transformSkipSelfAssignments,
  validateCheckReachabilities,
};

self.addEventListener('message', ({ data }) => {
  initPromise
    .then(() => {
      self.postMessage({
        done: true,
        // @ts-expect-error Check `index.ts` for details.
        // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access -- Check `index.ts` for details.
        value: methods[data.fn](...data.args, (...value) => {
          self.postMessage({ done: false, value });
        }),
      });
    })
    .catch(error => {
      self.postMessage({
        error:
          error instanceof Error
            ? { message: error.message, name: error.name }
            : { message: String(error), name: 'WorkerError' },
      });
    });
});
