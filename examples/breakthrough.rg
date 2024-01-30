
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
    null,
    v00, v10, v20, v30, v40, v50, v60, v70,
    v01, v11, v21, v31, v41, v51, v61, v71,
    v02, v12, v22, v32, v42, v52, v62, v72,
    v03, v13, v23, v33, v43, v53, v63, v73,
    v04, v14, v24, v34, v44, v54, v64, v74,
    v05, v15, v25, v35, v45, v55, v65, v75,
    v06, v16, v26, v36, v46, v56, v66, v76,
    v07, v17, v27, v37, v47, v57, v67, v77
};

const left: Direction = {:null,
    v10:v00, v20:v10, v30:v20, v40:v30, v50:v40, v60:v50, v70:v60,
    v11:v01, v21:v11, v31:v21, v41:v31, v51:v41, v61:v51, v71:v61,
    v12:v02, v22:v12, v32:v22, v42:v32, v52:v42, v62:v52, v72:v62,
    v13:v03, v23:v13, v33:v23, v43:v33, v53:v43, v63:v53, v73:v63,
    v14:v04, v24:v14, v34:v24, v44:v34, v54:v44, v64:v54, v74:v64,
    v15:v05, v25:v15, v35:v25, v45:v35, v55:v45, v65:v55, v75:v65,
    v16:v06, v26:v16, v36:v26, v46:v36, v56:v46, v66:v56, v76:v66,
    v17:v07, v27:v17, v37:v27, v47:v37, v57:v47, v67:v57, v77:v67
};
const right: Direction = {:null,
    v00:v10, v10:v20, v20:v30, v30:v40, v40:v50, v50:v60, v60:v70,
    v01:v11, v11:v21, v21:v31, v31:v41, v41:v51, v51:v61, v61:v71,
    v02:v12, v12:v22, v22:v32, v32:v42, v42:v52, v52:v62, v62:v72,
    v03:v13, v13:v23, v23:v33, v33:v43, v43:v53, v53:v63, v63:v73,
    v04:v14, v14:v24, v24:v34, v34:v44, v44:v54, v54:v64, v64:v74,
    v05:v15, v15:v25, v25:v35, v35:v45, v45:v55, v55:v65, v65:v75,
    v06:v16, v16:v26, v26:v36, v36:v46, v46:v56, v56:v66, v66:v76,
    v07:v17, v17:v27, v27:v37, v37:v47, v47:v57, v57:v67, v67:v77
};
const up: Direction = {:null,
    v01:v00, v02:v01, v03:v02, v04:v03, v05:v04, v06:v05, v07:v06,
    v11:v10, v12:v11, v13:v12, v14:v13, v15:v14, v16:v15, v17:v16,
    v21:v20, v22:v21, v23:v22, v24:v23, v25:v24, v26:v25, v27:v26,
    v31:v30, v32:v31, v33:v32, v34:v33, v35:v34, v36:v35, v37:v36,
    v41:v40, v42:v41, v43:v42, v44:v43, v45:v44, v46:v45, v47:v46,
    v51:v50, v52:v51, v53:v52, v54:v53, v55:v54, v56:v55, v57:v56,
    v61:v60, v62:v61, v63:v62, v64:v63, v65:v64, v66:v65, v67:v66,
    v71:v70, v72:v71, v73:v72, v74:v73, v75:v74, v76:v75, v77:v76
};
const down: Direction = {:null,
    v00:v01, v01:v02, v02:v03, v03:v04, v04:v05, v05:v06, v06:v07,
    v10:v11, v11:v12, v12:v13, v13:v14, v14:v15, v15:v16, v16:v17,
    v20:v21, v21:v22, v22:v23, v23:v24, v24:v25, v25:v26, v26:v27,
    v30:v31, v31:v32, v32:v33, v33:v34, v34:v35, v35:v36, v36:v37,
    v40:v41, v41:v42, v42:v43, v43:v44, v44:v45, v45:v46, v46:v47,
    v50:v51, v51:v52, v52:v53, v53:v54, v54:v55, v55:v56, v56:v57,
    v60:v61, v61:v62, v62:v63, v63:v64, v64:v65, v65:v66, v66:v67,
    v70:v71, v71:v72, v72:v73, v73:v74, v74:v75, v75:v76, v76:v77
};

const whiteOrEmpty: PieceToBool = {w:1, e:1, :0};
const blackOrEmpty: PieceToBool = {b:1, e:1, :0};
const opponentOrEmpty: PlayerToPieceToBool = {white:blackOrEmpty, :whiteOrEmpty};
const pieceOfPlayer: PieceOfPlayer = {white:w, :b};
const directionOfPlayer: PlayerToDirection = {white:up, :down};
const opponent: PlayerToPlayer = {white:black, :white};

var turnPlayer: Player = white;
var board: Board = {
    v00:b, v10:b, v20:b, v30:b, v40:b, v50:b, v60:b, v70:b,
    v01:b, v11:b, v21:b, v31:b, v41:b, v51:b, v61:b, v71:b,
    :e,
    v06:w, v16:w, v26:w, v36:w, v46:w, v56:w, v66:w, v76:w,
    v07:w, v17:w, v27:w, v37:w, v47:w, v57:w, v67:w, v77:w
};
var pos: Position = v00;


begin, turn: ;

turn, move: ? move -> moved;
turn, lose: ! move -> moved;

move, selectPos: player = PlayerOrKeeper(turnPlayer);
selectPos, selectedPos(position:Position): $ position;
selectedPos(position:Position), setPos(position:Position): position != Position(null);
setPos(position:Position), setFinished: pos = Position(position);
setFinished, checkOwn: board[pos] == pieceOfPlayer[turnPlayer];
checkOwn, forward: board[pos] = Piece(e);
forward, selectDirection: pos = Position(directionOfPlayer[turnPlayer][pos]);

selectDirection, directionForward: $ F;
directionForward, directionOK: board[pos] == Piece(e);

selectDirection, directionLeft: $ L;
directionLeft, directionLeftChecked: left[pos] != Position(null);
directionLeftChecked, directionOK: pos = Position(left[pos]);

selectDirection, directionRight: $ R;
directionRight, directionRightChecked: right[pos] != Position(null);
directionRightChecked, directionOK: pos = Position(right[pos]);

directionOK, moved: opponentOrEmpty[turnPlayer][board[pos]] == Bool(1);

moved, done: board[pos] = pieceOfPlayer[turnPlayer];
done, wincheck: player = PlayerOrKeeper(keeper);

wincheck, win: directionOfPlayer[turnPlayer][pos] == Position(null);
wincheck, continue: directionOfPlayer[turnPlayer][pos] != Position(null);
continue, turn: turnPlayer = opponent[turnPlayer];

lose, win: turnPlayer = opponent[turnPlayer];
win, score: goals[turnPlayer] = Score(100);
score, finish: goals[opponent[turnPlayer]] = Score(0);
finish, end: player = PlayerOrKeeper(keeper);

@unique begin turn move lose selectPos selectedPos(p:Position) setPos(p:Position) setFinished checkOwn forward selectDirection directionForward directionOK directionLeft directionLeftChecked directionRight directionRightChecked moved done wincheck continue turn lose win score finish end;
@simpleApply selectPos selectedPos(p:Position) setPos(p:Position) setFinished checkOwn forward selectDirection directionForward directionOK directionLeft directionLeftChecked directionRight directionRightChecked moved done;

@tagIndex selectPos : 0
@tagIndex selectDirection : 1