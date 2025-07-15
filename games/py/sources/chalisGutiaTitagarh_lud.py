from lineGames import *
game = LineGame()
game.addBoardAlquerque(width=9, height=9) 
game.addPiecesAlquerque(InitialPieces.SQUARE_2ROWS)
game.addPieces(WHITE_PIECE, 'cy6x3', 'cy6x4', 'cy6x5', 'cy6x6', 'cy6x7', 'cy6x8')
game.addPieces(BLACK_PIECE, 'cy2x0', 'cy2x1', 'cy2x2', 'cy2x3', 'cy2x4', 'cy2x5')
game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
game.setLimits(stagnation=None, maxTurns=1250)
game.printHRG()
