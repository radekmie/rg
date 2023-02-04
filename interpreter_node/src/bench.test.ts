import fs from 'fs';
import path from 'path';
import { describe, test } from 'vitest';

import { parse } from './parse';
import { Extension, Flag, noFlagsEnabled } from './types';
import * as utils from './utils';

// These are _really_ heavy.
describe.skip('bench', () => {
  const examplesPath = path.join(__dirname, '..', '..', 'examples');
  const examples = fs
    .readdirSync(examplesPath)
    .sort()
    .flatMap(fileName => {
      const filePath = path.join(examplesPath, fileName);
      const extension = path.extname(filePath);
      if (
        extension !== Extension.hrg &&
        extension !== Extension.rbg &&
        extension !== Extension.rg
      ) {
        return [];
      }

      const source = fs.readFileSync(filePath, { encoding: 'utf8' });
      return [{ extension, fileName, source }];
    });

  const flagNames = Object.keys(noFlagsEnabled) as Flag[];
  for (const flagBits of flagNames
    .map(() => [false, true])
    .reduce<boolean[][]>(utils.cartesian, [[]])) {
    const flagsSelected = flagNames.filter((flag, index) => flagBits[index]);
    const flags = flagsSelected.reduce(
      (flags, flag) => {
        flags[flag] = true;
        return flags;
      },
      { ...noFlagsEnabled },
    );

    for (const { extension, fileName, source } of examples) {
      const args = flagsSelected.map(flag => `--${flag}`).join(' ');
      test(`${fileName} ${args}`, () => {
        parse(source, { extension, flags });
      });
    }
  }
});
