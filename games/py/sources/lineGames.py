
# v0.10b

# - fixed issue with too large 'less' domain
# - 13(2) games implemented (aplikowalne kwadraty z trójkątami) (SmasandyutakankarikridaAllahabad_lud out)
# - 13(3) games implemented (wszystkie kółka i proste kwadraty)
# - setLimits: stagnation, maxTurns
# - wszystkie triangle direction działają
# - DashGuti perf śmiga
# - Lau Kata Kati perf śmiga
# - dopisany niby SPLIT
# - Alquerque perf śmiga
# - gry kółkowe powinny działać - Pretwa perf śmiga

from enum import Enum


_rules_header = '''
domain Piece = empty | whitePawn | blackPawn | border
domain Player = black | white
domain Score = 50 | 0 | 100
domain Bool = 0 | 1

domain Count = I(X) where X in 0..{counter}
domain CountOrNan = nan | Count
domain PieceCount = I(X) where X in 0..{numPieces}

pawnOf : Player -> Piece
pawnOf(white) = whitePawn
pawnOf(black) = blackPawn

opponent : Player -> Player
opponent(white) = black
opponent(black) = white

increment : Count -> CountOrNan
increment(I(X)) = if X < {counter} then I(X+1) else nan

less : PieceCount -> PieceCount -> Bool
less(I(X), I(Y)) = if X < Y then 1 else 0

'''

_rules_begin = '''
me : Player = white
pos : PositionOrNull
stagnation : Count = I(0)
turn : Count = I(0)
pawnCount: Player -> PieceCount = {:I(0)}

graph captureFromPosPossible() {
  branch dir in DirIndex {
    pos = dirs[dir][pos]
    check(board[pos] == pawnOf[opponent[me]])
    check(board[dirs[dir][pos]] == empty)
  }
}
'''

_rules_checkEnd = '''
graph checkEnd() {
  if #CHECK_END# {
    pos = nextPos[null]
    loop {
      if pos == null {break()}
      if board[pos] == whitePawn {
        pawnCount[white] = increment[pawnCount[white]]
      } else {
        if board[pos] == blackPawn {
          pawnCount[black] = increment[pawnCount[black]]
        }
      }
      pos = nextPos[pos]
      $_NEXT
    }
    
    if less(pawnCount[black], pawnCount[white]) == 1 {goals[white] = 100 goals[black] = 0} else {
      if less(pawnCount[white], pawnCount[black]) == 1 {goals[white] = 0 goals[black] = 100}
    }
    end()
  }
}
'''

_rules_turn_mandatory = '''
graph rules() {
  loop {
    if reachable(anyCapturePossible()) {
      checkEnd()
      player = me
      capturingMove()
    } else {
      if reachable(anyNormalMovePossible()) {
        checkEnd()
        player = me
        nonCapturingMove()
      } else {
        goals[me] = 0
        goals[opponent[me]] = 100
        end()
      }
    }
    #TURN_INC#
    me = opponent[me]
  }
}
'''

_rules_turn_nomandatory = '''
graph rules() {
  loop {
    if reachable(anyMovePossible()) {
      checkEnd()
      player = me
      move()
    } else {
      goals[me] = 0
      goals[opponent[me]] = 100
      end()
    }
    #TURN_INC#
    me = opponent[me]
  }
}
'''

_rules_inner_nomandatory_nosequence = '''
graph anyMovePossible() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  branch {
    branch dir in DirIndex {
     check(board[dirs[dir][pos]] == empty)
    }
  } or {
    captureFromPosPossible()
  }
}

graph move() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  branch {
    $$ pos
    $ NORMAL
    board[pos] = empty 
    branch dir in DirIndex {
      pos = dirs[dir][pos]
      check(board[pos] == empty)
      $ dir
    }
    player = keeper
    board[pos] = pawnOf[me]
    #STAGNATION_INC#
  } or {
    $$ pos
    $ CAPTURE
    board[pos] = empty
    branch dir in DirIndex {
      pos = dirs[dir][pos]
      check(board[pos] == pawnOf[opponent[me]])
      board[pos] = empty
      pos = dirs[dir][pos]
      check(board[pos] == empty)
      $ dir
    }
    player = keeper
    board[pos] = pawnOf[me]
    stagnation = I(0)
  }
}

'''

_rules_inner_nomandatory_sequence = '''
graph anyMovePossible() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  branch {
    branch dir in DirIndex {
     check(board[dirs[dir][pos]] == empty)
    }
  } or {
    captureFromPosPossible()
  }
}

graph move() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  branch {
    $$ pos
    $ NORMAL
    board[pos] = empty 
    branch dir in DirIndex {
      pos = dirs[dir][pos]
      check(board[pos] == empty)
      $ dir
    }
    player = keeper
    board[pos] = pawnOf[me]
    #STAGNATION_INC#
  } or {
    $$ pos
    $ CAPTURE
    board[pos] = empty
    loop {
      branch dir in DirIndex {
        pos = dirs[dir][pos]
        check(board[pos] == pawnOf[opponent[me]])
        board[pos] = empty
        pos = dirs[dir][pos]
        check(board[pos] == empty)
        $ dir
      }
      branch {} or {break()}
    }
    player = keeper
    board[pos] = pawnOf[me]
    stagnation = I(0)
  }
}
'''

_rules_inner_nomandatory_split = '''
graph anyMovePossible() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  branch {
    branch dir in DirIndex {
     check(board[dirs[dir][pos]] == empty)
    }
  } or {
    captureFromPosPossible()
  }
}

graph move() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  branch {
    $$ pos
    $ NORMAL
    board[pos] = empty 
    branch dir in DirIndex {
      pos = dirs[dir][pos]
      check(board[pos] == empty)
      $ dir
    }
    player = keeper
    board[pos] = pawnOf[me]
    #STAGNATION_INC#
  } or {
    $$ pos
    $ CAPTURE
    board[pos] = empty
    loop {
      branch dir in DirIndex {
        pos = dirs[dir][pos]
        check(board[pos] == pawnOf[opponent[me]])
        board[pos] = empty
        pos = dirs[dir][pos]
        check(board[pos] == empty)
        $ dir
      }
      if not(reachable(captureFromPosPossible())) {
        break()
      }
      branch {} or {break()}
      player = me
    }
    player = keeper
    board[pos] = pawnOf[me]
    stagnation = I(0)
  }
}
'''

