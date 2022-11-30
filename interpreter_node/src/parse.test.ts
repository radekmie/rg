import { parse } from '../src/parse';
import { Extension, Settings, noFlagsEnabled } from '../src/types';

function createRun(settings: Settings, definitions: string[] = []) {
  return (source: string[]) =>
    definitions
      .reduce(
        (source, definition) => source.replace(definition, ''),
        parse([...definitions, ...source].join('\n'), settings)
          .sourceRgFormatted,
      )
      .trim();
}

describe('compactSkipEdges', () => {
  const run = createRun(
    {
      extension: Extension.rg,
      flags: { ...noFlagsEnabled, compactSkipEdges: true },
    },
    ['begin, end: ;'],
  );

  test('prefix', () => {
    expect(run(['a, b: ;', 'b, c: x == x;', 'c, d: y == y;']))
      .toMatchInlineSnapshot(`
      "a, c: x == x;
      c, d: y == y;"
    `);
  });

  test('infix', () => {
    expect(run(['a, b: x == x;', 'b, c: ;', 'c, d: y == y;']))
      .toMatchInlineSnapshot(`
      "a, c: x == x;
      c, d: y == y;"
    `);
  });

  test('suffix', () => {
    expect(run(['a, b: x == x;', 'b, c: y == y;', 'c, d: ;']))
      .toMatchInlineSnapshot(`
      "a, b: x == x;
      b, d: y == y;"
    `);
  });
});

