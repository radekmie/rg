import AmazonsNaiveHrg from 'bundle-text:../../../../presets/hrg/amazons-naive.hrg';
import AmazonsSmartHrg from 'bundle-text:../../../../presets/hrg/amazons-smart.hrg';
import BreakthroughHrg from 'bundle-text:../../../../presets/hrg/breakthrough.hrg';
import Connect4Hrg from 'bundle-text:../../../../presets/hrg/connect4.hrg';
import DiceThrowCompareHrg from 'bundle-text:../../../../presets/hrg/dice-throw-compare.hrg';
import DiceThrowGuessHrg from 'bundle-text:../../../../presets/hrg/dice-throw-guess.hrg';
import GomokuFreestyleHrg from 'bundle-text:../../../../presets/hrg/gomoku-freestyle.hrg';
import KnightthroughHrg from 'bundle-text:../../../../presets/hrg/knightthrough.hrg';
import SatSolverHrg from 'bundle-text:../../../../presets/hrg/sat-solver.hrg';
import ShortestPathHrg from 'bundle-text:../../../../presets/hrg/shortest-path.hrg';
import TicTacToeHrg from 'bundle-text:../../../../presets/hrg/ticTacToe.hrg';
import Connect4Gdl from 'bundle-text:../../../../presets/kif/connect4.kif';
import MontyHallGdl from 'bundle-text:../../../../presets/kif/monty-hall.kif';
import TicTacToeGdl from 'bundle-text:../../../../presets/kif/ticTacToe.kif';
import AmazonsRbg from 'bundle-text:../../../../presets/rbg/amazons.rbg';
import BreakthroughRbg from 'bundle-text:../../../../presets/rbg/breakthrough.rbg';
import Connect4Rbg from 'bundle-text:../../../../presets/rbg/connect4.rbg';
import Hex2Rbg from 'bundle-text:../../../../presets/rbg/hex2.rbg';
import Hex9Rbg from 'bundle-text:../../../../presets/rbg/hex9.rbg';
import KnightthroughRbg from 'bundle-text:../../../../presets/rbg/knightthrough.rbg';
import TicTacToeRbg from 'bundle-text:../../../../presets/rbg/ticTacToe.rbg';
import BreakthroughRg from 'bundle-text:../../../../presets/rg/breakthrough.rg';
import TicTacToeRg from 'bundle-text:../../../../presets/rg/ticTacToe.rg';

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
  ['Hex2', Hex2Rbg, Language.rbg] as const,
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
