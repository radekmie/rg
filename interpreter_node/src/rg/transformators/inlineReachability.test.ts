import * as utils from '../../utils';
import * as ast from '../ast';

import * as t from './inlineReachability';

function pretty(object: unknown) {
  return utils.pretty(object, { colors: false });
}

function mkEdgeName(n: string): ast.EdgeName {
  return ast.EdgeName({ parts: [ast.Literal({ identifier: n })] });
}

function edge(
  lhs: ast.EdgeName,
  rhs: ast.EdgeName,
  label: ast.EdgeLabel,
): ast.EdgeDeclaration {
  return ast.EdgeDeclaration({ lhs: lhs, rhs: rhs, label: label });
}

function mkEdge(
  lhs: string,
  rhs: string,
  label: ast.EdgeLabel,
): ast.EdgeDeclaration {
  return edge(mkEdgeName(lhs), mkEdgeName(rhs), label);
}

function mkEdges(
  chain: ast.EdgeName[],
  labels?: ast.EdgeLabel[],
): ast.EdgeDeclaration[] {
  return [...Array(chain.length - 1).keys()].map(i =>
    edge(chain[i], chain[i + 1], labels ? labels[i] : ast.Skip({})),
  );
}

function mkLabel(n: number): ast.EdgeLabel {
  return ast.Assignment({
    lhs: ast.Reference({ identifier: 'x' + n.toString() }),
    rhs: ast.Reference({ identifier: 'y' }),
  });
}

describe('inlineReachability', () => {
  test('findAcceptablePaths should find a simple path in a chain', () => {
    let nodes = ['x', 'y', 'z'].map(mkEdgeName);
    let edges = mkEdges(nodes);

    expect(pretty(t.findAcceptablePaths(edges, nodes[0], nodes.reverse()[0])))
      .toMatchInlineSnapshot(`
      "[
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
      ]"
    `);
  });

  test('findAcceptablePaths should reject multiple paths', () => {
    let nodes1 = ['x', 'y', 'z'].map(mkEdgeName);
    let nodes2 = ['x', 'y1', 'z'].map(mkEdgeName);
    let edges = mkEdges(nodes1).concat(mkEdges(nodes2));

    expect(
      t.findAcceptablePaths(edges, nodes1[0], nodes1.reverse()[0]),
    ).toMatchInlineSnapshot(`"nope"`);
  });

  test.skip('substituteWithPaths should reuse edge when substituting single edge', () => {
    let nodes = ['x', 'y', 'z'].map(mkEdgeName);
    let edges = mkEdges(nodes);

    t.substituteWithPaths(
      edges,
      edges[0],
      mkEdges(['a', 'b'].map(mkEdgeName), [mkLabel(1)]),
      mkEdgeName('a'),
      mkEdgeName('b')
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
    let nodes = ['x', 'y', 'z'].map(mkEdgeName);
    let edges = mkEdges(nodes);
    let replacement = mkEdges(
      ['a', 'b', 'c'].map(mkEdgeName),
      [1, 2].map(mkLabel),
    );

    t.substituteWithPaths(edges, edges[0], replacement, mkEdgeName('a'), mkEdgeName('c'));

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
    let nodes = ['x', 'y', 'z'].map(mkEdgeName);
    let edges = mkEdges(nodes);
    let replacement = mkEdges(
      ['a', 'b', 'c'].map(mkEdgeName),
      [1, 2].map(mkLabel),
    );

    t.substituteWithPaths(edges, edges[0], replacement, mkEdgeName('a'), mkEdgeName('c'));

    expect(pretty(edges)).toMatchInlineSnapshot(`
      "[
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
            parts: [ { identifier: '__gen_1', kind: 'Literal' } ]
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

  test('substituteWithPaths should replace existing edge with new one', () => {
    let nodes = ['x', 'y', 'z'].map(mkEdgeName);
    let edges = mkEdges(nodes);
    let replacement = [mkEdge('a', 'b', mkLabel(1))];

    t.substituteWithPaths(edges, edges[0], replacement, mkEdgeName('a'), mkEdgeName('b'));

    expect(pretty(edges)).toMatchInlineSnapshot(`
      "[
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
