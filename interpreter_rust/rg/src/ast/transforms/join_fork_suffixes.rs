use crate::ast::{Edge, Error, Game, Node};
use std::collections::BTreeSet;
use std::sync::Arc;

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
//   4. y2 is not a reachability target -- it's deleted
//   5. e1 and e2 have the same label
//   6. y1 is not a reachability target -- it gains reachability paths

impl<Id: Clone + Ord> Game<Id> {
    pub fn join_fork_suffixes(&mut self) -> Result<(), Error<Id>> {
        while self.join_fork_suffixes_step() {}
        Ok(())
    }

    fn join_fork_suffixes_step(&mut self) -> bool {
        let mut changed = false;
        for (e4s, y1, e2) in self.to_join() {
            changed = true;
            self.remove_edge(&e2);
            for mut e4 in e4s {
                self.remove_edge(&e4);
                Arc::make_mut(&mut e4).rhs = y1.clone();
                self.add_edge(e4);
            }
        }
        changed
    }

    #[expect(clippy::type_complexity)]
    fn to_join(&self) -> Vec<(Vec<Arc<Edge<Id>>>, Node<Id>, Arc<Edge<Id>>)> {
        let prev_edges = self.prev_edges();

        let mut to_join = vec![];
        let mut as_target = BTreeSet::new();
        let mut to_change = BTreeSet::new();
        let mut to_remove = BTreeSet::new();
        for x in self.nodes() {
            for e1 in prev_edges.get(x).into_iter().flat_map(|x| x.iter()) {
                if to_remove.contains(&&e1.lhs)
                    // (1)
                    || self.outgoing_edge(&e1.lhs).is_none()
                    // (6)
                    || self.is_reachability_target(&e1.lhs)
                {
                    continue;
                }

                for e2 in prev_edges.get(x).into_iter().flat_map(|x| x.iter()) {
                    if e1 == e2
                        || as_target.contains(&e2.lhs)
                        // (5)
                        || e1.label != e2.label
                        // (2)
                        || self.outgoing_edge(&e2.lhs).is_none()
                        // (4)
                        || self.is_reachability_target(&e2.lhs)
                    {
                        continue;
                    }

                    // (3)
                    let e4s: Vec<_> = prev_edges
                        .get(&e2.lhs)
                        .into_iter()
                        .flat_map(|x| x.iter().map(|e| *e))
                        .collect();

                    if e4s.iter().any(|e4| to_change.contains(&e4.lhs)) {
                        continue;
                    }

                    as_target.insert(&e1.lhs);

                    e4s.iter().for_each(|e4| {
                        to_change.insert(&e4.lhs);
                    });
                    to_remove.insert(&e2.lhs);
                    to_join.push((
                        e4s.into_iter().cloned().collect(),
                        e1.lhs.clone(),
                        (*e2).clone(),
                    ));
                }
            }
        }
        to_join
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        join_fork_suffixes,
        small,
        "begin, end: ;
        1, l1: 1 == 1;
        1, r1: 2 == 2;
        l1, l2: 4 == 4;
        l2, 2: 0 == 0;
        r1, r2: 5 == 5;
        r2, 2: 0 == 0;
        2, 3: 7 == 7;",
        "begin, end: ;
        1, l1: 1 == 1;
        1, r1: 2 == 2;
        l1, l2: 4 == 4;
        l2, 2: 0 == 0;
        2, 3: 7 == 7;
        r1, l2: 5 == 5;"
    );

    test_transform!(
        join_fork_suffixes,
        bigger,
        "begin, a0: branch0 == branch0;
        a5, end: 5 == 5;
        a0, a1: 0 == 0;
        a1, a2: 1 == 1;
        a2, a3: 2 == 2;
        a3, a4: 3 == 3;
        a4, a5: 4 == 4;
        begin, b0: branch1 == branch1;
        b5, end: 5 == 5;
        b0, b1: 0 == 0;
        b1, b2: 1 == 1;
        b2, b3: 2 == 2;
        b3, b4: 3 == 3;
        b4, b5: 4 == 4;
        begin, c0: branch2 == branch2;
        c5, end: 5 == 5;
        c0, c1: 0 == 0;
        c1, c2: 1 == 1;
        c2, c3: 2 == 2;
        c3, c4: 3 == 3;
        c4, c5: 4 == 4;
        begin, d0: branch3 == branch3;
        d5, end: 5 == 5;
        d0, d1: 0 == 0;
        d1, d2: 1 == 1;
        d2, d3: 2 == 2;
        d3, d4: 3 == 3;
        d4, d5: 4 == 4;",
        "begin, a0: branch0 == branch0;
        a5, end: 5 == 5;
        a0, a1: 0 == 0;
        a1, a2: 1 == 1;
        a2, a3: 2 == 2;
        a3, a4: 3 == 3;
        a4, a5: 4 == 4;
        begin, a0: branch1 == branch1;
        begin, a0: branch2 == branch2;
        begin, a0: branch3 == branch3;"
    );

