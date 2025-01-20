// As simpleApplyTest1, but the second option for B is legal only if it matches the key.
// @simpleApply is possible for the second option, but not for the first one; exhaustive version not possible.
// Win stats of B: 75

//@simpleApply moveB 1: tagB1same tagB1 doneB extraB preend;

type Player = {A,B};
type Score = {0,100};

type Bool = {0,1};

var key: Bool = 0;

begin, moveA: player = PlayerOrSystem(A);
moveA, tagA0: key = 0;
tagA0, doneA: $ 0;
moveA, tagA1: key = 1;
tagA1, doneA: $ 1;

doneA, moveB: player = PlayerOrSystem(B);
moveB, tagB0same: key == 0;
moveB, tagB0: key != 0;
tagB0same, tagB0: goals[B]=Score(100);
tagB0, doneB: $ 0;

moveB, tagB1same: key == 1;
//moveB, tagB1: key != 1; removed in this variant
tagB1same, tagB1: goals[B]=Score(100);
tagB1, doneB: $ 1;

doneB, extraB: $ dummytag;
extraB, preend: player = PlayerOrSystem(keeper);
preend, end: player = PlayerOrSystem(keeper);

