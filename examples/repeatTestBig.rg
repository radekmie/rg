// Cycle on selectDir4. Need to change pos to v4. Only one move giving score 100.

type Player = {player};
type Score = {0, 100};
type Direction = Position -> Position;

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
   :v00,v02:v01,v03:v02,v04:v03,v05:v04,v06:v05,v07:v06,
v11:v10,v12:v11,v13:v12,v14:v13,v15:v14,v16:v15,v17:v16,
v21:v20,v22:v21,v23:v22,v24:v23,v25:v24,v26:v25,v27:v26,
v31:v30,v32:v31,v33:v32,v34:v33,v35:v34,v36:v35,v37:v36,
v41:v40,v42:v41,v43:v42,v44:v43,v45:v44,v46:v45,v47:v46,
v51:v50,v52:v51,v53:v52,v54:v53,v55:v54,v56:v55,v57:v56,
v61:v60,v62:v61,v63:v62,v64:v63,v65:v64,v66:v65,v67:v66,
v71:v70,v72:v71,v73:v72,v74:v73,v75:v74,v76:v75,v77:v76};
const down: Direction = {
   :v01,v01:v02,v02:v03,v03:v04,v04:v05,v05:v06,v06:v07,
v10:v11,v11:v12,v12:v13,v13:v14,v14:v15,v15:v16,v16:v17,
v20:v21,v21:v22,v22:v23,v23:v24,v24:v25,v25:v26,v26:v27,
v30:v31,v31:v32,v32:v33,v33:v34,v34:v35,v35:v36,v36:v37,
v40:v41,v41:v42,v42:v43,v43:v44,v44:v45,v45:v46,v46:v47,
v50:v51,v51:v52,v52:v53,v53:v54,v54:v55,v55:v56,v56:v57,
v60:v61,v61:v62,v62:v63,v63:v64,v64:v65,v65:v66,v66:v67,
v70:v71,v71:v72,v72:v73,v73:v74,v74:v75,v75:v76,v76:v77};
const left: Direction = {
   :v00,v11:v01,v12:v02,v13:v03,v14:v04,v15:v05,v16:v06,v17:v07,
v20:v10,v21:v11,v22:v12,v23:v13,v24:v14,v25:v15,v26:v16,v27:v17,
v30:v20,v31:v21,v32:v22,v33:v23,v34:v24,v35:v25,v36:v26,v37:v27,
v40:v30,v41:v31,v42:v32,v43:v33,v44:v34,v45:v35,v46:v36,v47:v37,
v50:v40,v51:v41,v52:v42,v53:v43,v54:v44,v55:v45,v56:v46,v57:v47,
v60:v50,v61:v51,v62:v52,v63:v53,v64:v54,v65:v55,v66:v56,v67:v57,
v70:v60,v71:v61,v72:v62,v73:v63,v74:v64,v75:v65,v76:v66,v77:v67};
const right: Direction = {
   :v10,v01:v11,v02:v12,v03:v13,v04:v14,v05:v15,v06:v16,v07:v17,
v10:v20,v11:v21,v12:v22,v13:v23,v14:v24,v15:v25,v16:v26,v17:v27,
v20:v30,v21:v31,v22:v32,v23:v33,v24:v34,v25:v35,v26:v36,v27:v37,
v30:v40,v31:v41,v32:v42,v33:v43,v34:v44,v35:v45,v36:v46,v37:v47,
v40:v50,v41:v51,v42:v52,v43:v53,v44:v54,v45:v55,v46:v56,v47:v57,
v50:v60,v51:v61,v52:v62,v53:v63,v54:v64,v55:v65,v56:v66,v57:v67,
v60:v70,v61:v71,v62:v72,v63:v73,v64:v74,v65:v75,v66:v76,v67:v77};

var pos: Position = v00;

begin,main: player = PlayerOrKeeper(player);

main,goUp: ;
goUp,goUp: pos = Position(up[pos]);
goUp,main: ;

main,goDown: ;
goDown,goDown: pos = Position(down[pos]);
goDown,main: ;

main,goLeft: ;
goLeft,goLeft: pos = Position(left[pos]);
goLeft,main: ;

main,goRight: ;
goRight,goRight: pos = Position(right[pos]);
goRight,main: ;

main, win1: pos == v66;
win1, setScore: goals[player] = Score(100);
main, win2: pos == v77;
win2, setScore: goals[player] = Score(100);
setScore, end: player = PlayerOrKeeper(keeper);
