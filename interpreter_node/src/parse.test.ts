import { describe, expect, test } from 'vitest';

import { parse } from './parse';
import { Language, Settings, noFlagsEnabled } from './types';

function createRun(settings: Settings, definitions: string[] = []) {
  // TODO: These are builtins added to all games.
  definitions = [
    'type Bool = { 0, 1 };',
    'type Goals = Player -> Score;',
    'type Player = { you };',
    'type PlayerOrKeeper = { you, keeper };',
    'type Score = { zero };',
    'type Visibility = Player -> Bool;',
    'var goals: Goals = { :zero };',
    'var player: PlayerOrKeeper = keeper;',
    'var visible: Visibility = { :1 };',
    ...definitions,
  ];

  return async (source: string[]) => {
    const gameSource = [...definitions, ...source].join('\n');
    const game = await parse(gameSource, settings);
    return definitions
      .reduce(
        (source, definition) => source.replace(definition, ''),
        game.sourceRgFormatted,
      )
      .replace(/\n\n+/g, '\n')
      .trim();
  };
}

describe('parse (.rg)', () => {
  describe('ast.Cast', () => {
    const run = createRun({ extension: Language.rg, flags: noFlagsEnabled }, [
      'type A = { 0 };',
      'type B = { 1 };',
      'type C = A -> B;',
      'type D = B -> C;',
      'var a: A = 0;',
      'var b: B = 1;',
      'var c: C = { :1 };',
      'var d: D = { :{ :1 } };',
      'begin, end: ;',
    ]);

    test.each([
      'a = a;',
      'a = A(a);',
      'A(a) = a;',
      'A(a) = A(a);',
      'b = b;',
      'b = B(b);',
      'B(b) = b;',
      'B(b) = B(b);',
      'c = c;',
      'c = C(c);',
      'C(c) = c;',
      'C(c) = C(c);',
      'd = d;',
      'd = D(d);',
      'D(d) = d;',
      'D(d) = D(d);',
      'c = d[b];',
      'c = d[B(b)];',
      'c = D(d)[b];',
      'c = D(d)[B(b)];',
      'c = C(d[b]);',
      'c = C(d[B(b)]);',
      'c = C(D(d)[b]);',
      'c = C(D(d)[B(b)]);',
      'C(c) = d[b];',
      'C(c) = d[B(b)];',
      'C(c) = D(d)[b];',
      'C(c) = D(d)[B(b)];',
      'C(c) = C(d[b]);',
      'C(c) = C(d[B(b)]);',
      'C(c) = C(D(d)[b]);',
      'C(c) = C(D(d)[B(b)]);',
      'b = d[b][a];',
      'b = d[b][A(a)];',
      'b = d[B(b)][a];',
      'b = d[B(b)][A(a)];',
      'b = c[a];',
      'b = c[A(a)];',
      'b = D(d)[b][a];',
      'b = D(d)[b][A(a)];',
      'b = D(d)[B(b)][a];',
      'b = D(d)[B(b)][A(a)];',
      'b = C(d[b])[a];',
      'b = C(d[b])[A(a)];',
      'b = C(d[B(b)])[a];',
      'b = C(d[B(b)])[A(a)];',
      'b = C(c)[a];',
      'b = C(c)[A(a)];',
      'b = C(D(d)[b])[a];',
      'b = C(D(d)[b])[A(a)];',
      'b = C(D(d)[B(b)])[a];',
      'b = C(D(d)[B(b)])[A(a)];',
      'b = B(d[b][a]);',
      'b = B(d[b][A(a)]);',
      'b = B(d[B(b)][a]);',
      'b = B(d[B(b)][A(a)]);',
      'b = B(c[a]);',
      'b = B(c[A(a)]);',
      'b = B(D(d)[b][a]);',
      'b = B(D(d)[b][A(a)]);',
      'b = B(D(d)[B(b)][a]);',
      'b = B(D(d)[B(b)][A(a)]);',
      'b = B(C(d[b])[a]);',
      'b = B(C(d[b])[A(a)]);',
      'b = B(C(d[B(b)])[a]);',
      'b = B(C(d[B(b)])[A(a)]);',
      'b = B(C(c)[a]);',
      'b = B(C(c)[A(a)]);',
      'b = B(C(D(d)[b])[a]);',
      'b = B(C(D(d)[b])[A(a)]);',
      'b = B(C(D(d)[B(b)])[a]);',
      'b = B(C(D(d)[B(b)])[A(a)]);',
      'B(b) = d[b][a];',
      'B(b) = d[b][A(a)];',
      'B(b) = d[B(b)][a];',
      'B(b) = d[B(b)][A(a)];',
      'B(b) = c[a];',
      'B(b) = c[A(a)];',
      'B(b) = D(d)[b][a];',
      'B(b) = D(d)[b][A(a)];',
      'B(b) = D(d)[B(b)][a];',
      'B(b) = C(d[b])[a];',
      'B(b) = C(d[b])[A(a)];',
      'B(b) = C(d[B(b)])[a];',
      'B(b) = C(d[B(b)])[A(a)];',
      'B(b) = C(c)[a];',
      'B(b) = C(c)[A(a)];',
      'B(b) = C(D(d)[b])[a];',
      'B(b) = C(D(d)[b])[A(a)];',
      'B(b) = C(D(d)[B(b)])[a];',
      'B(b) = B(d[b][a]);',
      'B(b) = B(d[b][A(a)]);',
      'B(b) = B(d[B(b)][a]);',
      'B(b) = B(d[B(b)][A(a)]);',
      'B(b) = B(c[a]);',
      'B(b) = B(c[A(a)]);',
      'B(b) = B(D(d)[b][a]);',
      'B(b) = B(D(d)[b][A(a)]);',
      'B(b) = B(D(d)[B(b)][a]);',
      'B(b) = B(C(d[b])[a]);',
      'B(b) = B(C(d[b])[A(a)]);',
      'B(b) = B(C(d[B(b)])[a]);',
      'B(b) = B(C(d[B(b)])[A(a)]);',
      'B(b) = B(C(c)[a]);',
      'B(b) = B(C(c)[A(a)]);',
      'B(b) = B(C(D(d)[b])[a]);',
      'B(b) = B(C(D(d)[b])[A(a)]);',
      'B(b) = B(C(D(d)[B(b)])[a]);',
      'B(b) = B(C(D(d)[B(b)])[A(a)]);',
    ])('%s', async label => {
      const edge = `x, y: ${label}`;
      await expect(run([edge])).resolves.toBe(edge);
    });
  });
});

