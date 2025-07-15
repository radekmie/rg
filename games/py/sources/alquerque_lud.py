from lineGames import *
game = LineGame()
game.addBoardAlquerque(width=5, height=5) 
game.addPiecesAlquerque(InitialPieces.SQUARE_E_TO_W_NOCENTRAL)
game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
game.setLimits(stagnation=None, maxTurns=1250)
game.printHRG()
