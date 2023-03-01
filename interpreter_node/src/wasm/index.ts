import pLimit from 'p-limit';

import * as rg from '../rg';
import { Settings } from '../types';
import * as utils from '../utils';

// Node.js requires a Worker polyfill.
if (typeof Worker === 'undefined') {
  eval("globalThis.Worker = require('web-worker');");
}

const worker = new Worker(new URL('worker.ts', import.meta.url), {
  type: 'module',
});

const queue = pLimit(1);

type WASM = typeof import('./module');
function workerMethod<Name extends keyof WASM, Progress extends unknown[]>(
  fn: Name,
  // eslint-disable-next-line @typescript-eslint/ban-types -- `Function` is a top type of all functions.
  args: Parameters<WASM[Name]> extends [...infer Args, Function]
    ? Args
    : Parameters<WASM[Name]>,
  onProgress: (...args: Progress) => void,
) {
  return queue(
    () =>
      new Promise<ReturnType<WASM[Name]>>((resolve, reject) => {
        function onError({ error }: ErrorEvent) {
          reject(error);
          worker.removeEventListener('error', onError);
          worker.removeEventListener('message', onMessage);
        }

        function onMessage({
          data,
        }: MessageEvent<
          | { done: false; value: Progress }
          | { done: true; value: ReturnType<WASM[Name]> }
          | { error: { message: string; name: string } }
        >) {
          if ('error' in data) {
            reject(Object.assign(new Error(), data.error));
          } else if (data.done) {
            resolve(data.value);
          } else {
            onProgress(...data.value);
            return;
          }

          worker.removeEventListener('error', onError);
          worker.removeEventListener('message', onMessage);
        }

        worker.addEventListener('error', onError);
        worker.addEventListener('message', onMessage);
        worker.postMessage({ fn, args });
      }),
  );
}

export async function analyzeRg(source: string, flags: Settings['flags']) {
  const [ast, formattedSource] = await workerMethod(
    'analyzeRg',
    // @ts-expect-error: Workaround for passing functions to Web Worker.
    [
      source,
      JSON.stringify(flags),
      '$$TRANSFORMATOR$$joinForkSuffixes' as unknown as () => unknown,
      '$$TRANSFORMATOR$$inlineReachability' as unknown as () => unknown,
      '$$TRANSFORMATOR$$mangleSymbols' as unknown as () => unknown,
    ],
    utils.noop,
  );
  return [JSON.parse(ast), formattedSource] as [rg.ast.GameDeclaration, string];
}

export async function parseRg(source: string) {
  const ast = await workerMethod('parseRg', [source], utils.noop);
  return JSON.parse(ast) as rg.ast.GameDeclaration;
}

export async function perfRg(
  gameDeclaration: rg.ast.GameDeclaration,
  depth: number,
  callback: (count: number) => void,
) {
  const ast = JSON.stringify(gameDeclaration);
  await workerMethod('perfRg', [ast, depth], callback);
}

export async function runRg(
  gameDeclaration: rg.ast.GameDeclaration,
  plays: number,
  callback: (
    plays: number,
    moves: number,
    turns: number,
    goals: string,
  ) => void,
) {
  const ast = JSON.stringify(gameDeclaration);
  await workerMethod('runRg', [ast, plays], callback);
}

export async function serializeRg(gameDeclaration: rg.ast.GameDeclaration) {
  const ast = JSON.stringify(gameDeclaration);
  const rg = await workerMethod('serializeRg', [ast], utils.noop);
  return rg;
}