_rules_inner_mandatory_nosequence = '''
graph anyCapturePossible() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  captureFromPosPossible()
}
graph anyNormalMovePossible() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  branch dir in DirIndex {
    check(board[dirs[dir][pos]] == empty)
  }
}

graph nonCapturingMove() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  $$ pos
  $ NORMAL
  board[pos] = empty 
  branch dir in DirIndex {
    pos = dirs[dir][pos]
    check(board[pos] == empty)
    $ dir
  }
  player = keeper
  board[pos] = pawnOf[me]
  #STAGNATION_INC#
}

graph capturingMove() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  $$ pos
  $ CAPTURE
  board[pos] = empty
  //loop {
    branch dir in DirIndex {
      pos = dirs[dir][pos]
      check(board[pos] == pawnOf[opponent[me]])
      board[pos] = empty
      pos = dirs[dir][pos]
      check(board[pos] == empty)
      $ dir
    }
  //  if not(reachable(captureFromPosPossible())) {
  //    break()
  //  }
  //}
  player = keeper
  board[pos] = pawnOf[me]
  stagnation = I(0)
}
'''

_rules_inner_mandatory_sequence = '''
graph anyCapturePossible() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  captureFromPosPossible()
}
graph anyNormalMovePossible() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  branch dir in DirIndex {
    check(board[dirs[dir][pos]] == empty)
  }
}

graph nonCapturingMove() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  $$ pos
  $ NORMAL
  board[pos] = empty 
  branch dir in DirIndex {
    pos = dirs[dir][pos]
    check(board[pos] == empty)
    $ dir
  }
  player = keeper
  board[pos] = pawnOf[me]
  #STAGNATION_INC#
}

graph capturingMove() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  $$ pos
  $ CAPTURE
  board[pos] = empty
  loop {
    branch dir in DirIndex {
      pos = dirs[dir][pos]
      check(board[pos] == pawnOf[opponent[me]])
      board[pos] = empty
      pos = dirs[dir][pos]
      check(board[pos] == empty)
      $ dir
    }
    if not(reachable(captureFromPosPossible())) {
      break()
    }
  }
  player = keeper
  board[pos] = pawnOf[me]
  stagnation = I(0)
}
'''

_rules_inner_mandatory_split = '''
graph anyCapturePossible() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  captureFromPosPossible()
}
graph anyNormalMovePossible() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  branch dir in DirIndex {
    check(board[dirs[dir][pos]] == empty)
  }
}

graph nonCapturingMove() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  $$ pos
  $ NORMAL
  board[pos] = empty 
  branch dir in DirIndex {
    pos = dirs[dir][pos]
    check(board[pos] == empty)
    $ dir
  }
  player = keeper
  board[pos] = pawnOf[me]
  #STAGNATION_INC#
}

graph capturingMove() {
  pos = Position(*)
  check(board[pos] == pawnOf[me])
  $$ pos
  $ CAPTURE
  board[pos] = empty
  loop {
    branch dir in DirIndex {
      pos = dirs[dir][pos]
      check(board[pos] == pawnOf[opponent[me]])
      board[pos] = empty
      pos = dirs[dir][pos]
      check(board[pos] == empty)
      $ dir
    }
    if not(reachable(captureFromPosPossible())) {
      break()
    }
    player = me
  }
  player = keeper
  board[pos] = pawnOf[me]
  stagnation = I(0)
}
'''

BLACK_PIECE = 'blackPawn'
WHITE_PIECE = 'whitePawn'

N = 'north'
E = 'east'
S = 'south'
W = 'west'
NE = 'northeast'
NW = 'northwest'
SE = 'southeast'
SW = 'southwest'
R = 'right'
L = 'left'

def rev(d):
    """
    Reverse direction d.
    """
    assert d in [N, E, S, W, NE, NW, SE, SW, R, L], "Direction must be one of N, E, S, W, NE, NW, SE, SW."
    if d == N: return S
    if d == E: return W
    if d == S: return N
    if d == W: return E
    if d == NE: return SW
    if d == NW: return SE
    if d == SE: return NW
    if d == SW: return NE
    if d == R: return L
    if d == L: return R

def rot(d, a):
    """
    Rotate direction d by angle a (in degrees).
    """
    assert a in [0, 45, 90, 135, 180, 225, 270, 315], "Angle must be one of 0, 45, 90, 135, 180, 225, 270, 315."
    assert d in [N, E, S, W, NE, NW, SE, SW], "Direction must be one of N, E, S, W, NE, NW, SE, SW."
    if a == 0: return d
    elif a == 45:
        if d == N: return NE
        if d == E: return SE
        if d == S: return SW
        if d == W: return NW
        if d == NE: return E
        if d == NW: return N
        if d == SE: return S
        if d == SW: return W
    elif a == 90:
        if d == N: return E
        if d == E: return S
        if d == S: return W
        if d == W: return N
        if d == NE: return SE
        if d == SE: return SW
        if d == SW: return NW
        if d == NW: return NE
    elif a == 135:
        if d == N: return SE
        if d == E: return SW
        if d == S: return NW
        if d == W: return NE
        if d == NE: return S
        if d == NW: return E
        if d == SE: return W
        if d == SW: return N
    elif a == 180:
        if d == N: return S
        if d == E: return W
        if d == S: return N
        if d == W: return E
        if d == NE: return SW
        if d == SE: return NW
        if d == SW: return NE
        if d == NW: return SE
    elif a == 225:
        if d == N: return SW
        if d == E: return NW
        if d == S: return NE
        if d == W: return SE
        if d == NE: return W
        if d == NW: return S
        if d == SE: return N
        if d == SW: return E
    elif a == 270:
        if d == N: return W
        if d == E: return N
        if d == S: return E
        if d == W: return S
        if d == NE: return NW
        if d == SE: return NE
        if d == SW: return SE
        if d == NW: return SW
    elif a == 315:
        if d == N: return NW
        if d == E: return NE
        if d == S: return SE
        if d == W: return SW
        if d == NE: return N
        if d == NW: return W
        if d == SE: return E
        if d == SW: return S
    else:
        raise ValueError("Angle must be one of 0, 45, 90, 135, 180, 225, 270, 315.")

