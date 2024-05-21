
// Moves form: Position (F + L + R)

type Player = {white, black};
type Score = {0, 100};
type Piece = {e, b, w};
type Board = Position -> Piece;
type PieceOfPlayer = Player -> Piece;
type PieceToBool = Piece -> Bool;
type PlayerToPieceToBool = Player -> PieceToBool;
type Direction = Position -> Position;
type PlayerToPlayer = Player -> Player;
type PlayerToDirection = Player -> Direction;

type Position = {
    v00, v10, v20, v30, v40, v50, v60, v70,
    v01, v11, v21, v31, v41, v51, v61, v71,
    v02, v12, v22, v32, v42, v52, v62, v72,
    v03, v13, v23, v33, v43, v53, v63, v73,
    v04, v14, v24, v34, v44, v54, v64, v74,
    v05, v15, v25, v35, v45, v55, v65, v75,
    v06, v16, v26, v36, v46, v56, v66, v76,
    v07, v17, v27, v37, v47, v57, v67, v77
};

const up: Direction = {
   :v00,v01:v00,v02:v01,v03:v02,v04:v03,v05:v04,v06:v05,v07:v06,
v10:v10,v11:v10,v12:v11,v13:v12,v14:v13,v15:v14,v16:v15,v17:v16,
v20:v20,v21:v20,v22:v21,v23:v22,v24:v23,v25:v24,v26:v25,v27:v26,
v30:v30,v31:v30,v32:v31,v33:v32,v34:v33,v35:v34,v36:v35,v37:v36,
v40:v40,v41:v40,v42:v41,v43:v42,v44:v43,v45:v44,v46:v45,v47:v46,
v50:v50,v51:v50,v52:v51,v53:v52,v54:v53,v55:v54,v56:v55,v57:v56,
v60:v60,v61:v60,v62:v61,v63:v62,v64:v63,v65:v64,v66:v65,v67:v66,
v70:v70,v71:v70,v72:v71,v73:v72,v74:v73,v75:v74,v76:v75,v77:v76};
const upLeft: Direction = {
   :v00,v01:v01,v02:v02,v03:v03,v04:v04,v05:v05,v06:v06,v07:v07,
v10:v10,v11:v00,v12:v01,v13:v02,v14:v03,v15:v04,v16:v05,v17:v06,
v20:v20,v21:v10,v22:v11,v23:v12,v24:v13,v25:v14,v26:v15,v27:v16,
v30:v30,v31:v20,v32:v21,v33:v22,v34:v23,v35:v24,v36:v25,v37:v26,
v40:v40,v41:v30,v42:v31,v43:v32,v44:v33,v45:v34,v46:v35,v47:v36,
v50:v50,v51:v40,v52:v41,v53:v42,v54:v43,v55:v44,v56:v45,v57:v46,
v60:v60,v61:v50,v62:v51,v63:v52,v64:v53,v65:v54,v66:v55,v67:v56,
v70:v70,v71:v60,v72:v61,v73:v62,v74:v63,v75:v64,v76:v65,v77:v66};
const upRight: Direction = {
   :v00,v01:v10,v02:v11,v03:v12,v04:v13,v05:v14,v06:v15,v07:v16,
v10:v10,v11:v20,v12:v21,v13:v22,v14:v23,v15:v24,v16:v25,v17:v26,
v20:v20,v21:v30,v22:v31,v23:v32,v24:v33,v25:v34,v26:v35,v27:v36,
v30:v30,v31:v40,v32:v41,v33:v42,v34:v43,v35:v44,v36:v45,v37:v46,
v40:v40,v41:v50,v42:v51,v43:v52,v44:v53,v45:v54,v46:v55,v47:v56,
v50:v50,v51:v60,v52:v61,v53:v62,v54:v63,v55:v64,v56:v65,v57:v66,
v60:v60,v61:v70,v62:v71,v63:v72,v64:v73,v65:v74,v66:v75,v67:v76,
v70:v70,v71:v71,v72:v72,v73:v73,v74:v74,v75:v75,v76:v76,v77:v77};
const down: Direction = {
   :v01,v01:v02,v02:v03,v03:v04,v04:v05,v05:v06,v06:v07,v07:v07,
v10:v11,v11:v12,v12:v13,v13:v14,v14:v15,v15:v16,v16:v17,v17:v17,
v20:v21,v21:v22,v22:v23,v23:v24,v24:v25,v25:v26,v26:v27,v27:v27,
v30:v31,v31:v32,v32:v33,v33:v34,v34:v35,v35:v36,v36:v37,v37:v37,
v40:v41,v41:v42,v42:v43,v43:v44,v44:v45,v45:v46,v46:v47,v47:v47,
v50:v51,v51:v52,v52:v53,v53:v54,v54:v55,v55:v56,v56:v57,v57:v57,
v60:v61,v61:v62,v62:v63,v63:v64,v64:v65,v65:v66,v66:v67,v67:v67,
v70:v71,v71:v72,v72:v73,v73:v74,v74:v75,v75:v76,v76:v77,v77:v77};
const downLeft: Direction = {
   :v00,v01:v01,v02:v02,v03:v03,v04:v04,v05:v05,v06:v06,v07:v07,
v10:v01,v11:v02,v12:v03,v13:v04,v14:v05,v15:v06,v16:v07,v17:v17,
v20:v11,v21:v12,v22:v13,v23:v14,v24:v15,v25:v16,v26:v17,v27:v27,
v30:v21,v31:v22,v32:v23,v33:v24,v34:v25,v35:v26,v36:v27,v37:v37,
v40:v31,v41:v32,v42:v33,v43:v34,v44:v35,v45:v36,v46:v37,v47:v47,
v50:v41,v51:v42,v52:v43,v53:v44,v54:v45,v55:v46,v56:v47,v57:v57,
v60:v51,v61:v52,v62:v53,v63:v54,v64:v55,v65:v56,v66:v57,v67:v67,
v70:v61,v71:v62,v72:v63,v73:v64,v74:v65,v75:v66,v76:v67,v77:v77};
const downRight: Direction = {
   :v11,v01:v12,v02:v13,v03:v14,v04:v15,v05:v16,v06:v17,v07:v07,
v10:v21,v11:v22,v12:v23,v13:v24,v14:v25,v15:v26,v16:v27,v17:v17,
v20:v31,v21:v32,v22:v33,v23:v34,v24:v35,v25:v36,v26:v37,v27:v27,
v30:v41,v31:v42,v32:v43,v33:v44,v34:v45,v35:v46,v36:v47,v37:v37,
v40:v51,v41:v52,v42:v53,v43:v54,v44:v55,v45:v56,v46:v57,v47:v47,
v50:v61,v51:v62,v52:v63,v53:v64,v54:v65,v55:v66,v56:v67,v57:v57,
v60:v71,v61:v72,v62:v73,v63:v74,v64:v75,v65:v76,v66:v77,v67:v67,
v70:v70,v71:v71,v72:v72,v73:v73,v74:v74,v75:v75,v76:v76,v77:v77};

