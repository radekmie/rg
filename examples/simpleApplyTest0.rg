type Player = {P};
type Score = {0,100};

type Bool = {0,1};
var key: Bool = 0;

//@simpleApplyExhaustive move A 0 : key0 tagA done end;
//@simpleApplyExhaustive move B 1 : key1 tagB done end;

begin, move: player = PlayerOrKeeper(P);

move, key0: key = 0;
key0, tagA: $ A;
tagA, done: $ 0;

move, key1: key = 1;
key1, tagB: $ B;
tagB, done: $ 1;

done, end: player = PlayerOrKeeper(keeper);
