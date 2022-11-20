import * as utils from '../../utils';
import * as ast from '../ast';

export function joinForkSuffixes({ edges }: ast.GameDeclaration) {
  // Before:
  //
  //  Visual:
  //
  //    z1──(e3)─►y1──(e1)─┐
  //                       ▼
  //                       x
  //                       ▲
  //    z2──(e4)─►y2──(e2)─┘
  //
  //   Edges:
  //   - e1 from y1 to x
  //   - e2 from y2 to x
  //   - e3 from z1 to y1
  //   - e4 from z2 to y2
  //
  // After:
  //
  //  Visual:
  //
  //    z1──(e3)─┐
  //             ▼
  //             y1──(e1)─►x
  //             ▲
  //    z2──(e4)─┘
  //
  //   Edges:
  //   - e1 from y1 to x
  //   - e3 from z1 to y1
  //   - e4 from z2 to y1
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
    // (1)
    if (!ast.lib.outgoing(edges, e1.lhs).some(e => e !== e1)) {
      for (const e2 of edges) {
        if (
          e1 !== e2 &&
          utils.isEqual(e1.rhs, e2.rhs) &&
          utils.isEqual(e1.label, e2.label) && // (8)
          utils.isEqual(ast.lib.bindings(e1.lhs), ast.lib.bindings(e2.lhs)) && // (5)
          !ast.lib.outgoing(edges, e2.lhs).some(e => e !== e2) && // (2)
          !edges.some(
            e =>
              e.label.kind === 'Reachability' &&
              (utils.isEqual(e.label.lhs, e2.lhs) ||
                utils.isEqual(e.label.rhs, e2.lhs)),
          ) // (4)
        ) {
          for (const e4 of edges) {
            if (
              ast.lib.isFollowing(e4, e2) &&
              !ast.lib.incoming(edges, e2.lhs).some(e => e !== e4) // (3)
            ) {
              for (const e3 of edges) {
                if (
                  !utils.isEqual(e3.lhs, e4.lhs) && // (8)
                  ast.lib.isFollowing(e3, e1) &&
                  utils.isEqual(
                    ast.lib.bindings(e3.lhs),
                    ast.lib.bindings(e4.lhs),
                  ) // (6)
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
}
