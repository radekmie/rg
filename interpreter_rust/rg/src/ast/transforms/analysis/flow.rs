use crate::ast::{Game, Label};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Node = crate::ast::Node<Arc<str>>;
type Edge = crate::ast::Edge<Arc<str>>;

pub struct Flow<'a> {
    nodes: BTreeSet<&'a Node>,
    prev_edges: BTreeMap<&'a Node, BTreeSet<&'a Edge>>,
    reachability_edges: BTreeMap<&'a Node, BTreeSet<Edge>>,
}

impl<'a> Flow<'a> {
    pub fn new(game: &'a Game<Arc<str>>) -> Self {
        let mut reachability_edges = BTreeMap::new();
        for edge in game.edges.iter() {
            if let Label::Reachability { lhs, .. } = &edge.label {
                // Create a `fake` edge simulating start of reachability check
                // node0, node1: ? start -> target;
                // node0 is predecessor of start
                let skip_edge = Edge::skip_edge(edge.lhs.clone(), lhs.clone());
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

    pub fn nodes(&self) -> &BTreeSet<&'a Node> {
        &self.nodes
    }

    pub fn predecessors(&self, node: &Node) -> BTreeSet<&Edge> {
        let mut result = BTreeSet::new();
        if let Some(prev_edges) = self.prev_edges.get(node) {
            result.extend(prev_edges.iter());
        }
        if let Some(reachability_edges) = self.reachability_edges.get(node) {
            result.extend(reachability_edges.iter());
        }
        result
    }

    pub fn entry(&self) -> Node {
        Node::new(Arc::from("begin"))
    }
}
