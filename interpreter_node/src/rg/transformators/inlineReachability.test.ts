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

function mkEdges(chain: ast.EdgeName[]): ast.EdgeDeclaration[] {
  return [...Array(chain.length - 1).keys()].map(i =>
    edge(chain[i], chain[i + 1], ast.Skip({})),
  );
}

function mkLabel(n: number): ast.EdgeLabel {
  return ast.Assignment({
    lhs: ast.Reference({ identifier: 'x' + n.toString() }),
    rhs: ast.Reference({ identifier: 'y' }),
  });
}

describe('inlineReachability', () => {
  test('findThePath should find a simple path in a chain', () => {
    let nodes = ['x', 'y', 'z'].map(mkEdgeName);
    let edges = mkEdges(nodes);

    expect(
      pretty(t.findThePath(edges, nodes[0], nodes.reverse()[0])),
    ).toMatchInlineSnapshot(`"[ { kind: 'Skip' }, { kind: 'Skip' } ]"`);
  });

  test('findThePath should reject multiple paths', () => {
    let nodes1 = ['x', 'y', 'z'].map(mkEdgeName);
    let nodes2 = ['x', 'y1', 'z'].map(mkEdgeName);
    let edges = mkEdges(nodes1).concat(mkEdges(nodes2));

    expect(
      t.findThePath(edges, nodes1[0], nodes1.reverse()[0]),
    ).toMatchInlineSnapshot(`"nope"`);
  });

  test('substitutePath should reuse edge when substituting single edge', () => {
    let nodes = ['x', 'y', 'z'].map(mkEdgeName);
    let edges = mkEdges(nodes);

    t.substitutePath(edges, edges[0], [mkLabel(1)]);

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

  test('substitutePath should create one edge when substituting two', () => {
    let nodes = ['x', 'y', 'z'].map(mkEdgeName);
    let edges = mkEdges(nodes);

    t.substitutePath(edges, edges[0], [1, 2].map(mkLabel));

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
});
