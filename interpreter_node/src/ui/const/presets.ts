import AmazonsNaiveHrg from 'bundle-text:../../../../games/hrg/amazons-naive.hrg';
import AmazonsSmartHrg from 'bundle-text:../../../../games/hrg/amazons-smart.hrg';
import BreakthroughHrg from 'bundle-text:../../../../games/hrg/breakthrough.hrg';
import Connect4Hrg from 'bundle-text:../../../../games/hrg/connect4.hrg';
import DiceThrowCompareHrg from 'bundle-text:../../../../games/hrg/dice-throw-compare.hrg';
import DiceThrowGuessHrg from 'bundle-text:../../../../games/hrg/dice-throw-guess.hrg';
import GomokuFreestyleHrg from 'bundle-text:../../../../games/hrg/gomoku-freestyle.hrg';
import KnightthroughHrg from 'bundle-text:../../../../games/hrg/knightthrough.hrg';
import SatSolverHrg from 'bundle-text:../../../../games/hrg/sat-solver.hrg';
import ShortestPathHrg from 'bundle-text:../../../../games/hrg/shortest-path.hrg';
import TicTacToeHrg from 'bundle-text:../../../../games/hrg/ticTacToe.hrg';
import Connect4Gdl from 'bundle-text:../../../../games/kif/connect4.kif';
import MontyHallGdl from 'bundle-text:../../../../games/kif/monty-hall.kif';
import TicTacToeGdl from 'bundle-text:../../../../games/kif/ticTacToe.kif';
import AmazonsRbg from 'bundle-text:../../../../games/rbg/amazons.rbg';
import BreakthroughRbg from 'bundle-text:../../../../games/rbg/breakthrough.rbg';
import Connect4Rbg from 'bundle-text:../../../../games/rbg/connect4.rbg';
import Hex5Rbg from 'bundle-text:../../../../games/rbg/hex_5x5.rbg';
import Hex6Rbg from 'bundle-text:../../../../games/rbg/hex_6x6.rbg';
import Hex7Rbg from 'bundle-text:../../../../games/rbg/hex_7x7.rbg';
import Hex8Rbg from 'bundle-text:../../../../games/rbg/hex_8x8.rbg';
import Hex9Rbg from 'bundle-text:../../../../games/rbg/hex_9x9.rbg';
import KnightthroughRbg from 'bundle-text:../../../../games/rbg/knightthrough.rbg';
import TicTacToeRbg from 'bundle-text:../../../../games/rbg/ticTacToe.rbg';
import BreakthroughRg from 'bundle-text:../../../../games/rg/breakthrough.rg';
import TicTacToeRg from 'bundle-text:../../../../games/rg/ticTacToe.rg';

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
  ['Dice Throw (compare)', DiceThrowCompareHrg, Language.hrg] as const,
  ['Dice Throw (guess)', DiceThrowGuessHrg, Language.hrg] as const,
  ['Gomoku (freestyle)', GomokuFreestyleHrg, Language.hrg] as const,
  ['Hex5', Hex5Rbg, Language.rbg] as const,
  ['Hex6', Hex6Rbg, Language.rbg] as const,
  ['Hex7', Hex7Rbg, Language.rbg] as const,
  ['Hex8', Hex8Rbg, Language.rbg] as const,
  ['Hex9', Hex9Rbg, Language.rbg] as const,
  ['Knightthrough', KnightthroughHrg, Language.hrg] as const,
  ['Knightthrough', KnightthroughRbg, Language.rbg] as const,
  ['SatSolver', SatSolverHrg, Language.hrg] as const,
  ['ShortestPath', ShortestPathHrg, Language.hrg] as const,
  ['MontyHall', MontyHallGdl, Language.gdl] as const,
  ['TicTacToe', TicTacToeGdl, Language.gdl] as const,
  ['TicTacToe', TicTacToeHrg, Language.hrg] as const,
  ['TicTacToe', TicTacToeRbg, Language.rbg] as const,
  ['TicTacToe', TicTacToeRg, Language.rg] as const,
].map(([name, source, extension]) => ({
  name: `${name}.${extension}`,
  source,
  extension,
}));
