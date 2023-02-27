import * as utils from '../../utils';
import * as lib from '../ast/lib';
import * as ast from '../ast/types';

const beginEdgeName = ast.EdgeName({
  parts: [ast.Literal({ identifier: 'begin' })],
});

// eslint-disable-next-line complexity -- It's fine.
export function compactSkipEdges({ edges }: ast.GameDeclaration) {
  // Rename all bindings if their names are not globally unique.
  if (!lib.areBindingsUnique(edges)) {
    lib.makeBindingsUnique(edges);
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
    if (lib.isSkip(y.label) && !lib.hasBindings(y.lhs)) {
      for (const x of edges.slice()) {
        if (
          (x.label.kind !== 'Assignment' ||
            x.label.lhs.kind !== 'Reference' ||
            x.label.lhs.identifier !== 'player' ||
            !lib.hasBindings(y.rhs)) &&
          lib.isFollowing(x, y) &&
          lib.outgoing(edges, y.lhs).every(z => z === y) &&
          !lib.hasConnection(edges, x.lhs, y.rhs)
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
    if (lib.isSkip(x.label) && !lib.hasBindings(x.rhs)) {
      for (const y of edges.slice()) {
        if (
          lib.isFollowing(x, y) &&
          lib.incoming(edges, x.rhs).every(z => z === x) &&
          !lib.hasConnection(edges, x.lhs, y.rhs)
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
      lib.isSkip(x.label) &&
      !lib.hasBindings(x.lhs) &&
      lib.incoming(edges, x.lhs).length === 0 &&
      lib.outgoing(edges, x.lhs).every(y => y === x) &&
      !utils.isEqual(x.lhs, beginEdgeName)
    ) {
      // console.log(x.lhs, x.rhs);
      utils.remove(edges, x);
    }
  }
}
