use crate::ast::{Edge, Error, Game, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type ToInline = (Node<Id>, Vec<Arc<Edge<Id>>>, Vec<Node<Id>>);

// Before:
// b, a1: x = 1;
// b, a2: x = 1;
// a1, a3: ;
// a2, a4: ;
//
// After:
// b, a1: x = 1;
// a1, a3: ;
// a1, a4: ;

impl Game<Id> {
    pub fn join_fork_prefixes(&mut self) -> Result<(), Error<Id>> {
        while self.join_fork_prefixes_step() {}
        Ok(())
    }

    fn join_fork_prefixes_step(&mut self) -> bool {
        let mut changed = false;
        for (node, to_remove, to_add) in self.find_to_join_prefixes() {
            changed = true;
            self.edges.retain(|edge| !to_remove.contains(edge));
            for next_node in to_add {
                let edge = Edge::new_skip(node.clone(), next_node);
                self.edges.push(Arc::from(edge));
            }
        }
        changed
    }

    fn find_to_join_prefixes(&self) -> Vec<ToInline> {
        let next_edges = self.next_edges();
        let prev_edges = self.prev_edges();
        let default_set = BTreeSet::new();
        let mut edges_to_join = vec![];
        // Start of the fork prefix
        for node in self.nodes() {
            // Get all outgoing edges that are not end edges
            let next_edges = next_edges
                .get(&node)
                .unwrap_or(&default_set)
                .iter()
                .filter(|e| next_edges.contains_key(&e.rhs));
            // Group outgoing edges by label
            let mut group_by_label = BTreeMap::new();
            for edge in next_edges {
                group_by_label
                    .entry(edge.label.clone())
                    .or_insert_with(Vec::new)
                    .push(edge);
            }
            for (_, edges) in group_by_label {
                if edges.len() < 2 {
                    continue;
                }
                // Remove all but the first edge
                // First edge should be the one with no other incoming edges
                let Some(head) = edges
                    .iter()
                    .find(|e| prev_edges.get(&e.rhs).is_some_and(|p| p.len() == 1))
                    .or_else(|| {
                        let mut bt = BTreeSet::new();
                        // Every edge should have the same predecessors
                        bt.extend(edges.iter().map(|e| {
                            prev_edges.get(&e.rhs).map(|prev| {
                                prev.iter()
                                    .map(|e| (e.lhs.clone(), e.label.clone()))
                                    .collect::<Vec<_>>()
                            })
                        }));
                        if bt.len() > 1 {
                            None
                        } else {
                            edges.first()
                        }
                    })
                else {
                    continue;
                };
                let node = head.rhs.clone();
                let tail = edges.iter().filter(|e| *e != head).collect::<Vec<_>>();
                let to_add = tail.iter().map(|e| e.rhs.clone()).collect::<Vec<_>>();
                let to_remove = tail.iter().map(|e| (***e).clone()).collect::<Vec<_>>();
                edges_to_join.push((node, to_remove, to_add));
            }
        }

        edges_to_join
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        join_fork_prefixes,
        small1,
        "begin, a: ;
        a, b: 1 == 1;
        a, c: 1 == 1;
        b, end: 2 == 2;
        c, d: ;",
        "begin, a: ;
        a, b: 1 == 1;
        b, end: 2 == 2;
        c, d: ;
        b, c: ;"
    );

    test_transform!(
        join_fork_prefixes,
        small2,
        "begin, a: ;
        begin, b: ;"
    );

    test_transform!(
        join_fork_prefixes,
        small3,
        "begin, a: ;
        a, f: ;
        begin, b: ;
        b, f: ;
        begin, c: ;
        c, f: ;
        begin, d: ;
        d, f: ;",
        "begin, a: ;
        a, f: ;
        b, f: ;
        c, f: ;
        d, f: ;
        a, b: ;
        b, c: ;
        c, d: ;"
    );

    test_transform!(
        join_fork_prefixes,
        small4,
        "begin, a: ;
        begin, b: ;
        b, c: ;"
    );

    test_transform!(
        join_fork_prefixes,
        small5,
        "begin, a: 1 == 1;
        begin, b: 1 == 1;
        begin, c: 2 == 2;
        begin, d: 2 == 2;
        a, end: ;
        b, end: ;
        c, end: ;
        d, end: ;",
        "begin, a: 1 == 1;
        begin, c: 2 == 2;
        a, end: ;
        b, end: ;
        c, end: ;
        d, end: ;
        a, b: ;
        c, d: ;"
    );

    test_transform!(
        join_fork_prefixes,
        small6,
        "begin, a: ;
        a, b: 1 == 1;
        a, c: 1 == 1;
        g, b: ;
        b, end: 2 == 2;
        c, d: ;",
        "begin, a: ;
        a, c: 1 == 1;
        g, b: ;
        b, end: 2 == 2;
        c, d: ;
        c, b: ;"
    );

    test_transform!(
        join_fork_prefixes,
        small7,
        "begin, a: ;
        a, b: 1 == 1;
        a, c: 1 == 1;
        g, c: ;
        b, end: 2 == 2;
        c, d: ;",
        "begin, a: ;
        a, b: 1 == 1;
        g, c: ;
        b, end: 2 == 2;
        c, d: ;
        b, c: ;"
    );

    test_transform!(
        join_fork_prefixes,
        small8,
        "begin, a: ;
        a, b: 1 == 1;
        a, c: 1 == 1;
        g(a: A), c: ;
        g(a: A), b: ;
        b, end: 2 == 2;
        c, d: ;",
        "begin, a: ;
        a, b: 1 == 1;
        g(a: A), c: ;
        g(a: A), b: ;
        b, end: 2 == 2;
        c, d: ;
        b, c: ;"
    );

    test_transform!(
        join_fork_prefixes,
        small9,
        "begin, a: ;
        a, b: 1 == 1;
        a, c: 1 == 1;
        g(a: A), c: 1 == 1;
        g(a: A), b: ;
        b, end: 2 == 2;
        c, d: ;"
    );

    test_transform!(
        join_fork_prefixes,
        small10,
        "begin, a: ;
        a, b: 1 == 1;
        a, c: 1 == 1;
        f, c: ;
        g(a: A), b: ;
        b, end: 2 == 2;
        c, d: ;"
    );
}