describe('expandGeneratorNodes', () => {
  const run = createRun(
    {
      extension: Extension.rg,
      flags: { ...noFlagsEnabled, expandGeneratorNodes: true },
    },
    [
      'type T1 = { 1, 2 };',
      'type T2 = { 3, 4 };',
      'type T3 = { 5, 6 };',
      'type T4 = { 1, 2, 3, 4, 5, 6 };',
      'const map: T4 -> T4 -> T4 = { :1 };',
      'begin, end: ;',
    ],
  );

  test('one lhs bind', () => {
    expect(run(['a(x: T1), b: x == x;'])).toMatchInlineSnapshot(`
      "a__bind__1, b: 1 == 1;
      a__bind__2, b: 2 == 2;"
    `);
  });

  test('one rhs bind', () => {
    expect(run(['a, b(x: T1): x == x;'])).toMatchInlineSnapshot(`
      "a, b__bind__1: 1 == 1;
      a, b__bind__2: 2 == 2;"
    `);
  });

  test('one lhs and one rhs bind (equal)', () => {
    expect(run(['a(x: T1), b(x: T1): x == x;'])).toMatchInlineSnapshot(`
      "a__bind__1, b__bind__1: 1 == 1;
      a__bind__2, b__bind__2: 2 == 2;"
    `);
  });

  test('one lhs and one rhs bind (different)', () => {
    expect(run(['a(x: T1), b(y: T2): T4(x) == T4(y);'])).toMatchInlineSnapshot(`
      "a__bind__1, b__bind__3: T4(1) == T4(3);
      a__bind__1, b__bind__4: T4(1) == T4(4);
      a__bind__2, b__bind__3: T4(2) == T4(3);
      a__bind__2, b__bind__4: T4(2) == T4(4);"
    `);
  });

  test('two lhs binds', () => {
    expect(run(['a(x: T1)(y: T2), b: T4(x) == T4(y);'])).toMatchInlineSnapshot(`
      "a__bind__1__bind__3, b: T4(1) == T4(3);
      a__bind__1__bind__4, b: T4(1) == T4(4);
      a__bind__2__bind__3, b: T4(2) == T4(3);
      a__bind__2__bind__4, b: T4(2) == T4(4);"
    `);
  });

  test('two rhs binds', () => {
    expect(run(['a, b(x: T1)(y: T2): T4(x) == T4(y);'])).toMatchInlineSnapshot(`
      "a, b__bind__1__bind__3: T4(1) == T4(3);
      a, b__bind__1__bind__4: T4(1) == T4(4);
      a, b__bind__2__bind__3: T4(2) == T4(3);
      a, b__bind__2__bind__4: T4(2) == T4(4);"
    `);
  });

  test('two lhs and one rhs bind (equal)', () => {
    expect(run(['a(x: T1)(y: T2), b(x: T1): T4(x) == T4(y);']))
      .toMatchInlineSnapshot(`
      "a__bind__1__bind__3, b__bind__1: T4(1) == T4(3);
      a__bind__1__bind__4, b__bind__1: T4(1) == T4(4);
      a__bind__2__bind__3, b__bind__2: T4(2) == T4(3);
      a__bind__2__bind__4, b__bind__2: T4(2) == T4(4);"
    `);
    expect(run(['a(x: T1)(y: T2), b(y: T2): T4(x) == T4(y);']))
      .toMatchInlineSnapshot(`
      "a__bind__1__bind__3, b__bind__3: T4(1) == T4(3);
      a__bind__1__bind__4, b__bind__4: T4(1) == T4(4);
      a__bind__2__bind__3, b__bind__3: T4(2) == T4(3);
      a__bind__2__bind__4, b__bind__4: T4(2) == T4(4);"
    `);
  });

  test('two lhs and one rhs bind (different)', () => {
    expect(run(['a(x: T1)(y: T2), b(z: T3): map[x][y] == T4(z);']))
      .toMatchInlineSnapshot(`
      "a__bind__1__bind__3, b__bind__5: map[1][3] == T4(5);
      a__bind__1__bind__3, b__bind__6: map[1][3] == T4(6);
      a__bind__1__bind__4, b__bind__5: map[1][4] == T4(5);
      a__bind__1__bind__4, b__bind__6: map[1][4] == T4(6);
      a__bind__2__bind__3, b__bind__5: map[2][3] == T4(5);
      a__bind__2__bind__3, b__bind__6: map[2][3] == T4(6);
      a__bind__2__bind__4, b__bind__5: map[2][4] == T4(5);
      a__bind__2__bind__4, b__bind__6: map[2][4] == T4(6);"
    `);
  });

  test('one lhs and two rhs binds (equal)', () => {
    expect(run(['a(x: T1), b(x: T1)(y: T2): T4(x) == T4(y);']))
      .toMatchInlineSnapshot(`
      "a__bind__1, b__bind__1__bind__3: T4(1) == T4(3);
      a__bind__1, b__bind__1__bind__4: T4(1) == T4(4);
      a__bind__2, b__bind__2__bind__3: T4(2) == T4(3);
      a__bind__2, b__bind__2__bind__4: T4(2) == T4(4);"
    `);
    expect(run(['a(y: T2), b(x: T1)(y: T2): T4(x) == T4(y);']))
      .toMatchInlineSnapshot(`
      "a__bind__3, b__bind__1__bind__3: T4(1) == T4(3);
      a__bind__3, b__bind__2__bind__3: T4(2) == T4(3);
      a__bind__4, b__bind__1__bind__4: T4(1) == T4(4);
      a__bind__4, b__bind__2__bind__4: T4(2) == T4(4);"
    `);
  });

  test('one lhs and two rhs binds (different)', () => {
    expect(run(['a(x: T1), b(y: T2)(z: T3): map[x][y] == T4(z);']))
      .toMatchInlineSnapshot(`
      "a__bind__1, b__bind__3__bind__5: map[1][3] == T4(5);
      a__bind__1, b__bind__3__bind__6: map[1][3] == T4(6);
      a__bind__1, b__bind__4__bind__5: map[1][4] == T4(5);
      a__bind__1, b__bind__4__bind__6: map[1][4] == T4(6);
      a__bind__2, b__bind__3__bind__5: map[2][3] == T4(5);
      a__bind__2, b__bind__3__bind__6: map[2][3] == T4(6);
      a__bind__2, b__bind__4__bind__5: map[2][4] == T4(5);
      a__bind__2, b__bind__4__bind__6: map[2][4] == T4(6);"
    `);
  });
});

// 1. Check each entrypoint of a subautomaton.
// 2. It has exactly one exclusive path.
// 3. It has no assignements.
describe.skip('inlineReachability', () => {
  const run = createRun(
    {
      extension: Extension.rg,
      flags: noFlagsEnabled,
    },
    ['begin, end: ;'],
  );

  test('basic', () => {
    expect(run(['a, b: ? x -> y;', 'x, y: 1 == 1;'])).toMatchInlineSnapshot(
      '"a, b: 1 == 1;"',
    );
    expect(
      run(['a, b: ? x -> z;', 'x, y: 1 == 1;', 'y, z: 2 == 2;']),
    ).toMatchInlineSnapshot('"a, temp: 1 == 1; temp, b: 2 == 2;"');
    expect(
      run(['a, b: ? x -> z;', 'x, y: ;', 'y, z: 2 == 2;']),
    ).toMatchInlineSnapshot('"a, temp: ; temp, b: 2 == 2;"');
    expect(
      run(['a, b: ? x -> z;', 'x, y: 1 == 1;', 'y, z: ;']),
    ).toMatchInlineSnapshot('"a, temp: 1 == 1; temp, b: ;"');
  });

  test('exclusive comparision', () => {
    expect(
      run([
        'x, y: ? a -> d;',
        'a, b: 1 == 1;',
        'a, c: 1 != 1;',
        'b, d: ;',
        'c, d: ;',
      ]),
    ).toMatchInlineSnapshot(
      '"x, _b: 1 == 1; x, _c: 1 != 1; _b, y: ; _c, y: ;"',
    );
  });

  test('exclusive reachability', () => {
    expect(
      run([
        'x, y: ? a -> d;',
        'a, b: ? e -> f;',
        'a, c: ! e -> f;',
        'b, d: ;',
        'c, d: ;',
        'e, f: ;',
      ]),
    ).toMatchInlineSnapshot(
      '"x, _b: ? e -> f; x, _c: ! e -> f; _b, y: ; _c, y: ; e, f: ;"',
    );
  });
});

