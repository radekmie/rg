import * as utils from '../../utils';
import * as ast from '../ast';
import * as t from './inlineReachability';

function pretty(object: unknown) {
  return utils.pretty(object, { colors: false });
}

function makeEdgeName(n: string): ast.EdgeName {
  return ast.EdgeName({ parts: [ast.Literal({ identifier: n })] });
}

function edge(
  lhs: ast.EdgeName,
  rhs: ast.EdgeName,
  label: ast.EdgeLabel,
): ast.EdgeDeclaration {
  return ast.EdgeDeclaration({ lhs, rhs, label });
}

function makeEdge(
  lhs: string,
  rhs: string,
  label: ast.EdgeLabel,
): ast.EdgeDeclaration {
  return edge(makeEdgeName(lhs), makeEdgeName(rhs), label);
}

function makeEdges(
  chain: ast.EdgeName[],
  labels?: ast.EdgeLabel[],
): ast.EdgeDeclaration[] {
  return [...Array(chain.length - 1).keys()].map(i =>
    edge(chain[i], chain[i + 1], labels ? labels[i] : ast.Skip({})),
  );
}

function makeLabel(n: number): ast.EdgeLabel {
  return ast.Assignment({
    lhs: ast.Reference({ identifier: 'x' + n.toString() }),
    rhs: ast.Reference({ identifier: 'y' }),
  });
}

