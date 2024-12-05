// Set key=0 or 1, then win if it is 1.
// Win stats: 50

//@disjointExhaustive readKey : readZero readOne;

//@simpleApplyExhaustive setKey 0 read done: setZero setKeyDone readKey readDone preend;
//@simpleApplyExhaustive setKey 1 read done: setOne setKeyDone readKey win readDone preend;

type Player = {A};
type Score = {0,100};

type Bool = {0,1};

var key: Bool = 0;

begin, setKey: player = PlayerOrKeeper(A);
setKey, setZero: key = 0;
setZero, setKeyDone: $ 0;
setKey, setOne: key = 1;
setOne, setKeyDone: $ 1;

setKeyDone, readKey: $ read;
readKey, readZero: key == 0;
readKey, readOne: key == 1;
readZero, readDone: ;
readOne, win: ;
win, readDone: goals[A]=Score(100);

readDone, preend: $ done;
preend, end: player = PlayerOrKeeper(keeper);
