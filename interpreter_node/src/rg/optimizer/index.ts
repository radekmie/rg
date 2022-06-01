import * as utils from '../../utils';
import * as ast from '../ast';

function hasBindings({ parts }: ast.EdgeName) {
  return parts.some(({ kind }) => kind === 'Binding');
}

function incoming(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  return edges.filter(x => isEqual(edgeName, x.rhs));
}

function isEqual(a: ast.EdgeName, b: ast.EdgeName) {
  return JSON.stringify(a) === JSON.stringify(b);
}

function isFollowing(x: ast.EdgeDeclaration, y: ast.EdgeDeclaration) {
  return isEqual(x.rhs, y.lhs);
}

function isSkip({ kind }: ast.EdgeLabel) {
  return kind === 'Skip';
}

function outgoing(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  return edges.filter(x => isEqual(edgeName, x.lhs));
}

// eslint-disable-next-line complexity -- It's fine.
export function compactSkipEdges({ edges }: ast.GameDeclaration) {
  // Before:
  //       x       y
  //   a ----> b ----> c
  //
  // After:
  //       x
  //   a ----> c
  //
  // Conditions:
  //   1. y = Skip
  //   2. b has no other outgoing edges
  //   3. b has no bindings
  //   4. c has no bindings
  for (const y of edges.slice()) {
    if (isSkip(y.label) && !hasBindings(y.lhs) && !hasBindings(y.rhs)) {
      for (const x of edges.slice()) {
        if (isFollowing(x, y) && outgoing(edges, y.lhs).every(z => z === y)) {
          utils.remove(edges, y);
          x.rhs = y.rhs;
        }
      }
    }
  }

  // Before:
  //       x       y
  //   a ----> b ----> c
  //
  // After:
  //       y
  //   a ----> c
  //
  // Conditions:
  //   1. x = Skip
  //   2. b has no other incoming edges
  //   3. a has no bindings
  //   4. b has no bindings
  for (const x of edges.slice()) {
    if (isSkip(x.label) && !hasBindings(x.lhs) && !hasBindings(x.rhs)) {
      for (const y of edges.slice()) {
        if (isFollowing(x, y) && incoming(edges, x.rhs).every(z => z === x)) {
          utils.remove(edges, x);
          y.lhs = x.lhs;
        }
      }
    }
  }
}
