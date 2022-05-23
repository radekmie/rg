import BreakthroughHL from 'bundle-text:../../../../examples/breakthrough.hrg';
import BreakthroughLL from 'bundle-text:../../../../examples/breakthrough.rg';
import TicTacToeLL from 'bundle-text:../../../../examples/ticTacToe.rg';

import { Extension } from '../../types';

export const presets = [
  ['Breakthrough (HL)', BreakthroughHL, Extension.hrg] as const,
  ['Breakthrough (LL)', BreakthroughLL, Extension.rg] as const,
  ['TicTacToe (LL)', TicTacToeLL, Extension.rg] as const,
].map(([name, source, extension]) => ({
  name,
  source,
  extension,
}));
