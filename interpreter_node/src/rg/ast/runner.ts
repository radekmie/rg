import * as ast from './types';
import * as wasm from '../../wasm';

export type Logger = { log: (message: string) => void };

function format(number: number) {
  return number.toFixed(3);
}

export async function run(
  gameDeclaration: ast.GameDeclaration,
  plays = 1,
  logger: Logger,
) {
  const now = performance.now();
  await wasm.runRg(gameDeclaration, plays, (plays, moves, turns, goals) => {
    const end = performance.now();
    logger.log(`after ${plays} plays:`);
    logger.log(`  avg. moves: ${format(moves)}`);
    logger.log(`  avg. turns: ${format(turns)}`);
    logger.log(`  avg. times: ${format((end - now) / plays)}ms`);
    logger.log('  avg. scores:');
    goals.split('\n').forEach(logger.log);
  });
}

export async function perf(
  gameDeclaration: ast.GameDeclaration,
  maxDepth: number,
  logger: Logger,
) {
  for (let depth = 0; depth <= maxDepth; ++depth) {
    const now = performance.now();
    await wasm.perfRg(gameDeclaration, depth, count => {
      const end = performance.now();
      logger.log(`perf(depth: ${depth}) = ${count} in ${format(end - now)}ms`);
    });
  }
}
