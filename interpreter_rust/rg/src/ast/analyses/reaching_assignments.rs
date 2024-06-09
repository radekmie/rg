use super::Analysis;
use crate::ast::{Edge, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

const IMPORTANT_VARIABLES: [&str; 3] = ["player", "goals", "visible"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assignment {
    pub is_repeated: bool,
    sources: BTreeSet<Node<Id>>,
}

impl Assignment {
    fn add(&mut self, node: &Node<Id>) {
        if self.sources.contains(node) {
            self.is_repeated = true;
        } else {
            self.sources.insert(node.clone());
        }
    }

    fn join(&mut self, other: &Self) {
        for node in &other.sources {
            self.add(node);
        }
    }

    fn new(node: &Node<Id>) -> Self {
        Self {
            is_repeated: false,
            sources: BTreeSet::from([node.clone()]),
        }
    }
}

pub struct ReachingAssignments;

impl Analysis for ReachingAssignments {
    type Domain = BTreeMap<Option<Id>, Assignment>;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(_program: &Game<Id>) -> Self::Domain {
        Self::Domain::default()
    }

    fn join(mut a: Self::Domain, b: Self::Domain) -> Self::Domain {
        for (variable, b_reached) in b.into_iter() {
            a.entry(variable)
                .and_modify(|a_reached| a_reached.join(&b_reached))
                .or_insert(b_reached);
        }
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
                input.retain(|variable, _| variable.as_ref() != Some(lhs));
            }
            if let Some(rhs) = rhs.uncast().as_reference() {
                input.retain(|variable, _| variable.as_ref() != Some(rhs));
            }
            input.remove(&None);
        }
        input
    }

    fn gen(mut input: Self::Domain, edge: &Edge<Id>) -> Self::Domain {
        if let Some((variable, _)) = edge.label.as_var_assignment() {
            if !IMPORTANT_VARIABLES.contains(&variable.as_ref()) {
                input
                    .entry(Some(variable.clone()))
                    .and_modify(|a_reached| a_reached.add(&edge.lhs))
                    .or_insert_with(|| Assignment::new(&edge.lhs));
            }
        } else {
            input
                .entry(None)
                .or_insert_with(|| Assignment::new(&edge.lhs));
        }

        input
    }
}
