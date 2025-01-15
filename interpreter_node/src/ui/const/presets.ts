import AmazonsHrg from 'bundle-text:../../../../games/hrg/amazons.hrg';
import AtaxxHrg from 'bundle-text:../../../../games/hrg/ataxx.hrg';
import BreakthroughHrg from 'bundle-text:../../../../games/hrg/breakthrough.hrg';
import Connect4Hrg from 'bundle-text:../../../../games/hrg/connect4.hrg';
import DiceThrowCompareHrg from 'bundle-text:../../../../games/hrg/diceThrowCompare.hrg';
import DiceThrowGuessHrg from 'bundle-text:../../../../games/hrg/diceThrowGuess.hrg';
import GomokuFreestyleHrg from 'bundle-text:../../../../games/hrg/gomoku-freestyle.hrg';
import KnightthroughHrg from 'bundle-text:../../../../games/hrg/knightthrough.hrg';
import PentagoHrg from 'bundle-text:../../../../games/hrg/pentago.hrg';
import SatSolverHrg from 'bundle-text:../../../../games/hrg/satSolver.hrg';
import ShortestPathHrg from 'bundle-text:../../../../games/hrg/shortestPath.hrg';
import TicTacToeHrg from 'bundle-text:../../../../games/hrg/ticTacToe.hrg';
import Connect4Gdl from 'bundle-text:../../../../games/kif/connect4.kif';
import MontyHallGdl from 'bundle-text:../../../../games/kif/montyHall.kif';
import TicTacToeGdl from 'bundle-text:../../../../games/kif/ticTacToe.kif';
import AmazonsRbg from 'bundle-text:../../../../games/rbg/amazons.rbg';
import AmazonsSplit2Rbg from 'bundle-text:../../../../games/rbg/amazons_split2.rbg';
import ArimaaRbg from 'bundle-text:../../../../games/rbg/arimaa.rbg';
import ArimaaFixedPositionRbg from 'bundle-text:../../../../games/rbg/arimaa_fixedPosition.rbg';
import ArimaaSplitRbg from 'bundle-text:../../../../games/rbg/arimaa_split.rbg';
import BreakthroughRbg from 'bundle-text:../../../../games/rbg/breakthrough.rbg';
import BreakthruRbg from 'bundle-text:../../../../games/rbg/breakthru.rbg';
import BreakthruSplitRbg from 'bundle-text:../../../../games/rbg/breakthru_split.rbg';
import CanadianDraughtsRbg from 'bundle-text:../../../../games/rbg/canadianDraughts.rbg';
import ChessRbg from 'bundle-text:../../../../games/rbg/chess.rbg';
import ChessGardner5x5KingCaptureRbg from 'bundle-text:../../../../games/rbg/chessGardner5x5_kingCapture.rbg';
import ChessLosAlamos6x6KingCaptureRbg from 'bundle-text:../../../../games/rbg/chessLosAlamos6x6_kingCapture.rbg';
import ChessQuick5x6KingCaptureRbg from 'bundle-text:../../../../games/rbg/chessQuick5x6_kingCapture.rbg';
import ChessSilverman4x5KingCaptureRbg from 'bundle-text:../../../../games/rbg/chessSilverman4x5_kingCapture.rbg';
import Chess200Rbg from 'bundle-text:../../../../games/rbg/chess_200.rbg';
import ChessKingCaptureRbg from 'bundle-text:../../../../games/rbg/chess_kingCapture.rbg';
import ChessKingCapture200Rbg from 'bundle-text:../../../../games/rbg/chess_kingCapture_200.rbg';
import ChineseCheckers6Rbg from 'bundle-text:../../../../games/rbg/chineseCheckers6.rbg';
import Connect4Rbg from 'bundle-text:../../../../games/rbg/connect4.rbg';
import Connect6Rbg from 'bundle-text:../../../../games/rbg/connect6.rbg';
import Connect6SplitRbg from 'bundle-text:../../../../games/rbg/connect6_split.rbg';
import DoubleChessRbg from 'bundle-text:../../../../games/rbg/doubleChess.rbg';
import EnglishDraughtsRbg from 'bundle-text:../../../../games/rbg/englishDraughts.rbg';
import EnglishDraughtsSplitRbg from 'bundle-text:../../../../games/rbg/englishDraughts_split.rbg';
import FoxAndHoundsRbg from 'bundle-text:../../../../games/rbg/foxAndHounds.rbg';
import GessRbg from 'bundle-text:../../../../games/rbg/gess.rbg';
import GoRbg from 'bundle-text:../../../../games/rbg/go.rbg';
import GoConstsumRbg from 'bundle-text:../../../../games/rbg/go_constsum.rbg';
import GoNopassRbg from 'bundle-text:../../../../games/rbg/go_nopass.rbg';
import GomokuFreestyleRbg from 'bundle-text:../../../../games/rbg/gomoku_freeStyle.rbg';
import GomokuStandardRbg from 'bundle-text:../../../../games/rbg/gomoku_standard.rbg';
import HexRbg from 'bundle-text:../../../../games/rbg/hex.rbg';
import Hex9x9Rbg from 'bundle-text:../../../../games/rbg/hex_9x9.rbg';
import InternationalDraughtsRbg from 'bundle-text:../../../../games/rbg/internationalDraughts.rbg';
import KnightthroughRbg from 'bundle-text:../../../../games/rbg/knightthrough.rbg';
import PaperSoccerRbg from 'bundle-text:../../../../games/rbg/paperSoccer.rbg';
import PentagoRbg from 'bundle-text:../../../../games/rbg/pentago.rbg';
import PentagoSplitRbg from 'bundle-text:../../../../games/rbg/pentago_split.rbg';
import ReversiRbg from 'bundle-text:../../../../games/rbg/reversi.rbg';
import SkirmishRbg from 'bundle-text:../../../../games/rbg/skirmish.rbg';
import TheMillGameRbg from 'bundle-text:../../../../games/rbg/theMillGame.rbg';
import TheMillGameSplitRbg from 'bundle-text:../../../../games/rbg/theMillGame_split.rbg';
import TicTacToeRbg from 'bundle-text:../../../../games/rbg/ticTacToe.rbg';
import YavalathRbg from 'bundle-text:../../../../games/rbg/yavalath.rbg';
import BreakthroughOptRg from 'bundle-text:../../../../games/rg/breakthrough-opt.rg';
import BreakthroughRg from 'bundle-text:../../../../games/rg/breakthrough.rg';
import MinimalRg from 'bundle-text:../../../../games/rg/minimal.rg';
import TicTacToeRg from 'bundle-text:../../../../games/rg/ticTacToe.rg';

