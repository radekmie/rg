import * as utils from '../../utils';
import * as ast from '../ast';

export function skipSelfAssignments({ edges }: ast.GameDeclaration) {
  // TODO: Flip the image horizontally (lhs on the left).
  // Before:
  //     <--(e1)-- y1 <--(e3)-- z1
  //   x
  //     <--(e2)-- y2 <--(e4)-- z2
  //
  // After:
  //                  <--(e3)-- z1
  //   x <--(e1)-- y1
  //                  <--(e4)-- z2
  //
  // Conditions:
  //   1. y1 has no other outgoing edges
  //   2. y2 has no other outgoing edges
  //   3. y2 has no other incoming edges
  //   4. y2 is not a reachability target
  //   5. y1 and y2 have the same bindings
  //   6. z1 and z2 have the same bindings
  //   7. z1 and z2 are not equal
  //   8. e1 and e2 have the same label
  for (const e1 of edges) {
    if (!ast.lib.outgoing(edges, e1.lhs).some(e => e !== e1)) {
      for (const e2 of edges) {
        if (
          e1 !== e2 &&
          utils.isEqual(e1.rhs, e2.rhs) &&
          utils.isEqual(e1.label, e2.label) &&
          utils.isEqual(ast.lib.bindings(e1.lhs), ast.lib.bindings(e2.lhs)) &&
          !edges.some(
            e =>
              e.label.kind === 'Reachability' &&
              (utils.isEqual(e.label.lhs, e2.lhs) ||
                utils.isEqual(e.label.rhs, e2.lhs)),
          )
        ) {
          for (const e4 of edges) {
            if (
              ast.lib.isFollowing(e4, e2) &&
              !ast.lib.incoming(edges, e2.lhs).some(e => e !== e4)
            ) {
              for (const e3 of edges) {
                if (
                  !utils.isEqual(e3.lhs, e4.lhs) &&
                  ast.lib.isFollowing(e3, e1) &&
                  utils.isEqual(
                    ast.lib.bindings(e3.lhs),
                    ast.lib.bindings(e4.lhs),
                  )
                ) {
                  e4.rhs = e3.rhs;
                  utils.remove(edges, e2);
                }
              }
            }
          }
        }
      }
    }
  }

  for (const edge of edges) {
    if (isSelfAssignment(edge.label)) {
      edge.label = ast.Skip({});
    }
  }
}

function isEqualReference(lhs: ast.Expression, rhs: ast.Expression): boolean {
  if (lhs.kind === 'Cast') {
    return isEqualReference(lhs.rhs, rhs);
  }

  if (rhs.kind === 'Cast') {
    return isEqualReference(lhs, rhs.rhs);
  }

  switch (lhs.kind) {
    case 'Access':
      return (
        rhs.kind === 'Access' &&
        isEqualReference(lhs.lhs, rhs.lhs) &&
        isEqualReference(lhs.rhs, rhs.rhs)
      );
    case 'Reference':
      return rhs.kind === 'Reference' && lhs.identifier === rhs.identifier;
  }
}

function isSelfAssignment(edgeLabel: ast.EdgeLabel) {
  return (
    edgeLabel.kind === 'Assignment' &&
    isEqualReference(edgeLabel.lhs, edgeLabel.rhs)
  );
}
