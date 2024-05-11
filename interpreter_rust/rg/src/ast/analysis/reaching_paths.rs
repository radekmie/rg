use super::Analysis;
use crate::ast::{Edge, Game, Node};
use std::collections::BTreeSet;
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachingPaths;

#[derive(Clone, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Path {
    pub has_duplicate: bool,
    nodes: Vec<Node<Id>>,
}

impl Path {
    pub fn push(&mut self, node: &Node<Id>) {
        if !self.has_duplicate {
            self.has_duplicate = self.nodes.contains(node);
            self.nodes.push(node.clone());
        }
    }
}

impl Analysis for ReachingPaths {
    type Domain = BTreeSet<Path>;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(_program: &Game<Id>) -> Self::Domain {
        Self::Domain::from([Path::default()])
    }

    fn join(mut a: Self::Domain, b: Self::Domain) -> Self::Domain {
        a.extend(b);
        a
    }

    fn kill(mut input: Self::Domain, edge: &Edge<Id>) -> Self::Domain {
        if edge.label.is_player_assignment() || edge.label.is_tag() {
            input.clear();
        }
        input
    }

    fn gen(input: Self::Domain, edge: &Edge<Id>) -> Self::Domain {
        input
            .into_iter()
            .map(|mut path| {
                path.push(&edge.lhs);
                path
            })
            .collect()
    }
}