describe('skipSelfAssignments', () => {
  const run = createRun(
    {
      extension: Extension.rg,
      flags: { ...noFlagsEnabled, skipSelfAssignments: true },
    },
    ['type T = { x };', 'var map: T -> T = { :x };', 'begin, end: ;'],
  );

  test('basic', () => {
    expect(run(['a, b: x = x;'])).toMatchInlineSnapshot('"a, b: ;"');
  });

  test('basic with cast', () => {
    expect(run(['a, b: x = T(x);'])).toMatchInlineSnapshot('"a, b: ;"');
  });

  test('access', () => {
    expect(run(['a, b: map[x] = map[x];'])).toMatchInlineSnapshot('"a, b: ;"');
  });

  test('access with cast', () => {
    expect(run(['a, b: map[x] = T(map[x]);'])).toMatchInlineSnapshot(
      '"a, b: ;"',
    );
    expect(run(['a, b: map[x] = map[T(x)];'])).toMatchInlineSnapshot(
      '"a, b: ;"',
    );
  });
});

describe('joinForkSuffixes', () => {
  // TODO test: don't join nodes referenced in reachability
  // TODO test: don't join nodes with different bindings
  const run = createRun(
    {
      extension: Extension.rg,
      flags: { ...noFlagsEnabled, joinForkSuffixes: true },
    },
    ['begin, end: ;'],
  );

  test('fork and join: small', () => {
    expect(
      run([
        '1, l1: 1 == 1;',
        '1, r1: 2 == 2;',
        'l1, l2: 4 == 4;',
        'l2, 2: 0 == 0;',
        'r1, r2: 5 == 5;',
        'r2, 2: 0 == 0;',
        '2, 3: 7 == 7;',
      ]),
    ).toMatchInlineSnapshot(`
      "1, l1: 1 == 1;
      1, r1: 2 == 2;
      l1, l2: 4 == 4;
      l2, 2: 0 == 0;
      r1, l2: 5 == 5;
      2, 3: 7 == 7;"
    `);
  });

  test('fork and join: bigger', () => {
    expect(
      run([
        'start, a0: branch0 == branch0;',
        'a5, end: 5 == 5;',
        'a0, a1: 0 == 0;',
        'a1, a2: 1 == 1;',
        'a2, a3: 2 == 2;',
        'a3, a4: 3 == 3;',
        'a4, a5: 4 == 4;',
        'start, b0: branch1 == branch1;',
        'b5, end: 5 == 5;',
        'b0, b1: 0 == 0;',
        'b1, b2: 1 == 1;',
        'b2, b3: 2 == 2;',
        'b3, b4: 3 == 3;',
        'b4, b5: 4 == 4;',
        'start, c0: branch2 == branch2;',
        'c5, end: 5 == 5;',
        'c0, c1: 0 == 0;',
        'c1, c2: 1 == 1;',
        'c2, c3: 2 == 2;',
        'c3, c4: 3 == 3;',
        'c4, c5: 4 == 4;',
        'start, d0: branch3 == branch3;',
        'd5, end: 5 == 5;',
        'd0, d1: 0 == 0;',
        'd1, d2: 1 == 1;',
        'd2, d3: 2 == 2;',
        'd3, d4: 3 == 3;',
        'd4, d5: 4 == 4;',
      ]),
    ).toMatchInlineSnapshot(`
      "start, a0: branch0 == branch0;
      a5, end: 5 == 5;
      a0, d1: 0 == 0;
      a4, a5: 4 == 4;
      start, b0: branch1 == branch1;
      b0, d1: 0 == 0;
      b3, a4: 3 == 3;
      start, c0: branch2 == branch2;
      c0, d1: 0 == 0;
      c2, b3: 2 == 2;
      start, d0: branch3 == branch3;
      d0, d1: 0 == 0;
      d1, c2: 1 == 1;"
    `);
  });

  test("don't join if both branches have more outgoing edges", () => {
    expect(
      run([
        '1, l1: 1 == 1;',
        '1, r1: 2 == 2;',
        'l1, l2: 4 == 4;',
        'l2, 2: 0 == 0;',
        'r1, r2: 5 == 5;',
        'r2, 2: 0 == 0;',
        '2, 3: 7 == 7;',
        'l2, 4: 0 == 0;',
        'r2, 4: 0 == 0;',
      ]),
    ).toMatchInlineSnapshot(`
      "1, l1: 1 == 1;
      1, r1: 2 == 2;
      l1, l2: 4 == 4;
      l2, 2: 0 == 0;
      r1, r2: 5 == 5;
      r2, 2: 0 == 0;
      2, 3: 7 == 7;
      l2, 4: 0 == 0;
      r2, 4: 0 == 0;"
    `);
  });

  test("don't join if one branch has more outgoing edges", () => {
    expect(
      run([
        '1, l1: 1 == 1;',
        '1, r1: 2 == 2;',
        'l1, l2: 4 == 4;',
        'l2, 2: 0 == 0;',
        'r1, r2: 5 == 5;',
        'r2, 2: 0 == 0;',
        '2, 3: 7 == 7;',
        'l2, 4: 0 == 0;',
      ]),
    ).toMatchInlineSnapshot(`
      "1, l1: 1 == 1;
      1, r1: 2 == 2;
      l1, l2: 4 == 4;
      l2, 2: 0 == 0;
      r1, r2: 5 == 5;
      r2, 2: 0 == 0;
      2, 3: 7 == 7;
      l2, 4: 0 == 0;"
    `);
  });

  test("don't join if both branches have more incoming edges", () => {
    expect(
      run([
        '1, l1: 1 == 1;',
        '1, r1: 2 == 2;',
        'l1, l2: 4 == 4;',
        'l2, 2: 0 == 0;',
        'r1, r2: 5 == 5;',
        'r2, 2: 0 == 0;',
        '2, 3: 7 == 7;',
        '4, l2: 0 == 0;',
        '4, r2: 0 == 0;',
      ]),
    ).toMatchInlineSnapshot(`
      "1, l1: 1 == 1;
      1, r1: 2 == 2;
      l1, l2: 4 == 4;
      l2, 2: 0 == 0;
      r1, r2: 5 == 5;
      r2, 2: 0 == 0;
      2, 3: 7 == 7;
      4, l2: 0 == 0;
      4, r2: 0 == 0;"
    `);
  });

  test('join if only one branch has more incoming edges', () => {
    expect(
      run([
        '1, l1: 1 == 1;',
        '1, r1: 2 == 2;',
        'l1, l2: 4 == 4;',
        'l2, 2: 0 == 0;',
        'r1, r2: 5 == 5;',
        'r2, 2: 0 == 0;',
        '2, 3: 7 == 7;',
        '4, l2: 0 == 0;',
      ]),
    ).toMatchInlineSnapshot(`
      "1, l1: 1 == 1;
      1, r1: 2 == 2;
      l1, l2: 4 == 4;
      l2, 2: 0 == 0;
      r1, l2: 5 == 5;
      2, 3: 7 == 7;
      4, l2: 0 == 0;"
    `);
  });

  test("don't create multiple edges between nodes", () => {
    expect(
      run([
        '1, l1: 0 == 0;',
        '1, r1: 0 == 0;',
        'l1, 2: 1 == 1;',
        'r1, 2: 1 == 1;',
        '2, 3: 7 == 7;',
      ]),
    ).toMatchInlineSnapshot(`
      "1, l1: 0 == 0;
      1, r1: 0 == 0;
      l1, 2: 1 == 1;
      r1, 2: 1 == 1;
      2, 3: 7 == 7;"
    `);
  });

  test('shape from breakthrough.rbg', () => {
    expect(
      run([
        '11, 9: 3 == 3;',
        '9, 12: 5 == 5;',
        '9, 18: 1 == 1;',
        '9, 20: 2 == 2;',
        '18, 15: 3 == 3;',
        '20, 15: 3 == 3;',
        '15, 12: 4 == 4;',
        '15, 23: ;',
        '23, 12: 5 == 5;',
      ]),
    ).toMatchInlineSnapshot(`
      "11, 9: 3 == 3;
      9, 12: 5 == 5;
      9, 18: 1 == 1;
      9, 20: 2 == 2;
      18, 15: 3 == 3;
      20, 15: 3 == 3;
      15, 12: 4 == 4;
      15, 23: ;
      23, 12: 5 == 5;"
    `);
  });
});
