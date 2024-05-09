use crate::ast::{Edge, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

pub struct Flow<'a> {
    pub nodes: BTreeSet<&'a Node<Id>>,
    prev_edges: BTreeMap<&'a Node<Id>, BTreeSet<&'a Edge<Id>>>,
    reachability_edges: BTreeMap<&'a Node<Id>, BTreeSet<Edge<Id>>>,
}

impl<'a> Flow<'a> {
    pub fn new(game: &'a Game<Arc<str>>) -> Self {
        let mut reachability_edges = BTreeMap::new();
        for edge in &game.edges {
            if let Label::Reachability { lhs, .. } = &edge.label {
                // Create a `fake` edge simulating start of reachability check
                // node0, node1: ? start -> target;
                // node0 is predecessor of start
                let skip_edge = Edge::new_skip(edge.lhs.clone(), lhs.clone());
                reachability_edges
                    .entry(lhs)
                    .or_insert_with(BTreeSet::new)
                    .insert(skip_edge);
            }
        }
        Self {
            nodes: game.nodes(),
            prev_edges: game.prev_edges(),
            reachability_edges,
        }
    }

    pub fn predecessors(&self, node: &Node<Id>) -> BTreeSet<&Edge<Id>> {
        let mut result = BTreeSet::new();
        result.extend(self.prev_edges.get(node).into_iter().flatten());
        result.extend(self.reachability_edges.get(node).into_iter().flatten());
        result
    }

    pub fn entry(&self) -> Node<Id> {
        Node::new(Id::from("begin"))
    }
}
