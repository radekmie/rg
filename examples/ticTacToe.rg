// TicTacToe (direct LL example)

type Player = {X,O};
type PlayerOrKeeper = {keeper,X,O};
type Score = {0,50,100};
type Goals = Player -> Score;
type Coord = {0,1,2};
type Piece = {e,X,O};
type ColumnOfBoard = Coord -> Piece;
type Board = Coord -> ColumnOfBoard;
type PlayerToPlayer = Player -> Player;
type Bool = {0,1};
type Visibility = Player -> Bool;

const opponent: PlayerToPlayer = {X:O, :X};

var player: PlayerOrKeeper = keeper;
var playerTurn: Player = X;
var goals: Goals = {:50};
var posX: CoordX = 0;
var posY: CoordY = 0;
var visibility: Visibility = {:1};

const otherInLine1: Coord -> Coord = {0:1, :0};
const otherInLine2: Coord -> Coord = {:2, 2:1};
const onLRDiagonal: Coord -> Coord -> Bool = {0:{0:1,:0}, 1:{1:1,:0}, 2:{2:1,:0}, :{:0}};
const onRLDiagonal: Coord -> Coord -> Bool = {2:{0:1,:0}, 1:{1:1,:0}, 0:{2:1,:0}, :{:0}};


begin,turn: playerTurn = Player(X);

turn,move: move rev-> set;
turn,end: move not-> set;

move,chooseX: player = PlayerOrKeeper(playerTurn);
// chooseX,chooseX#Coord: posX = Coord(#Coord);
// chooseX#Coord,chooseY: ;
// chooseY,chooseY#Coord: posY = Coord(#Coord);
// chooseY#Coord,check: ;
check,set: board[posX][posY] == Piece(e);
set,endmove: board[posX][posY] = Piece(playerTurn);

endmove,checkwin: player = PlayerOrKeeper(keeper);
checkwin,win: checkline rev-> endcheckline;
chechwin,nextturn: checkline not-> endcheckline;
nextturn,turn: playerTurn = opponent[playerTurn];

checkline,checklineH1:;
checklineH1,checklineH2: board[otherInLine1[posX]][posY] == Piece(playerTurn);
checklineH2,endcheckline: board[otherInLine2[posX]][posY] == Piece(playerTurn);
checkline,checklineV1:;
checklineV1,checklineV2: board[posX][otherInLine1[posY]] == Piece(playerTurn);
checklineV2,endcheckline: board[posX][otherInLine2[posY]] == Piece(playerTurn);
checkline,checklineLR1: onLRDiagonal[posX][posY] == 1;
checklineLR1,checklineLR2: board[otherInLine1[posX]][otherInLine1[posY]] == Piece(playerTurn);
checklineLR2,endcheckline: board[otherInLine2[posX]][otherInLine2[posY]] == Piece(playerTurn);
checkline,checklineRL1: onRLDiagonal[posX][posY] == 1;
checklineRL1,checklineRL2: board[otherInLine1[posX]][otherInLine2[posY]] == Piece(playerTurn);
checklineRL2,endcheckline: board[otherInLine2[posX]][otherInLine1[posY]] == Piece(playerTurn);

win,win1: goals[playerTurn] = Score(100);
win1,end: goals[opponent[playerTurn]] = Score(0);
end,end: player = PlayerOrKeeper(keeper);
