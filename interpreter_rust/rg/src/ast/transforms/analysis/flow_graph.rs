use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use utils::position::Span;

use crate::ast::{Edge, Game, Label, Node};

pub trait FlowGraph<L> {
    fn labels(&self) -> &Vec<L>;
    fn successors(&self, label: &L) -> Option<&BTreeSet<&L>>;
    fn predecessors(&self, label: &L) -> Option<&BTreeSet<&L>>;
    fn is_reversed(&self) -> bool;
    fn entry(&self) -> L;
    fn exit(&self) -> L;
}

type Id = Arc<str>;

pub struct GameFlow<'a> {
    edges: &'a Vec<Edge<Id>>,
    next_edges: BTreeMap<&'a Node<Id>, BTreeSet<&'a Edge<Id>>>,
    prev_edges: BTreeMap<&'a Node<Id>, BTreeSet<&'a Edge<Id>>>,
}

impl<'a> GameFlow<'a> {
    pub fn new(game: &'a Game<Id>) -> Self {
        Self {
            edges: &game.edges,
            next_edges: game.next_edges(),
            prev_edges: game.prev_edges(),
        }
    }
}

impl<'a> FlowGraph<Edge<Id>> for GameFlow<'a> {
    fn labels(&self) -> &Vec<Edge<Id>> {
        self.edges
    }

    fn successors(&self, label: &Edge<Id>) -> Option<&BTreeSet<&Edge<Id>>> {
        self.next_edges.get(&label.rhs)
    }

    fn predecessors(&self, label: &Edge<Id>) -> Option<&BTreeSet<&Edge<Id>>> {
        self.prev_edges.get(&label.rhs)
    }

    fn is_reversed(&self) -> bool {
        false
    }

    fn entry(&self) -> Edge<Id> {
        let lhs = Node::new(Arc::from("<begin>"));
        let rhs = Node::new(Arc::from("begin"));
        Edge::new(Span::none(), lhs, rhs, Label::Skip { span: Span::none() })
    }

    fn exit(&self) -> Edge<Id> {
        let lhs = Node::new(Arc::from("end"));
        let rhs = Node::new(Arc::from("<end>"));
        Edge::new(Span::none(), lhs, rhs, Label::Skip { span: Span::none() })
    }
}
