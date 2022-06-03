import { Command, program } from 'commander';
import fs from 'fs';
import path from 'path';

import { parse } from '../parse';
import * as rg from '../rg';
import { Extension } from '../types';
import * as utils from '../utils';

program
  .name('node lib/cli')
  .argument('<file>', 'path to game description file (.hrg or .rg)')
  .option('--compactSkipEdges', 'optimize automaton by compacting skip edges')
  .option('--expandGeneratorNodes', 'expand generator nodes')
  .configureHelp({ sortOptions: true, sortSubcommands: true });

function addCommand(
  name: string,
  description: string,
  operation: (game: ReturnType<typeof parse>, ...args: string[]) => void,
) {
  return program
    .command(name)
    .description(description)
    .action((...input) => {
      const {
        args: [file, ...args],
        parent,
      } = input.pop() as Command;

      const options = parent?.opts() ?? {};
      const extension = path.extname(file);
      if (extension !== Extension.hrg && extension !== Extension.rg) {
        throw new Error(`Unknown extension "${extension}".`);
      }

      const game = parse(fs.readFileSync(file, { encoding: 'utf8' }), {
        extension,
        flags: {
          compactSkipEdges: !!options.compactSkipEdges,
          expandGeneratorNodes: !!options.expandGeneratorNodes,
        },
      });

      operation(game, ...args);
    });
}

addCommand('hrg-ast', 'print high-level Abstract Syntax Tree', game => {
  console.log(JSON.stringify(game.astHrg));
});

addCommand('hrg-cst', 'print high-level Concrete Syntax Tree', game => {
  console.log(JSON.stringify(game.cstHrg));
});

addCommand('hrg-source', 'print high-level source', game => {
  console.log(game.sourceHrg);
});

addCommand('rg-ast', 'print  low-level Abstract Syntax Tree', game => {
  console.log(JSON.stringify(game.astRg));
});

addCommand('rg-cst', 'print  low-level Concrete Syntax Tree', game => {
  console.log(JSON.stringify(game.cstRg));
});

addCommand('rg-ist', 'print  low-level Interpreter State Tree', game => {
  console.log(JSON.stringify(game.istRg));
});

addCommand('rg-perf', 'run    low-level tree depth check', (game, depth) => {
  utils.assert(isFinite(+depth) && +depth > 0, 'depth must be positive');
  // eslint-disable-next-line @typescript-eslint/no-floating-promises -- Node.js will wait automatically.
  rg.ist.perf(game.istRg, +depth, console);
}).argument('<depth>', 'maximum tree depth');

addCommand('rg-run', 'run    low-level simulations', (game, plays) => {
  utils.assert(isFinite(+plays) && +plays > 0, 'plays must be positive');
  // eslint-disable-next-line @typescript-eslint/no-floating-promises -- Node.js will wait automatically.
  rg.ist.run(game.istRg, +plays, console);
}).argument('<plays>', 'number of simulated games');

addCommand('rg-source', 'print  low-level source', game => {
  console.log(game.sourceRg);
});

program.parse();