const whiteOrEmpty: PieceToBool = {w:1, e:1, :0};
const blackOrEmpty: PieceToBool = {b:1, e:1, :0};
const opponentOrEmpty: PlayerToPieceToBool = {white:blackOrEmpty, :whiteOrEmpty};
const pieceOfPlayer: PieceOfPlayer = {white:w, :b};
const FDirOfPlayer: PlayerToDirection = {white:up, :down};
const LDirOfPlayer: PlayerToDirection = {white:upLeft, :downLeft};
const RDirOfPlayer: PlayerToDirection = {white:upRight, :downRight};
const opponent: PlayerToPlayer = {white:black, :white};

var currentPlayer: Player = white;
var board: Board = {
    v00:b, v10:b, v20:b, v30:b, v40:b, v50:b, v60:b, v70:b,
    v01:b, v11:b, v21:b, v31:b, v41:b, v51:b, v61:b, v71:b,
    :e,
    v06:w, v16:w, v26:w, v36:w, v46:w, v56:w, v66:w, v76:w,
    v07:w, v17:w, v27:w, v37:w, v47:w, v57:w, v67:w, v77:w
};
var pos: Position = v00;


begin, move: ;

turn, move: ? move -> moved;
turn, lose: ! move -> moved;

move, selectPos: player = PlayerOrKeeper(currentPlayer);
selectPos, setPos(position:Position): $ position;
setPos(position:Position), checkOwn: pos = Position(position);
checkOwn, selectDir: board[pos] == pieceOfPlayer[currentPlayer];

selectDir, forwardDirCheck: board[Position(FDirOfPlayer[currentPlayer][pos])] == Piece(e);
forwardDirCheck, forwardDirSet: $ F;
forwardDirSet, forwardMove: board[pos] = Piece(e);
forwardMove, moved: pos = Position(FDirOfPlayer[currentPlayer][pos]);

selectDir, leftDirCheck: opponentOrEmpty[currentPlayer][board[LDirOfPlayer[currentPlayer][pos]]] == Bool(1);
leftDirCheck, leftDirSet: $ L;
leftDirSet, leftMove: board[pos] = Piece(e);
leftMove, moved: pos = Position(LDirOfPlayer[currentPlayer][pos]);

selectDir, rightDirCheck: opponentOrEmpty[currentPlayer][board[RDirOfPlayer[currentPlayer][pos]]] == Bool(1);
rightDirCheck, rightDirSet: $ R;
rightDirSet, rightMove: board[pos] = Piece(e);
rightMove, moved: pos = Position(RDirOfPlayer[currentPlayer][pos]);

moved, done: board[pos] = pieceOfPlayer[currentPlayer];
done, wincheck: player = PlayerOrKeeper(keeper);

wincheck, win: FDirOfPlayer[currentPlayer][pos] == pos;
wincheck, continue: FDirOfPlayer[currentPlayer][pos] != pos;
continue, turn: currentPlayer = opponent[currentPlayer];

lose, win: currentPlayer = opponent[currentPlayer];
win, score: goals[currentPlayer] = Score(100);
score, finish: goals[opponent[currentPlayer]] = Score(0);
finish, end: player = PlayerOrKeeper(keeper);

@simpleApply selectPos selectedPos(p:Position) setPos(p:Position) setFinished checkOwn forward selectDirection directionForward directionOK directionLeft directionLeftChecked directionRight directionRightChecked moved done;

@disjointExhaustive turn:move lose;
@disjointExhaustive wincheck:continue win;