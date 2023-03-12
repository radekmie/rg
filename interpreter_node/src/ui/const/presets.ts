import AmazonsNaiveHrg from 'bundle-text:../../../../examples/amazons-naive.hrg';
import AmazonsSmartHrg from 'bundle-text:../../../../examples/amazons-smart.hrg';
import BreakthroughHrg from 'bundle-text:../../../../examples/breakthrough.hrg';
import BreakthroughRbg from 'bundle-text:../../../../examples/breakthrough.rbg';
import BreakthroughRg from 'bundle-text:../../../../examples/breakthrough.rg';
import BreakthroughWithAnyRg from 'bundle-text:../../../../examples/breakthroughWithAny.rg';
import Connect4Hrg from 'bundle-text:../../../../examples/connect4.hrg';
import Hex2Rbg from 'bundle-text:../../../../examples/hex2.rbg';
import Hex9Rbg from 'bundle-text:../../../../examples/hex9.rbg';
import TicTacToeRbg from 'bundle-text:../../../../examples/ticTacToe.rbg';
import TicTacToeRg from 'bundle-text:../../../../examples/ticTacToe.rg';

import { Extension } from '../../types';

export const presets = [
  ['Amazons (naive)', AmazonsNaiveHrg, Extension.hrg] as const,
  ['Amazons (smart)', AmazonsSmartHrg, Extension.hrg] as const,
  ['Breakthrough', BreakthroughHrg, Extension.hrg] as const,
  ['Breakthrough', BreakthroughRbg, Extension.rbg] as const,
  ['Breakthrough', BreakthroughRg, Extension.rg] as const,
  ['Breakthrough (with any)', BreakthroughWithAnyRg, Extension.rg] as const,
  ['Connect4', Connect4Hrg, Extension.hrg] as const,
  ['Hex2', Hex2Rbg, Extension.rbg] as const,
  ['Hex9', Hex9Rbg, Extension.rbg] as const,
  ['TicTacToe', TicTacToeRbg, Extension.rbg] as const,
  ['TicTacToe', TicTacToeRg, Extension.rg] as const,
].map(([name, source, extension]) => ({
  name: `${name}${extension}`,
  source,
  extension,
}));
