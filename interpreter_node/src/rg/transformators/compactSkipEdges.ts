import * as utils from '../../utils';
import * as ast from '../ast';

const beginEdgeName = ast.EdgeName({
  parts: [ast.Literal({ identifier: 'begin' })],
});

// eslint-disable-next-line complexity -- It's fine.
export function compactSkipEdges({ edges }: ast.GameDeclaration) {
  // Rename all bindings so bind names are globally unique.
  let index = 0;
  for (const x of edges) {
    if (ast.lib.hasBindings(x.rhs)) {
      const mapping = utils.mapToObject(ast.lib.bindings(x.rhs), binding => [
        binding.identifier,
        `bind_${++index}`,
      ]);

      for (const y of edges) {
        if (x !== y) {
          if (ast.lib.isFollowing(x, y) || utils.isEqual(x.lhs, y.lhs)) {
            ast.lib.renameInEdgeLabel(y.label, mapping);
            ast.lib.renameInEdgeName(y.lhs, mapping);
          }

          if (ast.lib.isFollowing(y, x) || utils.isEqual(x.rhs, y.rhs)) {
            ast.lib.renameInEdgeLabel(y.label, mapping);
            ast.lib.renameInEdgeName(y.rhs, mapping);
          }
        }
      }

      ast.lib.renameInEdgeLabel(x.label, mapping);
      ast.lib.renameInEdgeName(x.lhs, mapping);
      ast.lib.renameInEdgeName(x.rhs, mapping);
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
  //   1. x != Assignment of `player` OR c has no bindings
  //   2. y == Skip
  //   3. b has no other outgoing edges
  //   4. b has no bindings
  //   5. there's no other edge between a and c (multiedges are not allowed)
  for (const y of edges.slice()) {
    if (ast.lib.isSkip(y.label) && !ast.lib.hasBindings(y.lhs)) {
      for (const x of edges.slice()) {
        if (
          (x.label.kind !== 'Assignment' ||
            x.label.lhs.kind !== 'Reference' ||
            x.label.lhs.identifier !== 'player' ||
            !ast.lib.hasBindings(y.rhs)) &&
          ast.lib.isFollowing(x, y) &&
          ast.lib.outgoing(edges, y.lhs).every(z => z === y) &&
          !ast.lib.hasConnection(edges, x.lhs, y.rhs)
        ) {
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
  //   1. x == Skip
  //   2. b has no other incoming edges
  //   3. b has no bindings
  //   4. there's no other edge between a and c (multiedges are not allowed)
  for (const x of edges.slice()) {
    if (ast.lib.isSkip(x.label) && !ast.lib.hasBindings(x.rhs)) {
      for (const y of edges.slice()) {
        if (
          ast.lib.isFollowing(x, y) &&
          ast.lib.incoming(edges, x.rhs).every(z => z === x) &&
          !ast.lib.hasConnection(edges, x.lhs, y.rhs)
        ) {
          utils.remove(edges, x);
          y.lhs = x.lhs;
        }
      }
    }
  }

  // Before:
  //       x
  //   a ----> b
  //
  // After:
  //
  //   b
  //
  // Conditions:
  //   1. x == Skip
  //   2. a has no other incoming edges
  //   3. a has no other outgoing edges
  //   4. a has no bindings
  //   5. a is not `begin`
  for (const x of edges.slice()) {
    if (
      ast.lib.isSkip(x.label) &&
      !ast.lib.hasBindings(x.lhs) &&
      ast.lib.incoming(edges, x.lhs).length === 0 &&
      ast.lib.outgoing(edges, x.lhs).every(y => y === x) &&
      !utils.isEqual(x.lhs, beginEdgeName)
    ) {
      // console.log(x.lhs, x.rhs);
      utils.remove(edges, x);
    }
  }
}
