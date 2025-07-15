from lineGames import *
game = LineGame()
game.addBoardAlquerque(width=5, height=5) 
game.addPiecesAlquerque(InitialPieces.SQUARE_E_TO_W_NOCENTRAL)
game.setRules(CaptureSequences.FULL, captureMandatory.MANDATORY)
game.setLimits(stagnation=100, maxTurns=None)
game.printHRG()
