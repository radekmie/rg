// Set key=0 or 1, then win if it is 1.
// Win stats: 50

//@disjoint setKey : setZero setOne

//@simpleApplyExhaustive setKey 0: setZero setKeyDone readKey;
//@simpleApplyExhaustive setKey 1: setZero setKeyDone readKey;

//@simpleApplyExhaustive readKey 0 done: readZero readDone preend;
//@simpleApplyExhaustive readKey 1 done: readOne win readDone preend;
//@simpleApplyExhaustive readKey hidden: readHidden;

type Player = {A,B};
type Score = {0,50,100};

type Int = {0,1,2};

var key: Int = 0;

begin, setKey: player = PlayerOrKeeper(A);
setKey, setZero: key = 0;
setZero, setKeyDone: $ 0;
setKey, setOne: key = 1;
setOne, setKeyDone: $ 1;
setKey, setTwo: key = 2;
setTwo, setKeyDone: $ 2;

setKeyDone, readKey: player = PlayerOrKeeper(B);
readKey, readZero: key == 0;
readKey, readOne: key == 1;
readZero, readDone: $ 0;
readOne, win: $ 1;
win, readDone: goals[B]=Score(100);

readKey, readHidden: $ hidden;
readHidden, readDone: key == 0;
readHidden, win: key == 1;
readHidden, draw: key == 2;
draw, readDone: goals[B]=Score(50);

readDone, preend: $ done;
preend, end: player = PlayerOrKeeper(keeper);
