use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use crate::ast::Game;

type Id = Arc<str>;
type Node = crate::ast::Node<Id>;
type Edge = crate::ast::Edge<Id>;

pub struct Flow<'a> {
    nodes: BTreeSet<&'a Node>,
    edges: Vec<Edge>,
    next_edges: BTreeMap<&'a Node, BTreeSet<&'a Edge>>,
    prev_edges: BTreeMap<&'a Node, BTreeSet<&'a Edge>>,
}

impl<'a> Flow<'a> {
    pub fn new(game: &'a Game<Id>) -> Self {
        Self {
            nodes: game.nodes(),
            edges: game.edges.clone(),
            next_edges: game.next_edges(),
            prev_edges: game.prev_edges(),
        }
    }
}

impl<'a> Flow<'a> {
    pub fn nodes(&self) -> &BTreeSet<&'a Node> {
        &self.nodes
    }

    pub fn edges(&self) -> &Vec<Edge> {
        &self.edges
    }

    pub fn successors(&self, node: &Node) -> Option<&BTreeSet<&Edge>> {
        self.next_edges.get(node)
    }

    pub fn predecessors(&self, node: &Node) -> Option<&BTreeSet<&Edge>> {
        self.prev_edges.get(node)
    }

    pub fn is_reversed(&self) -> bool {
        false
    }

    pub fn entry(&self) -> Node {
        Node::new(Arc::from("begin"))
    }

    pub fn exit(&self) -> Node {
        Node::new(Arc::from("end"))
    }
}
