use super::Analysis;
use crate::ast::{Edge, Game, Label, Node};
use std::collections::BTreeSet;
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachingPaths;

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
        } else if let Label::Comparison {
            lhs,
            rhs,
            negated: false,
        } = &edge.label
        {
            if let Some(lhs) = lhs.uncast().as_reference() {
                input.retain(|path| !path.variables.contains(lhs));
            }
            if let Some(rhs) = rhs.uncast().as_reference() {
                input.retain(|path| !path.variables.contains(rhs));
            }
        }
        input
    }

    fn gen(input: Self::Domain, edge: &Edge<Id>) -> Self::Domain {
        input
            .into_iter()
            .map(|mut path| {
                path.push(edge);
                path
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Path {
    pub has_duplicate: bool,
    pub nodes: BTreeSet<Node<Id>>,
    pub variables: BTreeSet<Id>,
}

impl Path {
    pub fn push(&mut self, edge: &Edge<Id>) {
        if !self.has_duplicate {
            self.has_duplicate = self.nodes.contains(&edge.lhs);
            if !self.has_duplicate {
                self.nodes.insert(edge.lhs.clone());
                if let Some((variable, _)) = edge.label.as_var_assignment() {
                    self.variables.insert(variable.clone());
                }
            }
        }
    }
}
