import { Command, program } from 'commander';
import fs from 'fs';
import path from 'path';

import { parse } from '../parse';
import * as rg from '../rg';
import { Extension } from '../types';
import * as utils from '../utils';

program
  .name('node lib/cli')
  .option('--addExplicitCasts', 'add type casts to all expressions')
  .option('--compactSkipEdges', 'optimize automaton by compacting skip edges')
  .option(
    '--expandGeneratorNodes',
    'expand generator nodes (.hrg and .rg only)',
  )
  .option('--inlineReachability', 'inline reachability when possible')
  .option(
    '--joinForkSuffixes',
    'join paths with identical labels leading to the same node',
  )
  .option('--mangleSymbols', 'mangle all user-defined symbols')
  .option(
    '--normalizeTypes',
    'normalize all types so Arrow types appear only in type definitions and are at most one level deep',
  )
  .option(
    '--reuseFunctions',
    'reuse subautomatons when translating function calls (.hrg only)',
  )
  .option(
    '--skipSelfAssignments',
    'replaces all self assignments (e.g., `x = x`) with skip edges',
  )
  .configureHelp({ sortSubcommands: true });

function addCommand(
  name: string,
  description: string,
  operation: (game: ReturnType<typeof parse>, ...args: string[]) => void,
) {
  return program
    .command(name)
    .argument('<file>', 'path to game description file (.hrg, .rbg, or .rg)')
    .description(description)
    .action((...input) => {
      const {
        args: [file, ...args],
        parent,
      } = input.pop() as Command;

      const options = parent?.opts() ?? {};
      const extension = path.extname(file);
      if (
        extension !== Extension.hrg &&
        extension !== Extension.rbg &&
        extension !== Extension.rg
      ) {
        throw new Error(`Unknown extension "${extension}".`);
      }

      const game = parse(fs.readFileSync(file, { encoding: 'utf8' }), {
        extension,
        flags: {
          addExplicitCasts: !!options.addExplicitCasts,
          compactSkipEdges: !!options.compactSkipEdges,
          expandGeneratorNodes: !!options.expandGeneratorNodes,
          inlineReachability: !!options.inlineReachability,
          joinForkSuffixes: !!options.joinForkSuffixes,
          mangleSymbols: !!options.mangleSymbols,
          normalizeTypes: !!options.normalizeTypes,
          reuseFunctions: !!options.reuseFunctions,
          skipSelfAssignments: !!options.skipSelfAssignments,
        },
      });

      operation(game, ...args);
    });
}

addCommand('hrg-ast', 'print .hrg Abstract Syntax Tree', game => {
  console.log(JSON.stringify(game.astHrg));
});

addCommand('hrg-cst', 'print .hrg Concrete Syntax Tree', game => {
  console.log(JSON.stringify(game.cstHrg));
});

addCommand('hrg-source', 'print .hrg source', game => {
  console.log(game.sourceHrgFormatted);
});

addCommand('rbg-ast', 'print .rbg Abstract Syntax Tree', game => {
  console.log(JSON.stringify(game.astRbg));
});

addCommand('rbg-cst', 'print .rbg Concrete Syntax Tree', game => {
  console.log(JSON.stringify(game.cstRbg));
});

addCommand('rbg-source', 'print .rbg source', game => {
  console.log(game.sourceRbgFormatted);
});

addCommand('rg-ast', 'print .rg  Abstract Syntax Tree', game => {
  console.log(JSON.stringify(game.astRg));
});

addCommand('rg-ist', 'print .rg  Interpreter State Tree', game => {
  console.log(JSON.stringify(game.istRg));
});

addCommand('rg-perf', 'run   .rg  tree depth check', (game, depth) => {
  utils.assert(isFinite(+depth) && +depth > 0, 'depth must be positive');
  // eslint-disable-next-line @typescript-eslint/no-floating-promises -- Node.js will wait automatically.
  rg.ist.perf(game.istRg, +depth, console);
}).argument('<depth>', 'maximum tree depth');

addCommand('rg-run', 'run   .rg  simulations', (game, plays) => {
  utils.assert(isFinite(+plays) && +plays > 0, 'plays must be positive');
  // eslint-disable-next-line @typescript-eslint/no-floating-promises -- Node.js will wait automatically.
  rg.ist.run(game.istRg, +plays, console);
}).argument('<plays>', 'number of simulated games');

addCommand('rg-source', 'print .rg  source', game => {
  console.log(game.sourceRgFormatted);
});

program.parse();
