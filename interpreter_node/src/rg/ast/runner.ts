import * as ast from './types';
import * as wasm from '../../wasm';

export type Logger = { log: (message: string) => void };

export async function run(
  gameDeclaration: ast.GameDeclaration,
  plays = 1,
  logger: Logger,
) {
  await wasm.runRg(gameDeclaration, plays, lines => lines.forEach(logger.log));
}

export async function perf(
  gameDeclaration: ast.GameDeclaration,
  maxDepth: number,
  logger: Logger,
) {
  for (let depth = 0; depth <= maxDepth; ++depth) {
    await wasm.perfRg(gameDeclaration, depth, (count, time) => {
      logger.log(`perf(depth: ${depth}) = ${count} in ${time}ms`);
    });
  }
}