describe('--addExplicitCasts', () => {
  const run = createRun(
    {
      extension: Language.rg,
      flags: { ...noFlagsEnabled, addExplicitCasts: true },
    },
    [
      'type A = { 0 };',
      'type B = { 1 };',
      'type C = A -> B;',
      'type D = B -> C;',
      'var a: A = 0;',
      'var b: B = 1;',
      'var c: C = { :1 };',
      'var d: D = { :{ :1 } };',
      'begin, end: ;',
    ],
  );

  test.each([
    'b = d[b][a];',
    'b = d[b][A(a)];',
    'b = d[B(b)][a];',
    'b = d[B(b)][A(a)];',
    'b = D(d)[b][a];',
    'b = D(d)[b][A(a)];',
    'b = D(d)[B(b)][a];',
    'b = D(d)[B(b)][A(a)];',
    'b = B(d[b][a]);',
    'b = B(d[b][A(a)]);',
    'b = B(d[B(b)][a]);',
    'b = B(d[B(b)][A(a)]);',
    'b = B(D(d)[b][a]);',
    'b = B(D(d)[b][A(a)]);',
    'b = B(D(d)[B(b)][a]);',
    'b = B(D(d)[B(b)][A(a)]);',
    'b = B(C(d[b])[a]);',
    'b = B(C(d[b])[A(a)]);',
    'b = B(C(d[B(b)])[a]);',
    'b = B(C(d[B(b)])[A(a)]);',
    'b = B(C(D(d)[b])[a]);',
    'b = B(C(D(d)[b])[A(a)]);',
    'b = B(C(D(d)[B(b)])[a]);',
    'b = B(C(D(d)[B(b)])[A(a)]);',
    'B(b) = d[b][a];',
    'B(b) = d[b][A(a)];',
    'B(b) = d[B(b)][a];',
    'B(b) = d[B(b)][A(a)];',
    'B(b) = D(d)[b][a];',
    'B(b) = D(d)[b][A(a)];',
    'B(b) = D(d)[B(b)][a];',
    'B(b) = B(d[b][a]);',
    'B(b) = B(d[b][A(a)]);',
    'B(b) = B(d[B(b)][a]);',
    'B(b) = B(d[B(b)][A(a)]);',
    'B(b) = B(D(d)[b][a]);',
    'B(b) = B(D(d)[b][A(a)]);',
    'B(b) = B(D(d)[B(b)][a]);',
    'B(b) = B(C(d[b])[a]);',
    'B(b) = B(C(d[b])[A(a)]);',
    'B(b) = B(C(d[B(b)])[a]);',
    'B(b) = B(C(d[B(b)])[A(a)]);',
    'B(b) = B(C(D(d)[b])[a]);',
    'B(b) = B(C(D(d)[b])[A(a)]);',
    'B(b) = B(C(D(d)[B(b)])[a]);',
    'B(b) = B(C(D(d)[B(b)])[A(a)]);',
  ])('%s', async label => {
    const input = `x, y: ${label}`;
    const output = 'x, y: B(b) = B(C(D(d)[B(b)])[A(a)]);';
    await expect(run([input])).resolves.toBe(output);
  });
});

