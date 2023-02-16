import { describe, expect, test } from 'vitest';

import { findAcceptablePaths, substituteWithPaths } from './inlineReachability';
import * as utils from '../../utils';
import * as ast from '../ast';

function serializeEdges(edges: ast.EdgeDeclaration[]) {
  return edges.map(ast.serializeEdge).join('\n');
}

function makeEdgeName(n: string) {
  return ast.EdgeName({ parts: [ast.Literal({ identifier: n })] });
}

function edge(lhs: ast.EdgeName, rhs: ast.EdgeName, label: ast.EdgeLabel) {
  return ast.EdgeDeclaration({ lhs, rhs, label });
}

function makeEdge(lhs: string, rhs: string, label: ast.EdgeLabel) {
  return edge(makeEdgeName(lhs), makeEdgeName(rhs), label);
}

function makeEdges(chain: ast.EdgeName[], labels?: ast.EdgeLabel[]) {
  return [...Array(chain.length - 1).keys()].map(i =>
    edge(chain[i], chain[i + 1], labels ? labels[i] : ast.Skip({})),
  );
}

function makeLabel(n: number) {
  return ast.Assignment({
    lhs: ast.Reference({ identifier: 'x' + n.toString() }),
    rhs: ast.Reference({ identifier: 'y' }),
  });
}

const makeFreshNode = ast.lib.makeFreshEdgeName([]);

describe('inlineReachability', () => {
  test('findAcceptablePaths should find a simple path in a chain', () => {
    const nodes = ['x', 'y', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes);
    const result = findAcceptablePaths(edges, nodes[0], nodes.reverse()[0]);

    expect(result.ok).toBe(true);
    utils.assert(result.ok, ''); // Make TypeScript happy.
    expect(serializeEdges(result.value)).toMatchInlineSnapshot(`
      "x, y: ;
      y, z: ;"
    `);
  });

  test('findAcceptablePaths should reject multiple paths', () => {
    const nodes1 = ['x', 'y', 'z'].map(makeEdgeName);
    const nodes2 = ['x', 'y1', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes1).concat(makeEdges(nodes2));
    const result = findAcceptablePaths(edges, nodes1[0], nodes1.reverse()[0]);

    expect(result.ok).toBe(false);
    utils.assert(!result.ok, ''); // Make TypeScript happy.
    expect(result.error).toMatchInlineSnapshot(
      '"can\'t ensure single path at runtime"',
    );
  });

  test('substituteWithPaths should replace existing edge with new ones', () => {
    const nodes = ['x', 'y', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes);
    const replacement = makeEdges(
      ['a', 'b', 'c'].map(makeEdgeName),
      [1, 2].map(makeLabel),
    );

    substituteWithPaths(
      edges,
      makeFreshNode,
      edges[0],
      replacement,
      makeEdgeName('a'),
      makeEdgeName('c'),
    );

    expect(serializeEdges(edges)).toMatchInlineSnapshot(`
      "x, __gen_1_reachability_a_c: ;
      y, z: ;
      __gen_1_reachability_a_c, __gen_2_b: x1 = y;
      __gen_2_b, y: x2 = y;"
    `);
  });

  test('substituteWithPaths should replace existing edge with new one', () => {
    const nodes = ['x', 'y', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes);
    const replacement = [makeEdge('a', 'b', makeLabel(1))];

    substituteWithPaths(
      edges,
      makeFreshNode,
      edges[0],
      replacement,
      makeEdgeName('a'),
      makeEdgeName('b'),
    );

    expect(serializeEdges(edges)).toMatchInlineSnapshot(`
      "x, __gen_3_reachability_a_b: ;
      y, z: ;
      __gen_3_reachability_a_b, y: x1 = y;"
    `);
  });
});
