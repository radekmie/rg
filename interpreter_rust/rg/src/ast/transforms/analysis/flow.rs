use crate::ast::{Game, Label};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type Node = crate::ast::Node<Id>;
type Edge = crate::ast::Edge<Id>;

pub struct Flow<'a> {
    nodes: BTreeSet<&'a Node>,
    prev_edges: BTreeMap<&'a Node, BTreeSet<&'a Edge>>,
    reachability_edges: BTreeMap<&'a Node, BTreeSet<Edge>>,
}

impl<'a> Flow<'a> {
    pub fn new(game: &'a Game<Id>) -> Self {
        let mut reachability_edges = BTreeMap::new();
        for edge in game.edges.iter() {
            if let Label::Reachability { lhs, .. } = &edge.label {
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

    pub fn predecessors(&self, node: &Node) -> Option<BTreeSet<&Edge>> {
        self.prev_edges.get(node).map(|prev| {
            let mut result = BTreeSet::new();
            result.extend(prev.iter());
            if let Some(reachability_edges) = self.reachability_edges.get(node) {
                result.extend(reachability_edges.iter());
            }
            result
        })
    }

    pub fn entry(&self) -> Node {
        Node::new(Arc::from("begin"))
    }
}
