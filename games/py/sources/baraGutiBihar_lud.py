from lineGames import *
game = LineGame()
game.addBoardCircle(lines=8, length=4) 
game.addPiecesCircle(InitialPieces.CIRCLE_CLUSTERED_NOCENTRAL)
game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
game.setLimits(stagnation=None, maxTurns=1250)
game.printHRG()
