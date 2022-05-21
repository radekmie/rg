import fs from 'fs';
import path from 'path';

import { openGame } from './io';
import { perf, run } from './run';
import { Extension, Optimize } from './types';
import * as utils from './utils';

function read(file: string) {
  return fs.readFileSync(file, { encoding: 'utf8' });
}

const file = process.argv[2];
const game = openGame(read(file), {
  extension: path.extname(file) as Extension,
  optimize: Optimize.yes,
});

switch (process.argv[3]) {
  case 'perf': {
    const maxDepth = +process.argv[4];
    utils.assert(isFinite(maxDepth) && maxDepth > 0, 'depth must be positive');
    // eslint-disable-next-line @typescript-eslint/no-floating-promises -- Node.js will wait automatically.
    perf(game.ist, maxDepth, console);
    break;
  }
  case 'print-ast':
    console.log(JSON.stringify(game.ast));
    break;
  case 'print-cst':
    console.log(JSON.stringify(game.cst));
    break;
  case 'print-graphviz':
    console.log(game.graphviz);
    break;
  case 'print-ist':
    console.log(JSON.stringify(game.ist));
    break;
  case 'print-source-hl':
    console.log(game.source.hl);
    break;
  case 'print-source-ll':
    console.log(game.source.ll);
    break;
  case 'run': {
    const plays = +process.argv[4];
    utils.assert(isFinite(plays) && plays > 0, 'plays must be positive');
    // eslint-disable-next-line @typescript-eslint/no-floating-promises -- Node.js will wait automatically.
    run(game.ist, plays, console);
    break;
  }
}
