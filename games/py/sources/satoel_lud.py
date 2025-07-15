from lineGames import *
game = LineGame()
game.addBoardAlquerque(width=9, height=9) 
for d in[N, S]: game.attachTriangle3(dir=d, height=3)
game.addPiecesAlquerque(InitialPieces.SQUARE_W_TO_E_NOCENTRAL)
game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
game.setLimits(stagnation=None, maxTurns=1250)
game.printHRG()
