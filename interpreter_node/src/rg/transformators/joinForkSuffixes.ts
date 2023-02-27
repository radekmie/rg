import * as utils from '../../utils';
import * as lib from '../ast/lib';
import * as ast from '../ast/types';

// eslint-disable-next-line complexity -- It could be split.
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
  //   4. y2 is not a reachability target -- it's deleted
  //   5. y1 and y2 have the same bindings
  //   6. z1 and z2 have the same bindings
  //   7. z1 and z2 are not equal
  //   8. e1 and e2 have the same label
  //   9. y1 is not a reachability target -- it gains reachability paths
  for (const e1 of edges) {
    const { lhs: y1, rhs: x } = e1;
    if (
      lib.outgoing(edges, y1).length === 1 && // (1)
      !lib.isReachabilityTarget(y1, edges) // (9)
    ) {
      for (const e2 of edges) {
        const y2 = e2.lhs;
        if (
          e1 !== e2 &&
          utils.isEqual(x, e2.rhs) &&
          utils.isEqual(e1.label, e2.label) && // (8)
          utils.isEqual(lib.bindings(y1), lib.bindings(y2)) && // (5)
          lib.outgoing(edges, y2).length === 1 && // (2)
          !lib.isReachabilityTarget(y2, edges) // (4)
        ) {
          for (const e4 of edges) {
            const z2 = e4.lhs;
            if (
              lib.isFollowing(e4, e2) &&
              lib.incoming(edges, y2).length === 1 // (3)
            ) {
              for (const e3 of edges) {
                const z1 = e3.lhs;
                if (
                  !utils.isEqual(z1, z2) && // (8)
                  lib.isFollowing(e3, e1) &&
                  utils.isEqual(lib.bindings(e3.lhs), lib.bindings(e4.lhs)) // (6)
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