    test_transform!(
        join_fork_suffixes,
        dont_join_multiple_outgoing,
        "begin, end: ;
        1, l1: 1 == 1;
        1, r1: 2 == 2;
        l1, l2: 4 == 4;
        l2, 2: 0 == 0;
        r1, r2: 5 == 5;
        r2, 2: 0 == 0;
        2, 3: 7 == 7;
        l2, 4: 0 == 0;
        r2, 4: 0 == 0;",
        "begin, end: ;
        1, l1: 1 == 1;
        1, r1: 2 == 2;
        l1, l2: 4 == 4;
        l2, 2: 0 == 0;
        r1, r2: 5 == 5;
        r2, 2: 0 == 0;
        2, 3: 7 == 7;
        l2, 4: 0 == 0;
        r2, 4: 0 == 0;"
    );

    test_transform!(
        join_fork_suffixes,
        dont_join_multiple_outgoing_single,
        "begin, end: ;
        1, l1: 1 == 1;
        1, r1: 2 == 2;
        l1, l2: 4 == 4;
        l2, 2: 0 == 0;
        r1, r2: 5 == 5;
        r2, 2: 0 == 0;
        2, 3: 7 == 7;
        l2, 4: 0 == 0;",
        "begin, end: ;
        1, l1: 1 == 1;
        1, r1: 2 == 2;
        l1, l2: 4 == 4;
        l2, 2: 0 == 0;
        r1, r2: 5 == 5;
        r2, 2: 0 == 0;
        2, 3: 7 == 7;
        l2, 4: 0 == 0;"
    );

    test_transform!(
        join_fork_suffixes,
        dont_join_multiple_incoming,
        "begin, end: ;
        1, l1: 1 == 1;
        1, r1: 2 == 2;
        l1, l2: 4 == 4;
        l2, 2: 0 == 0;
        r1, r2: 5 == 5;
        r2, 2: 0 == 0;
        2, 3: 7 == 7;
        4, l2: 0 == 0;
        4, r2: 1 == 1;",
        "begin, end: ;
        1, l1: 1 == 1;
        1, r1: 2 == 2;
        l1, l2: 4 == 4;
        l2, 2: 0 == 0;
        2, 3: 7 == 7;
        4, l2: 0 == 0;
        4, l2: 1 == 1;
        r1, l2: 5 == 5;"
    );

    test_transform!(
        join_fork_suffixes,
        dont_join_multiple_incoming_single,
        "begin, end: ;
        1, l1: 1 == 1;
        1, r1: 2 == 2;
        l1, l2: 4 == 4;
        l2, 2: 0 == 0;
        r1, r2: 5 == 5;
        r2, 2: 0 == 0;
        2, 3: 7 == 7;
        4, l2: 0 == 0;",
        "begin, end: ;
        1, l1: 1 == 1;
        1, r1: 2 == 2;
        l1, l2: 4 == 4;
        l2, 2: 0 == 0;
        2, 3: 7 == 7;
        4, l2: 0 == 0;
        r1, l2: 5 == 5;"
    );

    test_transform!(
        join_fork_suffixes,
        intermediate_multi_edges,
        "begin, end: ;
        1, l1: 0 == 0;
        1, r1: 0 == 0;
        l1, 2: 1 == 1;
        r1, 2: 1 == 1;
        2, 3: 7 == 7;",
        "begin, end: ;
        1, l1: 0 == 0;
        l1, 2: 1 == 1;
        2, 3: 7 == 7;"
    );

    test_transform!(
        join_fork_suffixes,
        shape_from_breakthrough,
        "begin, end: ;
        11, 9: 3 == 3;
        9, 12: 5 == 5;
        9, 18: 1 == 1;
        9, 20: 2 == 2;
        18, 15: 3 == 3;
        20, 15: 3 == 3;
        15, 12: 4 == 4;
        15, 23: ;
        23, 12: 5 == 5;",
        "begin, end: ;
        11, 9: 3 == 3;
        9, 12: 5 == 5;
        9, 18: 1 == 1;
        18, 15: 3 == 3;
        15, 12: 4 == 4;
        15, 23: ;
        23, 12: 5 == 5;
        9, 18: 2 == 2;"
    );

    test_transform!(
        join_fork_suffixes,
        last_reachability,
        "begin, end: ;
        x, y: ? 1 -> 3;
        1, 1a: 1==1;
        1a, 2a: 3==3;
        2a, 3: 4==4;
        1, 1b: 2==2;
        1b, 2b: 3==3;
        2b, 3: 4==4;",
        "begin, end: ;
        x, y: ? 1 -> 3;
        1, 1a: 1 == 1;
        1a, 2a: 3 == 3;
        2a, 3: 4 == 4;
        1, 1a: 2 == 2;"
    );

    test_transform!(
        join_fork_suffixes,
        dont_join_inner_reachability,
        "begin, end: ;
        x, y: ? 1 -> 2a;
        1, 1a: 1==1;
        1a, 2a: 3==3;
        2a, 3: 4==4;
        1, 1b: 2==2;
        1b, 2b: 3==3;
        2b, 3: 4==4;",
        "begin, end: ;
        x, y: ? 1 -> 2a;
        1, 1a: 1 == 1;
        1a, 2a: 3 == 3;
        2a, 3: 4 == 4;
        1, 1b: 2 == 2;
        1b, 2b: 3 == 3;
        2b, 3: 4 == 4;"
    );
}
