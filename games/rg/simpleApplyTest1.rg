// Player A has two choices for key: 0 or 1 and emits it as tag.
// Player B has the same two choices but wins when the choice agrees with the key.
// Note: all nodes can be @unique; tagB0 and tagB1 when considering @disjoint.
// Note: Player B does not have simpleApply because setting the score depends on the game state.
// Win stats of B: 50

// @simpleApplyExhaustive moveA 0: tagA0 doneA moveB
// @simpleApplyExhaustive moveA 1: tagA1 doneA moveB
// Or
// @simpleApplyExhaustive moveA 0: tagA0 doneA
// @simpleApplyExhaustive moveA 1: tagA1 doneA
// @simpleApplyExhaustive doneA: moveB

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
moveB, tagB1: key != 1;
tagB1same, tagB1: goals[B]=Score(100);
tagB1, doneB: $ 1;

doneB, extraB: $ dummytag;
extraB, preend: player = PlayerOrSystem(keeper);
preend, end: player = PlayerOrSystem(keeper);
