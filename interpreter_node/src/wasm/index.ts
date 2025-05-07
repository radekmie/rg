import pLimit from 'p-limit';

import { AnalyzedGameStep, RgGameDeclaration } from '../parse';
import { Settings } from '../types';

// Node.js requires a Worker polyfill.
if (typeof Worker === 'undefined') {
  eval("globalThis.Worker = require('web-worker');");
}

const worker = new Worker(new URL('worker.ts', import.meta.url), {
  type: 'module',
});

const queue = pLimit(1);

type WASM = typeof import('./interpreter');
function workerMethod<Name extends keyof WASM, Progress extends unknown[]>(
  fn: Name,
  // eslint-disable-next-line @typescript-eslint/ban-types -- `Function` is a top type of all functions.
  args: Parameters<WASM[Name]> extends [...infer Args, Function]
    ? Args
    : Parameters<WASM[Name]>,
  onProgress?: (...args: Progress) => void,
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
            onProgress?.(...data.value);
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

export async function analyze(source: string, settings: Settings) {
  const steps: AnalyzedGameStep[] = [];
  try {
    await workerMethod(
      'analyze',
      [source, settings.extension, JSON.stringify(settings.flags)],
      (step: string) => {
        steps.push(JSON.parse(step));
      },
    );

    return [steps, null] as const;
  } catch (error) {
    return [steps, error] as const;
  }
}

export async function apply(gameDeclaration: RgGameDeclaration, path: string) {
  const ast = JSON.stringify(gameDeclaration);
  const result = await workerMethod('apply', [ast, path]);
  return JSON.parse(result) as {
    isFinal: boolean;
    moves: string[];
    state: string;
  };
}

export type Logger = { log: (message: string) => void };

export async function perf(
  gameDeclaration: RgGameDeclaration,
  initialStatePath: string,
  maxDepth: number,
  logger: Logger,
) {
  for (let depth = 0; depth <= maxDepth; ++depth) {
    const ast = JSON.stringify(gameDeclaration);
    await workerMethod(
      'perf',
      [ast, initialStatePath, depth],
      (count: number, time: number) => {
        logger.log(`perf(depth: ${depth}) = ${count} in ${time}ms`);
      },
    );
  }
}

export async function run(
  gameDeclaration: RgGameDeclaration,
  initialStatePath: string,
  plays: number,
  logger: Logger,
) {
  const ast = JSON.stringify(gameDeclaration);
  await workerMethod(
    'run',
    [ast, initialStatePath, plays],
    (lines: string[]) => {
      lines.forEach(logger.log);
    },
  );
}
