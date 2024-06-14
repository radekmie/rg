// Cycle on selectDir4. Need to change pos to v4. Only one move giving score 100.

type Player = {player};
type Score = {0, 100};
type Direction = Position -> Position;

type Position = {v0,v1,v2,v3,v4};

const left: Direction = {v0:v4,v1:v0,v2:v1,v3:v2,v4:v3,:v0};
const right: Direction = {v0:v1,v1:v2,v2:v3,v3:v4,v4:v0,:v0};

var pos: Position = v0;

begin, selectDir4: player = PlayerOrKeeper(player);
selectDir4, selectDir4: pos = Position(left[pos]);
selectDir4, selectDir4: pos = Position(right[pos]);
selectDir4, win: pos == Position(v4);

win, setScore: goals[player] = Score(100);
setScore, end: player = PlayerOrKeeper(keeper);
