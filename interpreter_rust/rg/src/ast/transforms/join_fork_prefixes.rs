use crate::ast::{Edge, Error, Game, Node};
use std::{collections::BTreeSet, sync::Arc};

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
        let mut used = BTreeSet::new();
        let next_edges = self.next_edges();
        let mut edges_to_join = vec![];
        for edge in &self.edges {
            if edge.has_bindings()
                || used.contains(&edge.lhs)
                || !self.incoming_edges(&edge.lhs).all(|e| e.lhs == edge.lhs)
            {
                continue;
            }
            let from_same_node = next_edges.get(&edge.lhs).unwrap();
            let to_join: Vec<Edge<Id>> = from_same_node
                .iter()
                .filter(|e| e.label == edge.label && e.rhs != edge.rhs && !e.has_bindings())
                .map(|e| (**e).clone())
                .collect();
            if to_join.is_empty() {
                continue;
            }
            let to_add: Vec<_> = {
                let new_set = BTreeSet::new();
                let from_rhs = next_edges.get(&edge.rhs).unwrap_or(&new_set);
                to_join
                    .iter()
                    .filter(|e| {
                        !from_rhs
                            .iter()
                            .any(|from_rhs| from_rhs.label.is_skip() && from_rhs.rhs == e.rhs)
                    })
                    .map(|e| e.rhs.clone())
                    .collect()
            };
            used.insert(edge.lhs.clone());
            edges_to_join.push((edge.rhs.clone(), to_join, to_add));
        }
        edges_to_join
    }
}
