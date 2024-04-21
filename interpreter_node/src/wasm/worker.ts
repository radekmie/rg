import { readFile } from 'fs/promises';

import init, {
  analyzeRg,
  parseGdl,
  parseHrg,
  perfRg,
  runRg,
  serializeRg,
} from './interpreter';
import * as transformators from '../rg/transformators';

// Node.js requires a crypto polyfill. Importing it directly inlines it in the
// browser too, but we don't need it there. Yep, this is a nasty `eval` trick.
if (typeof crypto === 'undefined') {
  eval("globalThis.crypto = require('crypto').webcrypto;");
}

const url = new URL('./interpreter/index_bg.wasm', import.meta.url);
const response = url.protocol === 'file:' ? readFile(url.pathname) : fetch(url);
const initPromise = init(response);
initPromise.catch(console.error);

// It's a temporary workaround for passing functions to Web Worker.
function reify(arg: unknown) {
  const header = '$$TRANSFORMATOR$$';
  if (typeof arg === 'string' && arg.startsWith(header)) {
    const key = arg.replace(header, '');
    if (key in transformators) {
      const transformator = transformators[key as keyof typeof transformators];
      return (ast: string) => {
        const game = JSON.parse(ast);
        transformator(game);
        return JSON.stringify(game);
      };
    }
  }

  return arg;
}

const methods = {
  analyzeRg,
  parseGdl,
  parseHrg,
  perfRg,
  runRg,
  serializeRg,
};
self.addEventListener('message', ({ data }) => {
  initPromise
    .then(() => {
      self.postMessage({
        done: true,
        // @ts-expect-error Check `index.ts` for details.
        // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access -- Check `index.ts` for details.
        value: methods[data.fn](...data.args.map(reify), (...value) => {
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
