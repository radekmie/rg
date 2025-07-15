from lineGames import *
game = LineGame()
game.addBoardSquareWithDiagonals(width=9, height=9) 
game.addPiecesSquareWithDiagonals(InitialPieces.SQUARE_W_TO_E_NOCENTRAL)
game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
game.setLimits(stagnation=None, maxTurns=1250)
game.printHRG()