import { Language } from '../../types';

// prettier-ignore
const games = [
  ['Amazons', AmazonsHrg, Language.hrg],
  ['Amazons (split2)', AmazonsSplit2Rbg, Language.rbg],
  ['Amazons', AmazonsRbg, Language.rbg],
  ['Arimaa', ArimaaRbg, Language.rbg],
  ['Arimaa (fixed position)', ArimaaFixedPositionRbg, Language.rbg],
  ['Arimaa (split)', ArimaaSplitRbg, Language.rbg],
  ['Ataxx', AtaxxHrg, Language.hrg],
  ['Breakthrough', BreakthroughHrg, Language.hrg],
  ['Breakthrough', BreakthroughRbg, Language.rbg],
  ['Breakthrough', BreakthroughRg, Language.rg],
  ['Breakthrough (opt)', BreakthroughOptRg, Language.rg],
  ['Breakthru', BreakthruRbg, Language.rbg],
  ['Breakthru (split)', BreakthruSplitRbg, Language.rbg],
  ['CanadianDraughts', CanadianDraughtsRbg, Language.rbg],
  ['Chess', ChessRbg, Language.rbg],
  ['Chess (200)', Chess200Rbg, Language.rbg],
  ['Chess (king capture)', ChessKingCaptureRbg, Language.rbg],
  ['Chess (king capture, 200)', ChessKingCapture200Rbg, Language.rbg],
  ['ChessGardner (5x5, king capture)', ChessGardner5x5KingCaptureRbg, Language.rbg],
  ['ChessLosAlamos (6x6, king capture)', ChessLosAlamos6x6KingCaptureRbg, Language.rbg],
  ['ChessQuick (5x6, king capture)', ChessQuick5x6KingCaptureRbg, Language.rbg],
  ['ChessSilverman (4x5, king capture)', ChessSilverman4x5KingCaptureRbg, Language.rbg],
  ['ChineseCheckers6', ChineseCheckers6Rbg, Language.rbg],
  ['Connect4', Connect4Gdl, Language.gdl],
  ['Connect4', Connect4Hrg, Language.hrg],
  ['Connect4', Connect4Rbg, Language.rbg],
  ['Connect6', Connect6Rbg, Language.rbg],
  ['Connect6 (split)', Connect6SplitRbg, Language.rbg],
  ['Dice Throw (compare)', DiceThrowCompareHrg, Language.hrg],
  ['Dice Throw (guess)', DiceThrowGuessHrg, Language.hrg],
  ['DoubleChess', DoubleChessRbg, Language.rbg],
  ['EnglishDraughts', EnglishDraughtsRbg, Language.rbg],
  ['EnglishDraughts (split)', EnglishDraughtsSplitRbg, Language.rbg],
  ['FoxAndHounds', FoxAndHoundsRbg, Language.rbg],
  ['Gess', GessRbg, Language.rbg],
  ['Go', GoRbg, Language.rbg],
  ['Go (constsum)', GoConstsumRbg, Language.rbg],
  ['Go (nopass)', GoNopassRbg, Language.rbg],
  ['Gomoku (freestyle)', GomokuFreestyleHrg, Language.hrg],
  ['Gomoku (freestyle)', GomokuFreestyleRbg, Language.rbg],
  ['Gomoku (standard)', GomokuStandardRbg, Language.rbg],
  ['Hex', HexRbg, Language.rbg],
  ['Hex (9x9)', Hex9x9Rbg, Language.rbg],
  ['InternationalDraughts', InternationalDraughtsRbg, Language.rbg],
  ['Knightthrough', KnightthroughHrg, Language.hrg],
  ['Knightthrough', KnightthroughRbg, Language.rbg],
  ['Minimal', MinimalRg, Language.rg],
  ['MontyHall', MontyHallGdl, Language.gdl],
  ['PaperSoccer', PaperSoccerRbg, Language.rbg],
  ['Pentago', PentagoHrg, Language.hrg],
  ['Pentago', PentagoRbg, Language.rbg],
  ['Pentago (split)', PentagoSplitRbg, Language.rbg],
  ['Reversi', ReversiRbg, Language.rbg],
  ['SatSolver', SatSolverHrg, Language.hrg],
  ['ShortestPath', ShortestPathHrg, Language.hrg],
  ['Skirmish', SkirmishRbg, Language.rbg],
  ['TheMillGame', TheMillGameRbg, Language.rbg],
  ['TheMillGame (split)', TheMillGameSplitRbg, Language.rbg],
  ['TicTacToe', TicTacToeGdl, Language.gdl],
  ['TicTacToe', TicTacToeHrg, Language.hrg],
  ['TicTacToe', TicTacToeRbg, Language.rbg],
  ['TicTacToe', TicTacToeRg, Language.rg],
  ['Yavalath', YavalathRbg, Language.rbg],
] as const;

export const presets = games.map(([name, source, extension]) => ({
  name: `${name}.${extension}`,
  source,
  extension,
}));
