import * as utils from '../../utils';
import * as ast from '../ast';

export function findThePath(
  edges: ast.EdgeDeclaration[],
  start: ast.EdgeName,
  target: ast.EdgeName,
): ast.EdgeLabel[] | string {
  let x: ast.EdgeName = start;
  const visited: ast.EdgeName[] = [];
  const path: ast.EdgeLabel[] = [];
  // TODO when is '===' instead of 'isEqual' enough?
  while (!utils.isEqual(x, target)) {
    // no loops because they increase the number of possible paths
    if (visited.some(y => utils.isEqual(y, x))) {
      return 'nope';
    }

    const reachable = edges.filter(e => utils.isEqual(e.lhs, x));

    if (reachable.length != 1) {
      return 'nope';
    }

    const edge = reachable[0];
    const l = edge.label;

    if (l.kind === 'Assignment') {
      return 'nope';
    }

    path.push(l);
    visited.push(x);
    x = edge.rhs;
  }

  return path;
}

export function freshVar() {
  // TODO
  return ast.EdgeName({ parts: [ast.Literal({ identifier: '__gen_1' })] });
}

export function substitutePath(
  edges: ast.EdgeDeclaration[],
  originalEdge: ast.EdgeDeclaration,
  path: ast.EdgeLabel[],
) {
  if (path.length == 0) {
    originalEdge.label = ast.Skip({});
    return;
  }

  const target = originalEdge.rhs;

  originalEdge.label = path[0];
  originalEdge.rhs = freshVar();

  let last = originalEdge;
  for (const l of path.slice(1)) {
    const next = ast.EdgeDeclaration({
      lhs: last.rhs,
      rhs: freshVar(),
      label: l,
    });
    edges.push(next);
    last = next;
  }

  last.rhs = target;
}

export function inlineReachability({ edges }: ast.GameDeclaration) {
  for (const e of edges) {
    if (e.label.kind === 'Reachability' && !e.label.negated) {
      console.log('hello reachability');
      const path = findThePath(edges, e.label.lhs, e.label.rhs);
      if (typeof path === 'object') {
        substitutePath(edges, e, path);
      } else {
        console.log("can't find path, msg:" + path);
      }
    }
  }
  console.log('hello');
}
