import fs from 'fs';
import path from 'path';
import { describe, expect, test } from 'vitest';

import { parse } from './parse';
import { Flag, Language, noFlagsEnabled } from './types';
import * as wasm from './wasm';

describe('bench', () => {
  const examples = Object.entries({
    'amazons-naive.hrg': [1, 2176], //, 4307152],
    'amazons-smart.hrg': [1, 2176], //, 4307152],
    'breakthrough.hrg': [1, 22, 484, 11132], //, 256036],
    'breakthrough.rbg': [1, 22, 484, 11132], //, 256036],
    'breakthrough.rg': [1, 22, 484, 11132], //, 256036],
    'connect4.hrg': [1, 7, 49, 343, 2401], //, 16807, 117649, 823536, 5673234],
    'hex2.rbg': [1, 4, 12, 24, 12, 0],
    'hex9.rbg': [1, 81, 6480], //, 511920, 39929760],
    'ticTacToe.rbg': [1, 9, 72, 504, 3024, 15120], //, 54720, 148176, 200448, 127872],
    'ticTacToe.rg': [1, 9, 72, 504, 3024, 15120], //, 54720, 148176, 200448, 127872],
  }).map(([fileName, counts]) => {
    const filePath = path.join(__dirname, '..', '..', 'examples', fileName);
    const extension = path.extname(filePath).slice(1) as Language;
    const source = fs.readFileSync(filePath, { encoding: 'utf8' });
    return { counts, extension, fileName, source };
  });

  for (const { counts, extension, fileName, source } of examples) {
    const flagNames = Object.keys(noFlagsEnabled) as Flag[];
    const flagSets =
      // Translation layers are heavy, so we leave them limited for now.
      extension === Language.rg
        ? // No flags, all flags separately, and all flags.
          [[], ...flagNames.map(flagName => [flagName]), flagNames]
        : // All flags.
          [flagNames];

    describe(fileName, () => {
      for (const flagSet of flagSets) {
        test.concurrent(flagSet.join(' ') || '(no flags)', async () => {
          const { astRg } = await parse(source, {
            extension,
            flags: flagSet.reduce((flags, flag) => {
              flags[flag] = true;
              return flags;
            }, noFlagsEnabled),
          });

          for (let depth = 0; depth < counts.length; ++depth) {
            let count = -1;
            await wasm.perfRg(astRg, depth, n => (count = n));
            expect(count).toBe(counts[depth]);
          }
        });
      }
    });
  }
});
