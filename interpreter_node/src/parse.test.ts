import { parse } from '../src/parse';
import { Extension } from '../src/types';

describe('compactSkipEdges', () => {
  function run(source: string) {
    return parse(source, {
      extension: Extension.rg,
      flags: {
        compactSkipEdges: true,
        expandGeneratorNodes: true,
        mangleSymbols: false,
        removeSelfAssignments: false,
        reuseFunctions: false,
      },
    }).sourceRgFormatted;
  }

  test('prefix', () => {
    expect(run('a, b: ; b, c: x == x; c, d: y == y;')).toMatchInlineSnapshot(`
      "a, c: x == x;
      c, d: y == y;"
    `);
  });

  test('infix', () => {
    expect(run('a, b: x == x; b, c: ; c, d: y == y;')).toMatchInlineSnapshot(`
      "a, c: x == x;
      c, d: y == y;"
    `);
  });

  test('suffix', () => {
    expect(run('a, b: x == x; b, c: y == y; c, d: ;')).toMatchInlineSnapshot(`
      "a, b: x == x;
      b, d: y == y;"
    `);
  });
});

describe('expandGeneratorNodes', () => {
  const typeDefinitions = `
    type T1 = { 1, 2 };
    type T2 = { 3, 4 };
    type T3 = { 5, 6 };
  `;

  function run(source: string) {
    return parse(typeDefinitions + source, {
      extension: Extension.rg,
      flags: {
        compactSkipEdges: false,
        expandGeneratorNodes: true,
        mangleSymbols: false,
        removeSelfAssignments: false,
        reuseFunctions: false,
      },
    }).sourceRgFormatted;
  }

  test('one lhs bind', () => {
    expect(run('a(x: T1), b: x == x;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a__bind__1, b: 1 == 1;
      a__bind__2, b: 2 == 2;"
    `);
  });

  test('one rhs bind', () => {
    expect(run('a, b(x: T1): x == x;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a, b__bind__1: 1 == 1;
      a, b__bind__2: 2 == 2;"
    `);
  });

  test('one lhs and one rhs bind (equal)', () => {
    expect(run('a(x: T1), b(x: T1): x == x;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a__bind__1, b__bind__1: 1 == 1;
      a__bind__2, b__bind__2: 2 == 2;"
    `);
  });

  test('one lhs and one rhs bind (different)', () => {
    expect(run('a(x: T1), b(y: T2): x == y;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a__bind__1, b__bind__3: 1 == 3;
      a__bind__1, b__bind__4: 1 == 4;
      a__bind__2, b__bind__3: 2 == 3;
      a__bind__2, b__bind__4: 2 == 4;"
    `);
  });

  test('two lhs binds', () => {
    expect(run('a(x: T1)(y: T2), b: x == y;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a__bind__1__bind__3, b: 1 == 3;
      a__bind__1__bind__4, b: 1 == 4;
      a__bind__2__bind__3, b: 2 == 3;
      a__bind__2__bind__4, b: 2 == 4;"
    `);
  });

  test('two rhs binds', () => {
    expect(run('a, b(x: T1)(y: T2): x == y;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a, b__bind__1__bind__3: 1 == 3;
      a, b__bind__1__bind__4: 1 == 4;
      a, b__bind__2__bind__3: 2 == 3;
      a, b__bind__2__bind__4: 2 == 4;"
    `);
  });

  test('two lhs and one rhs bind (equal)', () => {
    expect(run('a(x: T1)(y: T2), b(x: T1): x == y;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a__bind__1__bind__3, b__bind__1: 1 == 3;
      a__bind__1__bind__4, b__bind__1: 1 == 4;
      a__bind__2__bind__3, b__bind__2: 2 == 3;
      a__bind__2__bind__4, b__bind__2: 2 == 4;"
    `);
    expect(run('a(x: T1)(y: T2), b(y: T2): x == y;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a__bind__1__bind__3, b__bind__3: 1 == 3;
      a__bind__1__bind__4, b__bind__4: 1 == 4;
      a__bind__2__bind__3, b__bind__3: 2 == 3;
      a__bind__2__bind__4, b__bind__4: 2 == 4;"
    `);
  });

  test('two lhs and one rhs bind (different)', () => {
    expect(run('a(x: T1)(y: T2), b(z: T3): x[y] == z;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a__bind__1__bind__3, b__bind__5: 1[3] == 5;
      a__bind__1__bind__3, b__bind__6: 1[3] == 6;
      a__bind__1__bind__4, b__bind__5: 1[4] == 5;
      a__bind__1__bind__4, b__bind__6: 1[4] == 6;
      a__bind__2__bind__3, b__bind__5: 2[3] == 5;
      a__bind__2__bind__3, b__bind__6: 2[3] == 6;
      a__bind__2__bind__4, b__bind__5: 2[4] == 5;
      a__bind__2__bind__4, b__bind__6: 2[4] == 6;"
    `);
  });

  test('one lhs and two rhs binds (equal)', () => {
    expect(run('a(x: T1), b(x: T1)(y: T2): x == y;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a__bind__1, b__bind__1__bind__3: 1 == 3;
      a__bind__1, b__bind__1__bind__4: 1 == 4;
      a__bind__2, b__bind__2__bind__3: 2 == 3;
      a__bind__2, b__bind__2__bind__4: 2 == 4;"
    `);
    expect(run('a(y: T2), b(x: T1)(y: T2): x == y;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a__bind__3, b__bind__1__bind__3: 1 == 3;
      a__bind__3, b__bind__2__bind__3: 2 == 3;
      a__bind__4, b__bind__1__bind__4: 1 == 4;
      a__bind__4, b__bind__2__bind__4: 2 == 4;"
    `);
  });

  test('one lhs and two rhs binds (different)', () => {
    expect(run('a(x: T1), b(y: T2)(z: T3): x[y] == z;')).toMatchInlineSnapshot(`
      "type T1 = { 1, 2 };
      type T2 = { 3, 4 };
      type T3 = { 5, 6 };

      a__bind__1, b__bind__3__bind__5: 1[3] == 5;
      a__bind__1, b__bind__3__bind__6: 1[3] == 6;
      a__bind__1, b__bind__4__bind__5: 1[4] == 5;
      a__bind__1, b__bind__4__bind__6: 1[4] == 6;
      a__bind__2, b__bind__3__bind__5: 2[3] == 5;
      a__bind__2, b__bind__3__bind__6: 2[3] == 6;
      a__bind__2, b__bind__4__bind__5: 2[4] == 5;
      a__bind__2, b__bind__4__bind__6: 2[4] == 6;"
    `);
  });
});
