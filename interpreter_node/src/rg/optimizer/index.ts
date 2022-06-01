import * as utils from '../../utils';
import * as ast from '../ast';

function bindings({ parts }: ast.EdgeName) {
  return parts.filter(function isBind(part): part is ast.Binding {
    return part.kind === 'Binding';
  });
}

function hasBindings(edgeName: ast.EdgeName) {
  return bindings(edgeName).length !== 0;
}

function incoming(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  return edges.filter(({ rhs }) => isEqual(edgeName, rhs));
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

function rebind(
  edgeLabel: ast.EdgeLabel,
  edgeName: ast.EdgeName,
  mapping: Record<string, string>,
) {
  for (const binding of bindings(edgeName)) {
    if (binding.identifier in mapping) {
      binding.identifier = mapping[binding.identifier];
    }
  }

  switch (edgeLabel.kind) {
    case 'Assignment':
    case 'Comparison':
      rebindExpression(edgeLabel.lhs, mapping);
      rebindExpression(edgeLabel.rhs, mapping);
      return;
    case 'Reachability':
    case 'Skip':
      return;
  }
}

function rebindExpression(
  expression: ast.Expression,
  mapping: Record<string, string>,
) {
  switch (expression.kind) {
    case 'Access':
      rebindExpression(expression.lhs, mapping);
      rebindExpression(expression.rhs, mapping);
      return;
    case 'Cast':
      rebindExpression(expression.rhs, mapping);
      return;
    case 'Reference':
      if (expression.identifier in mapping) {
        expression.identifier = mapping[expression.identifier];
      }
      return;
  }
}

function outgoing(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  return edges.filter(({ lhs }) => isEqual(edgeName, lhs));
}

// eslint-disable-next-line complexity -- It's fine.
export function compactSkipEdges({ edges }: ast.GameDeclaration) {
  // Rename all bindings so bind names are globally unique.
  let index = 0;
  for (const x of edges) {
    if (hasBindings(x.rhs)) {
      const mapping = Object.fromEntries(
        bindings(x.rhs).map(binding => [binding.identifier, `bind_${++index}`]),
      );

      for (const y of edges) {
        if (isFollowing(x, y)) {
          rebind(y.label, y.lhs, mapping);
          continue;
        }

        if (isFollowing(y, x)) {
          rebind(y.label, y.rhs, mapping);
          continue;
        }
      }

      rebind(x.label, x.rhs, mapping);
    }
  }

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
  for (const y of edges.slice()) {
    if (isSkip(y.label) && !hasBindings(y.lhs)) {
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
  //   4. b has no bindings
  for (const x of edges.slice()) {
    if (isSkip(x.label) && !hasBindings(x.rhs)) {
      for (const y of edges.slice()) {
        if (isFollowing(x, y) && incoming(edges, x.rhs).every(z => z === x)) {
          utils.remove(edges, x);
          y.lhs = x.lhs;
        }
      }
    }
  }
}
