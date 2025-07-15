from lineGames import *
game = LineGame()
game.addBoardCircle(lines=6, length=8) 
game.addPiecesCircle(InitialPieces.CIRCLE_CLUSTERED_NOCENTRAL)
game.setRules(CaptureSequences.FULL, captureMandatory.MANDATORY)
game.setLimits(stagnation=100, maxTurns=None)
game.printHRG()
