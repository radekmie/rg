use crate::ast::{Edge, Error, Game, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type ToInline = (Node<Id>, Vec<Edge<Id>>, Vec<Node<Id>>);

/// Before:
/// b, a1: x = 1;
/// b, a2: x = 1;
/// a1, a3: ;
/// a2, a4: ;
///
/// After:
/// b, a1: x = 1;
/// a1, a3: ;
/// a1, a4: ;

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
                self.edges.push(edge);
            }
        }
        changed
    }

    fn find_to_join_prefixes(&self) -> Vec<ToInline> {
        let next_edges = self.next_edges();
        let default_set = BTreeSet::new();
        let mut edges_to_join = vec![];
        // Start of the fork prefix
        for node in self.nodes() {
            // Get all outgoing edges that do not have bindings
            let next_edges = next_edges
                .get(&node)
                .unwrap_or(&default_set)
                .iter()
                .filter(|e| !e.has_bindings());
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
                // Add skip edges from the first edge to the rest
                if let [head, tail @ ..] = &edges[..] {
                    let node = head.rhs.clone();
                    let to_add = tail.iter().map(|e| e.rhs.clone()).collect::<Vec<_>>();
                    let to_remove = tail.iter().map(|e| (**e).clone()).collect::<Vec<_>>();
                    edges_to_join.push((node, to_remove, to_add));
                }
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
        b, end: 2 == 2;",
        "begin, a: ;
        a, b: 1 == 1;
        b, end: 2 == 2;
        b, c: ;"
    );

    test_transform!(
        join_fork_prefixes,
        small2,
        "begin, a: ;
        begin, b: ;",
        "begin, a: ;
        a, b: ;"
    );
}
