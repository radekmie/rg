from lineGames import *
game = LineGame()
game.addBoardSquareWithDiagonals(width=9, height=9) 
game.addPiecesSquareWithDiagonals(InitialPieces.SQUARE_4ROWS)
game.addPieces(WHITE_PIECE, 'cy6x2', 'cy6x6')
game.addPieces(BLACK_PIECE, 'cy6x0', 'cy6x8')
game.setRules(CaptureSequences.NONE, CaptureMandatory.NO_MANDATORY)
game.setLimits(stagnation=None, maxTurns=1250)
game.printHRG()
