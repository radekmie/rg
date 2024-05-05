import AmazonsNaiveHrg from 'bundle-text:../../../../examples/amazons-naive.hrg';
import AmazonsSmartHrg from 'bundle-text:../../../../examples/amazons-smart.hrg';
import AmazonsRbg from 'bundle-text:../../../../examples/amazons.rbg';
import BreakthroughHrg from 'bundle-text:../../../../examples/breakthrough.hrg';
import BreakthroughRbg from 'bundle-text:../../../../examples/breakthrough.rbg';
import BreakthroughRg from 'bundle-text:../../../../examples/breakthrough.rg';
import Connect4Hrg from 'bundle-text:../../../../examples/connect4.hrg';
import Connect4Gdl from 'bundle-text:../../../../examples/connect4.kif';
import Connect4Rbg from 'bundle-text:../../../../examples/connect4.rbg';
import Hex2Rbg from 'bundle-text:../../../../examples/hex2.rbg';
import Hex9Rbg from 'bundle-text:../../../../examples/hex9.rbg';
import KnightthroughHrg from 'bundle-text:../../../../examples/knightthrough.hrg';
import KnightthroughRbg from 'bundle-text:../../../../examples/knightthrough.rbg';
import TicTacToeGdl from 'bundle-text:../../../../examples/ticTacToe.kif';
import TicTacToeRbg from 'bundle-text:../../../../examples/ticTacToe.rbg';
import TicTacToeRg from 'bundle-text:../../../../examples/ticTacToe.rg';

import { Language } from '../../types';

export const presets = [
  ['Amazons (naive)', AmazonsNaiveHrg, Language.hrg] as const,
  ['Amazons (smart)', AmazonsSmartHrg, Language.hrg] as const,
  ['Amazons', AmazonsRbg, Language.rbg] as const,
  ['Breakthrough', BreakthroughHrg, Language.hrg] as const,
  ['Breakthrough', BreakthroughRbg, Language.rbg] as const,
  ['Breakthrough', BreakthroughRg, Language.rg] as const,
  ['Connect4', Connect4Hrg, Language.hrg] as const,
  ['Connect4', Connect4Gdl, Language.gdl] as const,
  ['Connect4', Connect4Rbg, Language.rbg] as const,
  ['Hex2', Hex2Rbg, Language.rbg] as const,
  ['Hex9', Hex9Rbg, Language.rbg] as const,
  ['Knightthrough', KnightthroughHrg, Language.hrg] as const,
  ['Knightthrough', KnightthroughRbg, Language.rbg] as const,
  ['TicTacToe', TicTacToeGdl, Language.gdl] as const,
  ['TicTacToe', TicTacToeRbg, Language.rbg] as const,
  ['TicTacToe', TicTacToeRg, Language.rg] as const,
].map(([name, source, extension]) => ({
  name: `${name}.${extension}`,
  source,
  extension,
}));
