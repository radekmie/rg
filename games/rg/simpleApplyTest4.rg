// A always chooses 1.
// simpleApply is not possible from guessZero.
// Win stats of B: 50

// @disjointExhaustive guessZero : tagB0same tagB0;

//@simpleApplyExhaustive moveB draw dummytag : getDraw doneB extraB preend;
//@simpleApplyExhaustive moveB guess : guessZero;
//@simpleApplyExhaustive moveB 1 dummytag : tagB1same tagB1 doneB extraB preend;

type Player = {A,B};
type Score = {0,50,100};

type Bool = {0,1};

var key: Bool = 0;

begin, moveA: player = PlayerOrSystem(A);
//moveA, tagA0: key = 0;
tagA0, doneA: $ 0;
moveA, tagA1: key = 1;
tagA1, doneA: $ 1;

doneA, moveB: player = PlayerOrSystem(B);
moveB, getDraw: $ draw;
moveB, guessZero: $ guess;
guessZero, tagB0same: key == 0;
guessZero, tagB0: key != 0;
tagB0same, tagB0: goals[B]=Score(100);
tagB0, doneB: $ 0;

moveB, tagB1same: key == 1;
//moveB, tagB1: key != 1; removed in this variant
tagB1same, tagB1: goals[B]=Score(100);
tagB1, doneB: $ 1;

getDraw, doneB: goals[B]=Score(50);

doneB, extraB: $ dummytag;
extraB, preend: player = PlayerOrSystem(keeper);
preend, end: player = PlayerOrSystem(keeper);
