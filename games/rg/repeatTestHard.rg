type Player = {tester};
type Score = {0, 100};
type Direction = Position -> Position;

type Position = {v0,v1,v2,v3,v4,v5,v6};

const left: Direction = {v0:v0,v1:v0,v2:v1,v3:v2,v4:v3,v5:v4,v6:v5,:v0};
const right: Direction = {v0:v1,v1:v2,v2:v3,v3:v4,v4:v5,v5:v6,v6:v6,:v0};

var pos: Position = v0;

begin, chooseTag: player = PlayerOrKeeper(tester);
chooseTag, choosenLeft: $ A;
choosenLeft, loop: pos=Position(v0);
chooseTag, choosenRight: $ B;
choosenRight, loop: pos=Position(v4);

loop, loopLeft: pos=left[pos];
loopLeft, loop: ;
loop, loopRight: pos=right[pos];
loopRight, loop: ;

loop, win: pos==Position(v2);
win, setscore: goals[tester]=100;
setscore, exit: $ WIN;
loop, lose: $ LOSE;
lose, show(p:Position): p==pos;
show(p:Position), exit: $ p;
exit, end: player = PlayerOrKeeper(keeper);