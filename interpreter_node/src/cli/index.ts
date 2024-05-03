import { Command, program } from 'commander';
import fs from 'fs/promises';
import path from 'path';

import { AnalyzedGame, parse } from '../parse';
import * as rg from '../rg';
import { Language } from '../types';
import * as utils from '../utils';

program
  .name('node lib/cli')
  .option('--addExplicitCasts', 'add type casts to all expressions')
  .option(
    '--calculateSimpleApply',
    'calculate missing @simpleApply pragmas automatically',
  )
  .option(
    '--calculateTagIndexes',
    'calculate missing @tagIndex and @tagMaxIndex pragmas automatically',
  )
  .option(
    '--calculateUniques',
    'calculate missing @unique pragmas automatically',
  )
  .option('--compactSkipEdges', 'optimize automaton by compacting skip edges')
  .option('--expandGeneratorNodes', 'expand generator nodes')
  .option('--inlineAssignment', 'inline assignment when possible')
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
    '--pruneSingletonTypes',
    'prune singleton types (i.e., Set types with one element)',
  )
  .option('--pruneUnreachableNodes', 'prune unreachable nodes')
  .option(
    '--reuseFunctions',
    'reuse subautomatons when translating function calls (.hrg only)',
  )
  .option(
    '--skipSelfAssignments',
    'replaces all self assignments (e.g., `x = x`) with skip edges',
  )
  .option(
    '--skipSelfComparisons',
    'replaces all self comparisons (e.g., `x == x`) with skip edges',
  )
  .configureHelp({ sortSubcommands: true });

function addCommand(
  name: string,
  description: string,
  operation: (game: AnalyzedGame, ...args: string[]) => void | Promise<void>,
) {
  return program
    .command(name)
    .argument('<file>', 'path to game description file (.hrg, .rbg, or .rg)')
    .description(description)
    .action(async (...input) => {
      const {
        args: [file, ...args],
        parent,
      } = input.pop() as Command;

      const options = parent?.opts() ?? {};
      const extension = path.extname(file).slice(1);
      if (!(Object.values(Language) as string[]).includes(extension)) {
        throw new Error(`Unknown extension "${extension}".`);
      }

      const source = await fs.readFile(file, { encoding: 'utf8' });
      const game = await parse(source, {
        extension: extension as Language,
        flags: {
          addExplicitCasts: !!options.addExplicitCasts,
          calculateSimpleApply: !!options.calculateSimpleApply,
          calculateTagIndexes: !!options.calculateTagIndexes,
          calculateUniques: !!options.calculateUniques,
          compactSkipEdges: !!options.compactSkipEdges,
          expandGeneratorNodes: !!options.expandGeneratorNodes,
          inlineReachability: !!options.inlineReachability,
          inlineAssignment: !!options.inlineAssignment,
          joinForkSuffixes: !!options.joinForkSuffixes,
          mangleSymbols: !!options.mangleSymbols,
          normalizeTypes: !!options.normalizeTypes,
          pruneSingletonTypes: !!options.pruneSingletonTypes,
          pruneUnreachableNodes: !!options.pruneUnreachableNodes,
          reuseFunctions: !!options.reuseFunctions,
          skipSelfAssignments: !!options.skipSelfAssignments,
          skipSelfComparisons: !!options.skipSelfComparisons,
        },
      });

      await operation(game, ...args);

      // Worker keeps the reference, so we have to exit manually.
      process.exit();
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

addCommand('rg-perf', 'run   .rg  tree depth check', async (game, depth) => {
  utils.assert(isFinite(+depth) && +depth > 0, 'depth must be positive');
  utils.assert(game.astRg, 'RG analysis failed');
  await rg.ast.perf(game.astRg, +depth, { log: x => console.log(x) });
}).argument('<depth>', 'maximum tree depth');

addCommand('rg-run', 'run   .rg  simulations', async (game, plays) => {
  utils.assert(isFinite(+plays) && +plays > 0, 'plays must be positive');
  utils.assert(game.astRg, 'RG analysis failed');
  await rg.ast.run(game.astRg, +plays, { log: x => console.log(x) });
}).argument('<plays>', 'number of simulated games');

addCommand('rg-source', 'print .rg  source', game => {
  console.log(game.sourceRgFormatted);
});

program.parse();