def dirToRot(dir):
    """
    Convert rotation to direction.
    """
    assert dir in [N, E, S, W, NE, NW, SE, SW], "Rotation must be one of N, E, S, W, NE, NW, SE, SW."
    if dir == S: return 0
    if dir == SW: return 45
    if dir == W: return 90
    if dir == NW: return 135
    if dir == N: return 180
    if dir == NE: return 225
    if dir == E: return 270
    if dir == SE: return 315
    
#def v(x, y, name): return name+"x"+str(x)+"y"+str(y)
def v(x, y, name): return name+"y"+str(y)+"x"+str(x)



class InitialPieces(Enum):
    SQUARE_E_TO_W_NOCENTRAL = 1
    SQUARE_W_TO_E_NOCENTRAL = 2
    CIRCLE_CLUSTERED_NOCENTRAL = 3
    TRIANGLE_FULL_NOROOT = 4
    SQUARE_1ROWS = 5
    SQUARE_2ROWS = 6
    SQUARE_4ROWS = 7

class CaptureSequences(Enum):
    NONE = 11
    SPLIT = 12
    FULL = 33
class captureMandatory(Enum):
    MANDATORY = 21
    NOMANDATORY = 22

class LineGame:
    def __init__(self):
        self.obj={}
        self.centralletter='c'
        self.dirs=[]
        self.maxx=-1
        self.maxy=-1
        self.captSeq=-1
        self.captMand=-1
        self.stagnation=None
        self.maxTurns=100
        self.counter=100
        self.numpieces=1

    def addBoardPoint(self):
        self.maxx = 1
        self.maxy = 1
        self.centraltype='Point'
        n=self.centralletter
        self.dirs = [N, E, S, W, NE, NW, SE, SW] 
        self.obj[v(0, 0, n)] = {}
     
    def addBoardAlquerque(self, width, height):
        assert width > 0 and height > 0, "Width and height must be greater than 0."
        assert width % 2 == 1 and height % 2 == 1, "Width and height must be odd for Alquerque boards."
        self.maxx = width
        self.maxy = height
        self.centraltype='Alquerque'
        n=self.centralletter
        self.dirs = [N, E, S, W, NE, NW, SE, SW]  # all directions for Alquerque
        for y in range(0,height):
            for x in range(0,width):
                self.obj[v(x, y, n)] = {}
                if x > 0: self.obj[v(x, y, n)][W] = v(x - 1, y, n)
                if y > 0: self.obj[v(x, y, n)][N] = v(x, y - 1, n)
                if x < width-1: self.obj[v(x, y, n)][E] = v(x + 1, y, n)
                if y < height-1: self.obj[v(x, y, n)][S] = v(x, y + 1, n)
                if x > 0 and y > 0 and (x+y)%2==0: self.obj[v(x, y, n)][NW] = v(x - 1, y - 1, n)
                if x < width-1 and y > 0 and (x+y)%2==0: self.obj[v(x, y, n)][NE] = v(x + 1, y - 1, n)
                if x > 0 and y < height-1 and (x+y)%2==0: self.obj[v(x, y, n)][SW] = v(x - 1, y + 1, n)
                if x < width-1 and y < height-1 and (x+y)%2==0: self.obj[v(x, y, n)][SE] = v(x + 1, y + 1, n)
    
    def addPiecesAlquerque(self, initPieces):
        assert self.centraltype in ['Alquerque', 'SquareWithDiagonals', 'Grid'], "This method can only be used for Alquerque, Grid, or SquareWithDiagonals boards."
        assert initPieces in [InitialPieces.SQUARE_E_TO_W_NOCENTRAL, InitialPieces.SQUARE_W_TO_E_NOCENTRAL, InitialPieces.SQUARE_2ROWS, InitialPieces.SQUARE_4ROWS, InitialPieces.SQUARE_1ROWS], "Unsupported initial pieces type."
        height = self.maxy
        width = self.maxx
        n = self.centralletter
        if initPieces in [InitialPieces.SQUARE_E_TO_W_NOCENTRAL, InitialPieces.SQUARE_W_TO_E_NOCENTRAL]:
          self.numpieces += width * height
          for y in range(0,(height)//2): 
              for x in range(0,width): 
                  if v(x, y, n) in self.obj: self.obj[v(x, y, n)]['_piece'] = BLACK_PIECE
          for y in range((height)//2+1,height): 
              for x in range(0,width): 
                  if v(x, y, n) in self.obj: self.obj[v(x, y, n)]['_piece'] = WHITE_PIECE
          if initPieces == InitialPieces.SQUARE_E_TO_W_NOCENTRAL:
              for x in range(0,(width)//2): 
                  if v(x, y, n) in self.obj: self.obj[v(x, (height)//2, n)]['_piece'] = BLACK_PIECE
              for x in  range((height)//2+1,width):  
                  if v(x, y, n) in self.obj: self.obj[v(x, (height)//2, n)]['_piece'] = WHITE_PIECE
          elif initPieces == InitialPieces.SQUARE_W_TO_E_NOCENTRAL:
              for x in range(0,(width)//2): 
                  if v(x, y, n) in self.obj: self.obj[v(x, (height)//2, n)]['_piece'] = WHITE_PIECE
              for x in  range((height)//2+1,width):  
                  if v(x, y, n) in self.obj: self.obj[v(x, (height)//2, n)]['_piece'] = BLACK_PIECE
        if initPieces in [InitialPieces.SQUARE_1ROWS]:
          assert height >= 2, "Height must be at least 2 for SQUARE_1ROWS initial pieces."
          self.numpieces += width * 2
          for x in range(0,width): 
                  if v(x, 0, n) in self.obj: self.obj[v(x, 0, n)]['_piece'] = BLACK_PIECE
          for x in range(0,width): 
                  if v(x, width-1, n) in self.obj: self.obj[v(x, width-1, n)]['_piece'] = WHITE_PIECE
        if initPieces in [InitialPieces.SQUARE_2ROWS]:
          assert height >= 4, "Height must be at least 4 for SQUARE_2ROWS initial pieces."
          self.numpieces += width * 4
          for y in range(0,2): 
              for x in range(0,width): 
                  if v(x, y, n) in self.obj: self.obj[v(x, y, n)]['_piece'] = BLACK_PIECE
          for y in range(height-2,height): 
              for x in range(0,width): 
                  if v(x, y, n) in self.obj: self.obj[v(x, y, n)]['_piece'] = WHITE_PIECE
        if initPieces in [InitialPieces.SQUARE_4ROWS]:
          assert height >= 8, "Height must be at least 8 for SQUARE_4ROWS initial pieces."
          self.numpieces += width * 8
          for y in range(0,2): 
              for x in range(0,width): 
                  if v(x, y, n) in self.obj: self.obj[v(x, y, n)]['_piece'] = BLACK_PIECE
          for y in range(height-2,height): 
              for x in range(0,width): 
                  if v(x, y, n) in self.obj: self.obj[v(x, y, n)]['_piece'] = WHITE_PIECE

    def addBoardSquareWithDiagonals(self, width, height): 
        assert width > 0 and height > 0, "Width and height must be greater than 0."
        assert width % 2 == 1 and height % 2 == 1, "Width and height must be odd for SquareWithDiagonals boards."
        self.maxx = width
        self.maxy = height
        self.centraltype='SquareWithDiagonals'
        n=self.centralletter
        self.dirs = [N, E, S, W, NE, NW, SE, SW]  
        for y in range(0,height,2):
            for x in range(0,width,2):
                self.obj[v(x, y, n)] = {}
                if x > 0: self.obj[v(x, y, n)][W] = v(x - 2, y, n)
                if y > 0: self.obj[v(x, y, n)][N] = v(x, y - 2, n)
                if x < width - 2: self.obj[v(x, y, n)][E] = v(x + 2, y, n)
                if y < height - 2: self.obj[v(x, y, n)][S] = v(x, y + 2, n)
                if x > 0 and y > 0: self.obj[v(x, y, n)][NW] = v(x - 1, y - 1, n)
                if x < width - 2 and y > 0: self.obj[v(x, y, n)][NE] = v(x + 1, y - 1, n)
                if x > 0 and y < height - 2: self.obj[v(x, y, n)][SW] = v(x - 1, y + 1, n)
                if x < width - 2 and y < height - 2: self.obj[v(x, y, n)][SE] = v(x + 1, y + 1, n)
        for y in range(1,height,2):
            for x in range(1,width,2):
                self.obj[v(x, y, n)] = {}
                self.obj[v(x, y, n)][NW] = v(x - 1, y - 1, n)
                self.obj[v(x, y, n)][NE] = v(x + 1, y - 1, n)
                self.obj[v(x, y, n)][SW] = v(x - 1, y + 1, n)
                self.obj[v(x, y, n)][SE] = v(x + 1, y + 1, n)

    def addPiecesSquareWithDiagonals(self, initPieces):
        assert self.centraltype == 'SquareWithDiagonals', "This method can only be used for SquareWithDiagonals boards."
        
        if initPieces in [InitialPieces.SQUARE_E_TO_W_NOCENTRAL, InitialPieces.SQUARE_W_TO_E_NOCENTRAL]:
          self.addPiecesAlquerque(initPieces)  # Use the same method as Alquerque for initial pieces
        if initPieces in [InitialPieces.SQUARE_2ROWS, InitialPieces.SQUARE_4ROWS]:
          self.addPiecesAlquerque(initPieces)  # Use the same method as Alquerque for initial pieces

    def addBoardGrid(self, width, height):
        assert width > 0 and height > 0, "Width and height must be greater than 0."
        self.maxx = width
        self.maxy = height
        self.centraltype='Grid'
        n=self.centralletter
        self.dirs = [N, E, S, W,]
        for y in range(0,height):
            for x in range(0,width):
                self.obj[v(x, y, n)] = {}
                if x > 0: self.obj[v(x, y, n)][W] = v(x - 1, y, n)
                if y > 0: self.obj[v(x, y, n)][N] = v(x, y - 1, n)
                if x < width-1: self.obj[v(x, y, n)][E] = v(x + 1, y, n)
                if y < height-1: self.obj[v(x, y, n)][S] = v(x, y + 1, n)

    def addPiecesGrid(self, initPieces):
        assert self.centraltype == 'Grid', "This method can only be used for Grid boards."
        self.addPiecesAlquerque(initPieces)  # Use the same method as Alquerque for initial pieces

    def _getTriangleRoot(self, dir):
        assert dir in [N, E, S, W, NE, NW, SE, SW], "Direction must be one of N, E, S, W, NE, NW, SE, SW."
        assert self.centraltype != 'Circle', "This method cannot be used for Circle boards."
        if self.centraltype == 'Point': return v(0, 0, self.centralletter)

        if dir == N: return v(self.maxx//2, 0, self.centralletter)
        if dir == E: return v(self.maxx-1, self.maxy//2, self.centralletter)
        if dir == S: return v(self.maxx//2, self.maxy-1, self.centralletter)
        if dir == W: return v(0, self.maxy//2, self.centralletter)
        
        if dir == NW: return v(0, 0, self.centralletter)
        if dir == NE: return v(self.maxx-1, 0, self.centralletter)
      
        if dir == SW: return v(0,  self.maxy-1, self.centralletter)
        if dir == SE: return v(self.maxx-1,  self.maxy-1, self.centralletter)

    def attachTriangle3(self, dir, height):
        assert height > 1, "Height must be greater than 1."
        assert dir in [N, E, S, W, NE, NW, SE, SW], "Direction must be one of N, E, S, W, NE, NW, SE, SW."
        assert self.centraltype != 'Circle', "This method cannot be used for Circles (for now at least)."
        n="t"+dir+"v"
        rotate = dirToRot(dir)

        vroot = self._getTriangleRoot(dir)
        self.obj[vroot][rot(SW,rotate)] = v(0, 1, n) 
        self.obj[vroot][rot(S,rotate)] =  v(1, 1, n) 
        self.obj[vroot][rot(SE,rotate)] = v(2, 1, n) 
        for y in range(1,height):
            self.obj[v(0, y, n)] = { rot(E,rotate): v(1, y, n) }
            if y < height-1: self.obj[v(0, y, n)][rot(SW,rotate)] = v(0, y + 1, n) 
            if y > 1:      
                        self.obj[v(0, y, n)][rot(NE,rotate)] = v(0, y - 1, n)
                        self.obj[v(0, y, n)][rot(E,rotate)] = v(1, y , n)
            if y == 1:     self.obj[v(0, y, n)][rot(NE,rotate)] = vroot
            self.obj[v(1, y, n)] = { rot(E,rotate): v(2, y, n), rot(W,rotate): v(0, y, n) }
            if y < height-1: self.obj[v(1, y, n)][rot(S,rotate)] = v(1, y + 1, n)
            if y > 1:      self.obj[v(1, y, n)][rot(N,rotate)] = v(1, y - 1, n)
            if y == 1:     self.obj[v(1, y, n)][rot(N,rotate)] = vroot
            self.obj[v(2, y, n)] = { rot(W,rotate): v(1, y, n) }
            if y < height-1: self.obj[v(2, y, n)][rot(SE,rotate)] = v(2, y + 1, n)
            if y > 1:      
                        self.obj[v(2, y, n)][rot(NW,rotate)] = v(2, y - 1, n)
                        self.obj[v(2, y, n)][rot(W,rotate)]  = v(1, y , n)
            if y == 1:     self.obj[v(2, y, n)][rot(NW,rotate)] = vroot

    def addPiecesTriangle(self, dir, initPieces, piece):
        assert self.centraltype != 'Circle', "This method cannot be used for Circle boards."
        assert initPieces in [InitialPieces.TRIANGLE_FULL_NOROOT], "Unsupported initial pieces type."

        if initPieces == InitialPieces.TRIANGLE_FULL_NOROOT:
            for v in self.obj.keys():
                if v.startswith("t"+dir+"v") :
                    self.obj[v]['_piece'] = piece
                    self.numpieces += 1

    def addBoardCircle(self, lines, length):
        assert lines > 0 and lines <= 8, "Lines must be in [1, 8]"
        assert length > 0, "Length must be greater than 0."
        assert lines % 2 == 0, "Lines must be even for Circle boards."

        self.maxx = lines
        self.maxy = length
        self.centraltype='Circle'
        n=self.centralletter
        
        if lines == 2: self.dirs = [N, S]
        elif lines == 4: self.dirs = [N, E, S, W]
        elif lines == 6: self.dirs = [N, NE, E, S, SW, W]
        elif lines == 8: self.dirs = [N, NE, E, SE, S, SW, W, NW]
        else: raise ValueError("Unsupported number of lines for Circle board.")

        self.obj[v(0, 0, n)] = {}
        for x in range(0,lines): self.obj[v(0, 0, n)][self.dirs[x]] = v(x, 1, n) # central point
        for y in range(1,length):
            for x in range(0,lines):
                self.obj[v(x, y, n)] = {}
                self.obj[v(x, y, n)][R] = v( (x+1)%lines, y, n)
                self.obj[v(x, y, n)][L] = v( (x-1)%lines, y, n)
                if y==1: self.obj[v(x, y, n)][rev(self.dirs[x])] = v(0, 0, n)
                else: self.obj[v(x, y, n)][rev(self.dirs[x])] = v(x, y-1, n)
                if y < length - 1: self.obj[v(x, y, n)][self.dirs[x]] = v(x, y + 1, n)
        self.dirs.append(R)
        self.dirs.append(L)

    def addPiecesCircle(self, initPieces):
        assert self.centraltype == 'Circle', "This method can only be used for Circle boards."
        assert initPieces in [InitialPieces.CIRCLE_CLUSTERED_NOCENTRAL], "Unsupported initial pieces type."
        assert self.maxx%2==0, "Number of ines must be even for CIRCLE_CLUSTERED_NOCENTRAL."

        for y in range(1,self.maxy): # length
            for x in range(0,self.maxx): # lines
                if v(x, y, self.centralletter) in self.obj:
                    self.obj[v(x, y, self.centralletter)]['_piece'] = WHITE_PIECE if x<self.maxx/2 else BLACK_PIECE
                    self.numpieces += 1

    def addEdge(self, fromNode, toNode, dir):
        assert fromNode in self.obj, f"Node {fromNode} does not exist."
        assert toNode in self.obj, f"Node {toNode} does not exist."
        assert dir not in self.obj[fromNode], f"Direction {dir} already exists in node {fromNode}."
        if dir not in self.dirs: self.dirs.append(dir)
        self.obj[fromNode][dir] = toNode
        self.obj[toNode][rev(dir)] = fromNode

    def addPieces(self, piece, *nodes):
        for node in nodes:
          assert node in self.obj, f"Node {node} does not exist."
          self.obj[node]['_piece'] = piece
          self.numpieces += 1
        
    def addNode(self, name, piece=None, edges=None):
        self.obj[name] = {}
        if edges is not None:
            for dir, target in edges.items():
                self.obj[name][dir] = target
                self.obj[target][rev(dir)] = name
        if piece is not None:
            self.obj[name]['_piece'] = piece
            self.numpieces += 1
        
    def setRules(self, capturesSequences, capturesMandatory):
        assert capturesSequences in [CaptureSequences.NONE, CaptureSequences.SPLIT, CaptureSequences.FULL], "Unsupported capture sequences type."
        assert capturesMandatory in [captureMandatory.MANDATORY, captureMandatory.NOMANDATORY], "Unsupported capture mandatory type."
        self.captSeq = capturesSequences
        self.captMand = capturesMandatory

    def setLimits(self, stagnation=None, maxTurns=None):
        assert stagnation != None or maxTurns != None, "At least one of stagnation or maxTurns must be set."
        self.stagnation = stagnation
        self.maxTurns = maxTurns
        self.counter = max(maxTurns if maxTurns is not None else 0, stagnation if stagnation is not None else 0)

    def checkEdgeCorrectness(self):
        """
        Check if the edges of the board are correctly defined.
        """
        for key in self.obj.keys():
            for dir in self.dirs:
                if dir in self.obj[key] and dir != "_piece":
                    next_pos = self.obj[key][dir]
                    if next_pos not in self.obj:
                        raise Exception(f"Error: {key} has an edge to {next_pos} which is not defined.")
                    if rev(dir) not in self.obj[next_pos]:
                        raise Exception(f"Error: {next_pos} does not have a reverse edge to {key}.")
                    if self.obj[next_pos][rev(dir)] != key:
                        raise Exception(f"Error: {next_pos} has a reverse edge to {key} but it points to {self.obj[next_pos][rev(dir)]}.")
    
    def _toHRGPositionString(self):
        s = 'domain Position ='
        for key in sorted(self.obj.keys()):
            s+= " " + key+" |"
        s= s[:-1] + '\n'  # remove last '|'
        s += 'domain PositionOrNull = null | Position\n'
        s+= '\n'
        ks = sorted(self.obj.keys())
        s+= 'nextPos : PositionOrNull -> PositionOrNull\n'
        s+= f'nextPos(null) = {ks[0]}\n'
        for i in range(len(ks)-1): s += f'nextPos({ks[i]}) = {ks[i+1]}\n'
        s += f'nextPos({ks[-1]}) = null\n'
        return s 

    def _toHRGPositionDir(self, dir):
        s = dir+' : PositionOrNull -> PositionOrNull\n'
        s += dir+'(null) = null\n'
    
        for key in sorted(self.obj.keys()):
            ks = self.obj[key].keys()
            if dir in ks: s += f"{dir}({key}) = {self.obj[key][dir]}\n"
            else:         s += f"{dir}({key}) = null\n"
        return s      

    def _toHRGBoard(self):
        s =  'board : PositionOrNull -> Piece = {\n'
        s += '  :empty;\n'
        s += '  null = border;\n'
        for key in sorted(self.obj.keys()):
            if '_piece' in self.obj[key]:s += f"  {key} = {self.obj[key]['_piece']};\n"
        s = s[:-2] + '\n}\n'
        return s   
    
    def _toHRGDirIndex(self):
        s = 'domain DirIndex = S(X) where X in 0..' + str(len(self.dirs)-1) + '\n'
        s += 'dirs : DirIndex -> PositionOrNull -> PositionOrNull\n'
        for i, d in enumerate(self.dirs): s += f'dirs(S({i})) = {d}\n'
        return s 

    def printHRG(self):
        print(_rules_header.format(counter=self.counter, numPieces=self.numpieces))
        print(self._toHRGPositionString())
        for dir in self.dirs:print(self._toHRGPositionDir(dir))
        print(self._toHRGDirIndex())
        print(self._toHRGBoard())     
        print(_rules_begin)

        if self.captSeq == CaptureSequences.FULL:
            if self.captMand==captureMandatory.MANDATORY: inner=_rules_inner_mandatory_sequence
            else:                                         inner=_rules_inner_nomandatory_sequence
        elif self.captSeq == CaptureSequences.SPLIT:
            if self.captMand==captureMandatory.MANDATORY: inner=_rules_inner_mandatory_split
            else:                                         inner=_rules_inner_nomandatory_split
        elif self.captSeq == CaptureSequences.NONE:
            if self.captMand==captureMandatory.MANDATORY: inner=_rules_inner_mandatory_nosequence
            else:                                         inner=_rules_inner_nomandatory_nosequence
        inner=inner.replace('#STAGNATION_INC#', "" if self.stagnation is None else "stagnation = increment[stagnation]")
        print(inner)
        if self.stagnation is None: checkEnd = _rules_checkEnd.replace('#CHECK_END#', f'turn == I({self.maxTurns})')
        elif self.maxTurns is None: checkEnd = _rules_checkEnd.replace('#CHECK_END#', f'stagnation == I({self.stagnation})')
        else:                       checkEnd = _rules_checkEnd.replace('#CHECK_END#', f'stagnation == I({self.stagnation}) || turn == I({self.maxTurns})')
        print(checkEnd)
        if self.captMand==captureMandatory.MANDATORY: print(_rules_turn_mandatory.replace('#TURN_INC#','' if self.maxTurns is None else "turn = increment[turn]"))
        else:                                         print(_rules_turn_nomandatory.replace('#TURN_INC#','' if self.maxTurns is None else "turn = increment[turn]"))

    def printDebug(self):
        print(f"Central type: {self.centraltype}")
        print(f"Counter: {self.counter}, Stagnation: {self.stagnation}, Max turns: {self.maxTurns}")
        print('Dirs:', self.dirs)
        for key in sorted(self.obj.keys()):
            print(f"{key}: {{", end="")
            for subkey in sorted(self.obj[key].keys()): print(f"{subkey}:{self.obj[key][subkey]}, ", end="")
            print("}")


# GAMES

def Alquerque_rbg():
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    game.addPiecesAlquerque(InitialPieces.SQUARE_E_TO_W_NOCENTRAL)
    game.setRules(CaptureSequences.FULL, captureMandatory.MANDATORY)
    game.setLimits(stagnation=100, maxTurns=None)
    return game

def Alquerque_lud():
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    game.addPiecesAlquerque(InitialPieces.SQUARE_E_TO_W_NOCENTRAL)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def Pretwa_rbg():
    game = LineGame()
    game.addBoardCircle(lines=6, length=4) 
    game.addPiecesCircle(InitialPieces.CIRCLE_CLUSTERED_NOCENTRAL)
    game.setRules(CaptureSequences.FULL, captureMandatory.MANDATORY)
    game.setLimits(stagnation=100, maxTurns=None)
    return game

def Pretwa_lud():  
    game = LineGame()
    game.addBoardCircle(lines=6, length=4) 
    game.addPiecesCircle(InitialPieces.CIRCLE_CLUSTERED_NOCENTRAL)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def GolSkuish_rbg():
    game = LineGame()
    game.addBoardCircle(lines=6, length=8) 
    game.addPiecesCircle(InitialPieces.CIRCLE_CLUSTERED_NOCENTRAL)
    game.setRules(CaptureSequences.FULL, captureMandatory.MANDATORY)
    game.setLimits(stagnation=100, maxTurns=None)
    return game

def GolEkuish_lud(): 
    game = LineGame()
    game.addBoardCircle(lines=6, length=8) 
    game.addPiecesCircle(InitialPieces.CIRCLE_CLUSTERED_NOCENTRAL)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def BaraGutiBihar_lud(): 
    game = LineGame()
    game.addBoardCircle(lines=8, length=4) 
    game.addPiecesCircle(InitialPieces.CIRCLE_CLUSTERED_NOCENTRAL)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def Aiyawatstani_lud():
    game = LineGame()
    game.addBoardSquareWithDiagonals(width=9, height=9) 
    game.addPiecesSquareWithDiagonals(InitialPieces.SQUARE_4ROWS)
    game.addPieces(WHITE_PIECE, 'cy6x2', 'cy6x6')
    game.addPieces(BLACK_PIECE, 'cy6x0', 'cy6x8')
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def BaraGuti_lud():
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    game.addPiecesAlquerque(InitialPieces.SQUARE_W_TO_E_NOCENTRAL)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def BisGutiya_lud():
    game = LineGame()
    game.addBoardSquareWithDiagonals(width=9, height=9) 
    game.addPiecesSquareWithDiagonals(InitialPieces.SQUARE_W_TO_E_NOCENTRAL)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def ChalisGutiaTitagarh_lud():
    game = LineGame()
    game.addBoardAlquerque(width=9, height=9) 
    game.addPiecesAlquerque(InitialPieces.SQUARE_2ROWS)
    game.addPieces(WHITE_PIECE, 'cy6x3', 'cy6x4', 'cy6x5', 'cy6x6', 'cy6x7', 'cy6x8')
    game.addPieces(BLACK_PIECE, 'cy2x0', 'cy2x1', 'cy2x2', 'cy2x3', 'cy2x4', 'cy2x5')
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def MeurimuengRimuengPeuetPloh_lud():
    game = LineGame()
    game.addBoardAlquerque(width=9, height=9) 
    game.addPiecesAlquerque(InitialPieces.SQUARE_W_TO_E_NOCENTRAL)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def RattiChittiBakri_lud():
    game = LineGame()
    game.addBoardAlquerque(width=9, height=9) 
    game.addPiecesAlquerque(InitialPieces.SQUARE_W_TO_E_NOCENTRAL)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def Tavelspel_lud():
    game = LineGame()
    game.addBoardGrid(width=13, height=13)
    game.addPiecesGrid(InitialPieces.SQUARE_2ROWS)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def TerhuchuSmall_lud():
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    game.addPiecesAlquerque(InitialPieces.SQUARE_2ROWS)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def Tuknanavuhpi_lud():
    game = LineGame()
    game.addBoardSquareWithDiagonals(width=9, height=9) 
    game.addPiecesSquareWithDiagonals(InitialPieces.SQUARE_W_TO_E_NOCENTRAL)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def AhtarahGuti_lud(): # same as LamPusri
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, S]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesAlquerque(InitialPieces.SQUARE_E_TO_W_NOCENTRAL)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def DamSingapore_lud():
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, S]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesAlquerque(InitialPieces.SQUARE_2ROWS)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game
    
def DamHariman_lud(): # same as HewakamKeliya_lud
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, S]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesAlquerque(InitialPieces.SQUARE_2ROWS)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def DashGuti_rbg():
    game = LineGame()
    game.addBoardPoint()
    game.attachTriangle3(dir=S, height=4)
    game.attachTriangle3(dir=N, height=4)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.addNode("nodeL", piece=WHITE_PIECE, edges={E: v(0, 0, 'c')})
    game.addNode("nodeR", piece=BLACK_PIECE, edges={W: v(0, 0, 'c')})
    game.setRules(CaptureSequences.FULL, captureMandatory.MANDATORY)
    game.setLimits(stagnation=100, maxTurns=None)
    return game   

def DashGuti_lud():
    game = LineGame()
    game.addBoardPoint()
    game.attachTriangle3(dir=S, height=4)
    game.attachTriangle3(dir=N, height=4)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.addNode("nodeL", piece=WHITE_PIECE, edges={E: v(0, 0, 'c'), S: 'tsouthvy3x0', N: 'tnorthvy3x2'})
    game.addNode("nodeR", piece=BLACK_PIECE, edges={W: v(0, 0, 'c'), S: 'tsouthvy3x2', N: 'tnorthvy3x0'})
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game   

def HewakamKeliya_lud():
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, S]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesAlquerque(InitialPieces.SQUARE_2ROWS)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def KauaDorki_lud():
    game = LineGame()
    game.addBoardPoint()
    for d in[N, S]: game.attachTriangle3(dir=d, height=3)
    game.addNode("nodeW1", piece=WHITE_PIECE, edges={E: v(0, 0, 'c')})
    game.addNode("nodeW2", piece=WHITE_PIECE, edges={E: 'nodeW1'})
    game.addNode("nodeE1", piece=BLACK_PIECE, edges={W: v(0, 0, 'c')})
    game.addNode("nodeE2", piece=BLACK_PIECE, edges={W: 'nodeE1'})
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def KotuEllima_lud(): # rotated 90*
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, E, S, W]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesAlquerque(InitialPieces.SQUARE_W_TO_E_NOCENTRAL)
    for d in[S, W]: game.addPiecesTriangle(d, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    for d in[N, E]: game.addPiecesTriangle(d, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def LamPusri_lud():
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, S]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesAlquerque(InitialPieces.SQUARE_E_TO_W_NOCENTRAL)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def LauKataKati_rbg():
    game = LineGame()
    game.addBoardPoint()
    game.attachTriangle3(dir=S, height=4)
    game.attachTriangle3(dir=N, height=4)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.FULL, captureMandatory.MANDATORY)
    game.setLimits(stagnation=100, maxTurns=None)
    return game

def LauKataKati_lud():
    game = LineGame()
    game.addBoardPoint()
    game.attachTriangle3(dir=S, height=4)
    game.attachTriangle3(dir=N, height=4)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def MogulPutthan_lud():
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, S]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesAlquerque(InitialPieces.SQUARE_2ROWS)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def Peralikatuma_lud(): # rotated 90*
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, E, S, W]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesAlquerque(InitialPieces.SQUARE_2ROWS)
    for d in[S, W]: game.addPiecesTriangle(d, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    for d in[N, E]: game.addPiecesTriangle(d, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.addPieces(WHITE_PIECE, 'cy2x0')
    game.addPieces(BLACK_PIECE, 'cy2x4')
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

def Satoel_lud():
    game = LineGame()
    game.addBoardAlquerque(width=9, height=9) 
    for d in[N, S]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesAlquerque(InitialPieces.SQUARE_W_TO_E_NOCENTRAL)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

'''
def SmasandyutakankarikridaAllahabad_lud(): # TODO can make nocapture move after making capture move!! (also with other piece)
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, E, S, W, NE, NW, SE, SW]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesTriangle(S, InitialPieces.TRIANGLE_FULL_NOROOT, WHITE_PIECE)
    game.addPiecesTriangle(N, InitialPieces.TRIANGLE_FULL_NOROOT, BLACK_PIECE)
    game.setRules(CaptureSequences.SPLIT, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game    
'''

def Smasandyutakankarikrida_lud():
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, S,]: game.attachTriangle3(dir=d, height=5)
    game.addPieces(WHITE_PIECE, 'tsouthvy3x0', 'tsouthvy3x1', 'tsouthvy3x2', 'tsouthvy4x0', 'tsouthvy4x1', 'tsouthvy4x2')
    game.addPieces(BLACK_PIECE, 'tnorthvy3x0', 'tnorthvy3x1', 'tnorthvy3x2', 'tnorthvy4x0', 'tnorthvy4x1', 'tnorthvy4x2')
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game

'''
def Terhuchu_todo_lud(): # not working: Within the triangular extensions, pieces may move two places at a time, in a straight line.
    game = LineGame()
    game.addBoardAlquerque(width=5, height=5) 
    for d in[N, E, S, W, NE, NW, SE, SW]: game.attachTriangle3(dir=d, height=3)
    game.addPiecesAlquerque(InitialPieces.SQUARE_1ROWS)
    game.addPieces(WHITE_PIECE, 'cy3x1', 'cy3x2', 'cy3x3', 'tsouthvy1x0')
    game.addPieces(BLACK_PIECE, 'cy1x1', 'cy1x3', 'cy1x3', 'tnorthvy1x0')
    game.setRules(CaptureSequences.NONE, captureMandatory.NOMANDATORY)
    game.setLimits(stagnation=None, maxTurns=1250)
    return game
'''

    




if __name__ == "__main__":
    pass

    #Alquerque_rbg().printHRG()
    #Alquerque_lud().printHRG()

    #Pretwa_rbg().printHRG()
    #Pretwa_lud().printHRG()

    #GolSkuish_rbg().printHRG()
    #GolEkuish_lud().printHRG()

    #BaraGutiBihar_lud().printHRG()

    #Aiyawatstani_lud().printHRG()

    #BaraGuti_lud().printHRG()

    #BisGutiya_lud().printHRG()

    #ChalisGutiaTitagarh_lud().printHRG()

    #MerimuengRimuengPeuetPloh_lud().printHRG()

    #RattiChittiBakri_lud().printHRG()

    #Tavelspel_lud().printHRG()

    #TerhuchuSmall_lud().printHRG()

    #Tuknanavuhpi_lud().printHRG()

    #AhtarahGuti_lud().printHRG()

    #DamSingapore_lud().printHRG()

    #DamHariman_lud().printHRG()

    #DashGuti_rbg().printHRG()
    DashGuti_lud().printHRG()

    #HewakamKeliya_lud().printHRG()

    #KauaDorki_lud().printHRG()

    #KotuEllima_lud().printHRG()

    #LamPusri_lud().printHRG()

    #LauKataKati_rbg().printHRG()
    #LauKataKati_lud().printHRG()

    #MogulPutthan_lud().printHRG()

    #Peralikatuma_lud().printHRG()

    #Satoel_lud().printHRG()

    #Smasandyutakankarikrida_lud().printHRG()