describe('inlineReachability', () => {
  test('findAcceptablePaths should find a simple path in a chain', () => {
    const nodes = ['x', 'y', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes);

    expect(pretty(t.findAcceptablePaths(edges, nodes[0], nodes.reverse()[0])))
      .toMatchInlineSnapshot(`
      "{
        ok: true,
        value: [
          {
            kind: 'EdgeDeclaration',
            label: { kind: 'Skip' },
            lhs: {
              kind: 'EdgeName',
              parts: [ { identifier: 'x', kind: 'Literal' } ]
            },
            rhs: {
              kind: 'EdgeName',
              parts: [ { identifier: 'y', kind: 'Literal' } ]
            }
          },
          {
            kind: 'EdgeDeclaration',
            label: { kind: 'Skip' },
            lhs: {
              kind: 'EdgeName',
              parts: [ { identifier: 'y', kind: 'Literal' } ]
            },
            rhs: {
              kind: 'EdgeName',
              parts: [ { identifier: 'z', kind: 'Literal' } ]
            }
          }
        ]
      }"
    `);
  });

  test('findAcceptablePaths should reject multiple paths', () => {
    const nodes1 = ['x', 'y', 'z'].map(makeEdgeName);
    const nodes2 = ['x', 'y1', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes1).concat(makeEdges(nodes2));

    expect(
      t.findAcceptablePaths(edges, nodes1[0], nodes1.reverse()[0]),
    ).toMatchInlineSnapshot(`
      Object {
        "error": "can't ensure single path at runtime",
        "ok": false,
      }
    `);
  });

  test.skip('substituteWithPaths should reuse edge when substituting single edge', () => {
    const nodes = ['x', 'y', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes);

    t.substituteWithPaths(
      edges,
      edges[0],
      makeEdges(['a', 'b'].map(makeEdgeName), [makeLabel(1)]),
      makeEdgeName('a'),
      makeEdgeName('b'),
    );

    expect(pretty(edges)).toMatchInlineSnapshot(`
      "[
        {
          kind: 'EdgeDeclaration',
          label: {
            kind: 'Assignment',
            lhs: { identifier: 'x1', kind: 'Reference' },
            rhs: { identifier: 'y', kind: 'Reference' }
          },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'x', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'y', kind: 'Literal' } ]
          }
        },
        {
          kind: 'EdgeDeclaration',
          label: { kind: 'Skip' },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'y', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'z', kind: 'Literal' } ]
          }
        }
      ]"
    `);
  });

  test.skip('substituteWithPaths should create one edge when substituting two', () => {
    const nodes = ['x', 'y', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes);
    const replacement = makeEdges(
      ['a', 'b', 'c'].map(makeEdgeName),
      [1, 2].map(makeLabel),
    );

    t.substituteWithPaths(
      edges,
      edges[0],
      replacement,
      makeEdgeName('a'),
      makeEdgeName('c'),
    );

    expect(pretty(edges)).toMatchInlineSnapshot(`
      "[
        {
          kind: 'EdgeDeclaration',
          label: {
            kind: 'Assignment',
            lhs: { identifier: 'x1', kind: 'Reference' },
            rhs: { identifier: 'y', kind: 'Reference' }
          },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'x', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: '__gen_1', kind: 'Literal' } ]
          }
        },
        {
          kind: 'EdgeDeclaration',
          label: { kind: 'Skip' },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'y', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'z', kind: 'Literal' } ]
          }
        },
        {
          kind: 'EdgeDeclaration',
          label: {
            kind: 'Assignment',
            lhs: { identifier: 'x2', kind: 'Reference' },
            rhs: { identifier: 'y', kind: 'Reference' }
          },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: '__gen_1', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'y', kind: 'Literal' } ]
          }
        }
      ]"
    `);
  });

  test('substituteWithPaths should replace existing edge with new ones', () => {
    const nodes = ['x', 'y', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes);
    const replacement = makeEdges(
      ['a', 'b', 'c'].map(makeEdgeName),
      [1, 2].map(makeLabel),
    );

    t.substituteWithPaths(
      edges,
      edges[0],
      replacement,
      makeEdgeName('a'),
      makeEdgeName('c'),
    );

    expect(pretty(edges)).toMatchInlineSnapshot(`
      "[
        {
          kind: 'EdgeDeclaration',
          label: { kind: 'Skip' },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: '__gen_1_ignoreme', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'y', kind: 'Literal' } ]
          }
        },
        {
          kind: 'EdgeDeclaration',
          label: { kind: 'Skip' },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'y', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'z', kind: 'Literal' } ]
          }
        },
        {
          kind: 'EdgeDeclaration',
          label: {
            kind: 'Assignment',
            lhs: { identifier: 'x1', kind: 'Reference' },
            rhs: { identifier: 'y', kind: 'Reference' }
          },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'x', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: '__gen_2_b', kind: 'Literal' } ]
          }
        },
        {
          kind: 'EdgeDeclaration',
          label: {
            kind: 'Assignment',
            lhs: { identifier: 'x2', kind: 'Reference' },
            rhs: { identifier: 'y', kind: 'Reference' }
          },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: '__gen_2_b', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'y', kind: 'Literal' } ]
          }
        }
      ]"
    `);
  });

  test('substituteWithPaths should replace existing edge with new one', () => {
    const nodes = ['x', 'y', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes);
    const replacement = [makeEdge('a', 'b', makeLabel(1))];

    t.substituteWithPaths(
      edges,
      edges[0],
      replacement,
      makeEdgeName('a'),
      makeEdgeName('b'),
    );

    expect(pretty(edges)).toMatchInlineSnapshot(`
      "[
        {
          kind: 'EdgeDeclaration',
          label: { kind: 'Skip' },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: '__gen_3_ignoreme', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'y', kind: 'Literal' } ]
          }
        },
        {
          kind: 'EdgeDeclaration',
          label: { kind: 'Skip' },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'y', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'z', kind: 'Literal' } ]
          }
        },
        {
          kind: 'EdgeDeclaration',
          label: {
            kind: 'Assignment',
            lhs: { identifier: 'x1', kind: 'Reference' },
            rhs: { identifier: 'y', kind: 'Reference' }
          },
          lhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'x', kind: 'Literal' } ]
          },
          rhs: {
            kind: 'EdgeName',
            parts: [ { identifier: 'y', kind: 'Literal' } ]
          }
        }
      ]"
    `);
  });
});
