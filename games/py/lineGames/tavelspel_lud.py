from lineGames import *
game = LineGame()
game.addBoardGrid(width=13, height=13)
game.addPiecesGrid(InitialPieces.SQUARE_2ROWS)
game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
game.setLimits(stagnation=None, maxTurns=1250)
game.printHRG()
