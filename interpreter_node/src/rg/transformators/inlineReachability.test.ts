import * as utils from '../../utils';
import { Result } from '../../utils';
import * as ast from '../ast';
import * as s from '../ast/serializer';
import * as t from './inlineReachability';

function serializeEdges(edges: ast.EdgeDeclaration[]): string {
  return edges.map(s.serializeEdge).join('\n');
}

function serializeResult(
  maybeEdges: Result<ast.EdgeDeclaration[], string>,
): string {
  if (maybeEdges.ok) {
    return 'success:\n' + serializeEdges(maybeEdges.value);
  } else {
    return 'failure:\n' + maybeEdges.error;
  }
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

    expect(
      serializeResult(
        t.findAcceptablePaths(edges, nodes[0], nodes.reverse()[0]),
      ),
    ).toMatchInlineSnapshot(`
      "success:
      x, y: ;
      y, z: ;"
    `);
  });

  test('findAcceptablePaths should reject multiple paths', () => {
    const nodes1 = ['x', 'y', 'z'].map(makeEdgeName);
    const nodes2 = ['x', 'y1', 'z'].map(makeEdgeName);
    const edges = makeEdges(nodes1).concat(makeEdges(nodes2));

    expect(
      serializeResult(
        t.findAcceptablePaths(edges, nodes1[0], nodes1.reverse()[0]),
      ),
    ).toMatchInlineSnapshot(`
      "failure:
      can't ensure single path at runtime"
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

    expect(serializeEdges(edges)).toMatchInlineSnapshot(`
      "__gen_1_ignoreme, y: ;
      y, z: ;
      x, __gen_2_b: x1 = y;
      __gen_2_b, y: x2 = y;"
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

    expect(serializeEdges(edges)).toMatchInlineSnapshot(`
      "__gen_3_ignoreme, y: ;
      y, z: ;
      x, y: x1 = y;"
    `);
  });
});
