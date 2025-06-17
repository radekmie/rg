type Player = {tester};
type Score = {0, 100};
type Direction = Position -> Position;
type Bool = {0, 1};
type Position = {v0,v1,v2,v3,v4,v5,v6};

const left: Direction = {v0:v0,v1:v0,v2:v1,v3:v2,v4:v3,v5:v4,v6:v5,:v0};
const right: Direction = {v0:v1,v1:v2,v2:v3,v3:v4,v4:v5,v5:v6,v6:v6,:v0};

const winPos: Position -> Bool = {v0:1,v1:1,:0};
var pos: Position = v0;

begin, loop: player = PlayerOrSystem(tester);

loop, loopLeft: pos=left[pos];
loopLeft, loop: ;
loop, loopRight: pos=right[pos];
loopRight, loop: ;

loop, win: winPos[pos] == 1;
win, showWin: $$ pos;
showWin, setWinScore: $ UNKNOWN;
setWinScore, exit: goals[tester]=100;

loop, lose: winPos[pos] == 0;
lose, showLose: $$ pos;
showLose, exit: $ UNKNOWN;

// Variant:
// loop, win: winPos[pos] == 1;
// win, setWinScore: $$ pos;
// setWinScore, showUnknown: goals[tester]=100;
// setWinScore, exit: $ UNKNOWN;

// loop, lose: winPos[pos] == 0;
// lose, showUnknown: $$ pos;
//showUnknown, exit: $ UNKNOWN;

exit, end: player = PlayerOrSystem(keeper);