describe('--compactSkipEdges', () => {
  const run = createRun({
    extension: Language.rg,
    flags: { ...noFlagsEnabled, compactSkipEdges: true },
  });

  test('prefix', async () => {
    await expect(run(['begin, b: ;', 'b, c: x == x;', 'c, end: y == y;']))
      .resolves.toMatchInlineSnapshot(`
      "begin, c: x == x;
      c, end: y == y;"
    `);
  });

  test('infix', async () => {
    await expect(run(['begin, b: x == x;', 'b, c: ;', 'c, end: y == y;']))
      .resolves.toMatchInlineSnapshot(`
      "begin, c: x == x;
      c, end: y == y;"
    `);
  });

  test('suffix', async () => {
    await expect(run(['begin, b: x == x;', 'b, c: y == y;', 'c, end: ;']))
      .resolves.toMatchInlineSnapshot(`
      "begin, b: x == x;
      b, end: y == y;"
    `);
  });
});

describe('--expandGeneratorNodes', () => {
  const run = createRun(
    {
      extension: Language.rg,
      flags: { ...noFlagsEnabled, expandGeneratorNodes: true },
    },
    [
      'type T1 = { 1, 2 };',
      'type T2 = { 3, 4 };',
      'type T3 = { 5, 6 };',
      'type T4 = { 1, 2, 3, 4, 5, 6 };',
      'const map: T4 -> T4 -> T4 = { :{ :1 } };',
      'begin, end: ;',
    ],
  );

  test('one lhs bind', async () => {
    await expect(run(['a(x: T1), b: x == x;'])).resolves.toMatchInlineSnapshot(`
      "a__bind__1, b: 1 == 1;
      a__bind__2, b: 2 == 2;"
    `);
  });

  test('one rhs bind', async () => {
    await expect(run(['a, b(x: T1): x == x;'])).resolves.toMatchInlineSnapshot(`
      "a, b__bind__1: 1 == 1;
      a, b__bind__2: 2 == 2;"
    `);
  });

  test('one lhs and one rhs bind (equal)', async () => {
    await expect(run(['a(x: T1), b(x: T1): x == x;'])).resolves
      .toMatchInlineSnapshot(`
      "a__bind__1, b__bind__1: 1 == 1;
      a__bind__2, b__bind__2: 2 == 2;"
    `);
  });

  test('one lhs and one rhs bind (different)', async () => {
    await expect(run(['a(x: T1), b(y: T2): T4(x) == T4(y);'])).resolves
      .toMatchInlineSnapshot(`
      "a__bind__1, b__bind__3: T4(1) == T4(3);
      a__bind__1, b__bind__4: T4(1) == T4(4);
      a__bind__2, b__bind__3: T4(2) == T4(3);
      a__bind__2, b__bind__4: T4(2) == T4(4);"
    `);
  });

  test('two lhs binds', async () => {
    await expect(run(['a(x: T1)(y: T2), b: T4(x) == T4(y);'])).resolves
      .toMatchInlineSnapshot(`
      "a__bind__1__bind__3, b: T4(1) == T4(3);
      a__bind__1__bind__4, b: T4(1) == T4(4);
      a__bind__2__bind__3, b: T4(2) == T4(3);
      a__bind__2__bind__4, b: T4(2) == T4(4);"
    `);
  });

  test('two rhs binds', async () => {
    await expect(run(['a, b(x: T1)(y: T2): T4(x) == T4(y);'])).resolves
      .toMatchInlineSnapshot(`
      "a, b__bind__1__bind__3: T4(1) == T4(3);
      a, b__bind__1__bind__4: T4(1) == T4(4);
      a, b__bind__2__bind__3: T4(2) == T4(3);
      a, b__bind__2__bind__4: T4(2) == T4(4);"
    `);
  });

  test('two lhs and one rhs bind (equal)', async () => {
    await expect(run(['a(x: T1)(y: T2), b(x: T1): T4(x) == T4(y);'])).resolves
      .toMatchInlineSnapshot(`
      "a__bind__1__bind__3, b__bind__1: T4(1) == T4(3);
      a__bind__1__bind__4, b__bind__1: T4(1) == T4(4);
      a__bind__2__bind__3, b__bind__2: T4(2) == T4(3);
      a__bind__2__bind__4, b__bind__2: T4(2) == T4(4);"
    `);
    await expect(run(['a(x: T1)(y: T2), b(y: T2): T4(x) == T4(y);'])).resolves
      .toMatchInlineSnapshot(`
      "a__bind__1__bind__3, b__bind__3: T4(1) == T4(3);
      a__bind__1__bind__4, b__bind__4: T4(1) == T4(4);
      a__bind__2__bind__3, b__bind__3: T4(2) == T4(3);
      a__bind__2__bind__4, b__bind__4: T4(2) == T4(4);"
    `);
  });

  test('two lhs and one rhs bind (different)', async () => {
    await expect(run(['a(x: T1)(y: T2), b(z: T3): map[x][y] == T4(z);']))
      .resolves.toMatchInlineSnapshot(`
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

  test('one lhs and two rhs binds (equal)', async () => {
    await expect(run(['a(x: T1), b(x: T1)(y: T2): T4(x) == T4(y);'])).resolves
      .toMatchInlineSnapshot(`
      "a__bind__1, b__bind__1__bind__3: T4(1) == T4(3);
      a__bind__1, b__bind__1__bind__4: T4(1) == T4(4);
      a__bind__2, b__bind__2__bind__3: T4(2) == T4(3);
      a__bind__2, b__bind__2__bind__4: T4(2) == T4(4);"
    `);
    await expect(run(['a(y: T2), b(x: T1)(y: T2): T4(x) == T4(y);'])).resolves
      .toMatchInlineSnapshot(`
      "a__bind__3, b__bind__1__bind__3: T4(1) == T4(3);
      a__bind__4, b__bind__1__bind__4: T4(1) == T4(4);
      a__bind__3, b__bind__2__bind__3: T4(2) == T4(3);
      a__bind__4, b__bind__2__bind__4: T4(2) == T4(4);"
    `);
  });

  test('one lhs and two rhs binds (different)', async () => {
    await expect(run(['a(x: T1), b(y: T2)(z: T3): map[x][y] == T4(z);']))
      .resolves.toMatchInlineSnapshot(`
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
describe('--inlineReachability', () => {
  const run = createRun(
    {
      extension: Language.rg,
      flags: { ...noFlagsEnabled, inlineReachability: true },
    },
    ['begin, end: ;'],
  );

  test('basic', async () => {
    await expect(run(['a, b: ? x -> y;', 'x, y: 1 == 1;'])).resolves
      .toMatchInlineSnapshot(`
      "a, __gen_1_reachability_x_y: ;
      x, y: 1 == 1;
      __gen_1_reachability_x_y, b: 1 == 1;"
    `);
    await expect(run(['a, b: ? x -> z;', 'x, y: 1 == 1;', 'y, z: 2 == 2;']))
      .resolves.toMatchInlineSnapshot(`
      "a, __gen_1_reachability_x_z: ;
      x, y: 1 == 1;
      y, z: 2 == 2;
      __gen_1_reachability_x_z, __gen_2_y: 1 == 1;
      __gen_2_y, b: 2 == 2;"
    `);
    await expect(run(['a, b: ? x -> z;', 'x, y: ;', 'y, z: 2 == 2;'])).resolves
      .toMatchInlineSnapshot(`
      "a, __gen_1_reachability_x_z: ;
      x, y: ;
      y, z: 2 == 2;
      __gen_1_reachability_x_z, __gen_2_y: ;
      __gen_2_y, b: 2 == 2;"
    `);
    await expect(run(['a, b: ? x -> z;', 'x, y: 1 == 1;', 'y, z: ;'])).resolves
      .toMatchInlineSnapshot(`
      "a, __gen_1_reachability_x_z: ;
      x, y: 1 == 1;
      y, z: ;
      __gen_1_reachability_x_z, __gen_2_y: 1 == 1;
      __gen_2_y, b: ;"
    `);
  });

  test('inlines exclusive comparison', async () => {
    await expect(
      run([
        'x, y: ? a -> d;',
        'a, b: 1 == 1;',
        'a, c: 1 != 1;',
        'b, d: ;',
        'c, d: ;',
      ]),
    ).resolves.toMatchInlineSnapshot(`
      "x, __gen_1_reachability_a_d: ;
      a, b: 1 == 1;
      a, c: 1 != 1;
      b, d: ;
      c, d: ;
      __gen_1_reachability_a_d, __gen_2_b: 1 == 1;
      __gen_1_reachability_a_d, __gen_3_c: 1 != 1;
      __gen_3_c, y: ;
      __gen_2_b, y: ;"
    `);
  });

  test("doesn't inline non-exclusive comparison", async () => {
    await expect(
      run([
        'type T = {1, 2};',
        'var v: T = 1;',
        'x, y: ? a -> d;',
        'a, b: v == 1;',
        'a, c: v != 2;',
        'b, d: ;',
        'c, d: ;',
      ]),
    ).resolves.toMatchInlineSnapshot(`
      "type T = { 1, 2 };
      var v: T = 1;
      x, y: ? a -> d;
      a, b: v == 1;
      a, c: v != 2;
      b, d: ;
      c, d: ;"
    `);
  });

  test('inlines exclusive reachability', async () => {
    await expect(
      run([
        'x, y: ? a -> d;',
        'a, b: ? e -> f;',
        'a, c: ! e -> f;',
        'b, d: ;',
        'c, d: ;',
        'e, f: ;',
      ]),
    ).resolves.toMatchInlineSnapshot(`
      "x, __gen_1_reachability_a_d: ;
      a, __gen_4_reachability_e_f: ;
      a, __gen_5_reachability_e_f: ;
      b, d: ;
      c, d: ;
      e, f: ;
      __gen_1_reachability_a_d, __gen_6_reachability_e_f: ;
      __gen_1_reachability_a_d, __gen_7_reachability_e_f: ;
      __gen_3_c, y: ;
      __gen_2_b, y: ;
      __gen_4_reachability_e_f, b: ;
      __gen_5_reachability_e_f, c: ;
      __gen_6_reachability_e_f, __gen_2_b: ;
      __gen_7_reachability_e_f, __gen_3_c: ;"
    `);
  });

  test("doesn't inline non-exclusive reachability", async () => {
    // technically it is exclusive but that can be seen only after
    // further analysis
    await expect(
      run([
        'x, y: ? a -> d;',
        'a, b: ? e -> f;',
        'a, c: ! e -> g;',
        'b, d: ;',
        'c, d: ;',
        'e, f: ;',
        'e, g: ;',
      ]),
    ).resolves.toMatchInlineSnapshot(`
      "x, y: ? a -> d;
      a, b: ? e -> f;
      a, c: ! e -> g;
      b, d: ;
      c, d: ;
      e, f: ;
      e, g: ;"
    `);
  });

  test('may copy trailing edges', async () => {
    await expect(
      run([
        'x, y: ? a -> c;',
        'a, b: 0 == 0;',
        'b, c: 1 == 1;',
        'c, d: 2 == 2;',
      ]),
    ).resolves.toMatchInlineSnapshot(`
      "x, __gen_1_reachability_a_c: ;
      a, b: 0 == 0;
      b, c: 1 == 1;
      c, d: 2 == 2;
      __gen_1_reachability_a_c, __gen_2_b: 0 == 0;
      __gen_2_b, y: 1 == 1;
      y, __gen_3_d: 2 == 2;"
    `);
  });
});

describe('--normalizeTypes', () => {
  const run = createRun(
    {
      extension: Language.rg,
      flags: { ...noFlagsEnabled, normalizeTypes: true },
    },
    ['type A = { 0 };', 'begin, end: ;'],
  );

  test('constant declarations', async () => {
    await expect(run(['const b: A -> A = { :0 };'])).resolves
      .toMatchInlineSnapshot(`
      "type Type1 = A -> A;
      const b: Type1 = { :0 };"
    `);
    await expect(run(['const b: A -> A -> A = { :{ :0 } };'])).resolves
      .toMatchInlineSnapshot(`
      "type Type1 = A -> A;
      type Type2 = A -> Type1;
      const b: Type2 = { :{ :0 } };"
    `);
  });

  test('variable declarations', async () => {
    await expect(run(['var b: A -> A = { :0 };'])).resolves
      .toMatchInlineSnapshot(`
      "type Type1 = A -> A;
      var b: Type1 = { :0 };"
    `);
    await expect(run(['var b: A -> A -> A = { :{ :0 } };'])).resolves
      .toMatchInlineSnapshot(`
      "type Type1 = A -> A;
      type Type2 = A -> Type1;
      var b: Type2 = { :{ :0 } };"
    `);
  });

  test('type declarations', async () => {
    await expect(run(['type B = A -> A;'])).resolves.toMatchInlineSnapshot(
      '"type B = A -> A;"',
    );
    await expect(run(['type B = A -> A -> A;'])).resolves
      .toMatchInlineSnapshot(`
      "type B = A -> Type1;
      type Type1 = A -> A;"
    `);
    await expect(run(['type B = A -> A -> A -> A;'])).resolves
      .toMatchInlineSnapshot(`
      "type B = A -> Type2;
      type Type1 = A -> A;
      type Type2 = A -> Type1;"
    `);
  });

  test('existing types', async () => {
    await expect(run(['type B = A -> A -> A;', 'type Type1 = A;'])).resolves
      .toMatchInlineSnapshot(`
      "type B = A -> Type2;
      type Type1 = A;
      type Type2 = A -> A;"
    `);
  });
});
