import fs from 'fs';

import buildAST from './ast';
import parse from './cst';
import buildIST from './ist';

export default function openGame(path: string) {
  const gameSource = fs.readFileSync(path, { encoding: 'utf8' });
  const cst = parse(gameSource);
  const ast = buildAST(cst);
  const ist = buildIST(ast);
  return { ast, cst, ist };
}
