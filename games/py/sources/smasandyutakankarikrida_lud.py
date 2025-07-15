from lineGames import *
game = LineGame()
game.addBoardAlquerque(width=5, height=5) 
for d in[N, S,]: game.attachTriangle3(dir=d, height=5)
game.addPieces(WHITE_PIECE, 'tsouthvy3x0', 'tsouthvy3x1', 'tsouthvy3x2', 'tsouthvy4x0', 'tsouthvy4x1', 'tsouthvy4x2')
game.addPieces(BLACK_PIECE, 'tnorthvy3x0', 'tnorthvy3x1', 'tnorthvy3x2', 'tnorthvy4x0', 'tnorthvy4x1', 'tnorthvy4x2')
game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
game.setLimits(stagnation=None, maxTurns=1250)
game.printHRG()
