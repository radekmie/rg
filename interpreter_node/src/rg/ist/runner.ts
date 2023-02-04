import * as wasm from '../../wasm';
import * as ist from './types';

export type Logger = { log: (message: string) => void };

function format(number: number) {
  return number.toFixed(3);
}

export function run(game: ist.Game, plays = 1, logger: Logger) {
  const now = performance.now();
  wasm.run_rg(
    JSON.stringify(game),
    plays,
    (plays: number, moves: number, turns: number, goals: string) => {
      const end = performance.now();
      logger.log(`after ${plays} plays:`);
      logger.log(`  avg. moves: ${format(moves)}`);
      logger.log(`  avg. turns: ${format(turns)}`);
      logger.log(`  avg. times: ${format((end - now) / plays)}ms`);
      logger.log('  avg. scores:');
      goals.split('\n').forEach(logger.log);
    },
  );
}

export function perf(game: ist.Game, maxDepth: number, logger: Logger) {
  for (let depth = 0; depth <= maxDepth; ++depth) {
    const now = performance.now();
    wasm.perf_rg(JSON.stringify(game), depth, (count: number) => {
      const end = performance.now();
      logger.log(`perf(depth: ${depth}) = ${count} in ${format(end - now)}ms`);
    });
  }
}
