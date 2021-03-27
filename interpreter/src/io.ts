import fs from 'fs';
import path from 'path';

import buildAST from './ast';
import parse from './cst';
import translate from './down-level';
import buildIST from './ist';

function read(file: string) {
  return fs.readFileSync(file, { encoding: 'utf8' });
}

function analyze(file: string) {
  const extension = path.extname(file);
  switch (extension) {
    case '.hrg': {
      const hl = read(file);
      const ll = translate(hl);
      return { hl, ll };
    }
    case '.rg': {
      const ll = read(file);
      return { hl: null, ll };
    }
    default:
      throw new Error(`Unknown extension "${extension}".`);
  }
}

export default function openGame(file: string) {
  const source = analyze(file);
  const cst = parse(source.ll);
  const ast = buildAST(cst);
  const ist = buildIST(ast);
  return { ast, cst, ist, source };
}
