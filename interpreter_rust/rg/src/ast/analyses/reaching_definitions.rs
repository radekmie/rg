use super::Analysis;
use crate::ast::{Edge, Game, Node};
use std::collections::BTreeMap;
use std::sync::Arc;

type Id = Arc<str>;

#[derive(Clone, Eq, PartialEq)]
pub enum Definition {
    /// All predecessors use this definition or none.
    Any(Arc<Edge<Id>>),
    /// All predecessors use this definition.
    All(Arc<Edge<Id>>),
    /// Predecessors have conflicting definitions.
    Mixed,
}

impl Definition {
    pub fn as_all(&self) -> Option<&Arc<Edge<Id>>> {
        match self {
            Self::All(edge) => Some(edge),
            _ => None,
        }
    }

    pub fn is_mixed(&self) -> bool {
        matches!(self, Self::Mixed)
    }

    fn merge(&mut self, other: Self) {
        match (&self, other) {
            (_, Self::Mixed) => {
                *self = Self::Mixed;
            }
            (Self::All(a) | Self::Any(a), Self::All(b) | Self::Any(b)) if *a != b => {
                *self = Self::Mixed;
            }
            (Self::Any(_), Self::All(b)) => {
                *self = Self::All(b);
            }
            _ => {}
        }
    }

    fn weaken(self) -> Self {
        match self {
            Self::All(edge) => Self::Any(edge),
            _ => self,
        }
    }
}

pub struct ReachingDefinitions;

impl Analysis for ReachingDefinitions {
    type Domain = BTreeMap<Id, Definition>;

    fn bot(&self) -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(&self, game: &Game<Id>) -> Self::Domain {
        // Fake initial value assignments.
        let fake_definition = Definition::All(Arc::from(Edge::new_skip(
            Node::new(Id::from("")),
            Node::new(Id::from("")),
        )));
        game.variables
            .iter()
            .map(|variable| (variable.identifier.clone(), fake_definition.clone()))
            .collect()
    }

    fn join(&self, mut a: Self::Domain, b: Self::Domain) -> Self::Domain {
        for (key, value_b) in b {
            a.entry(key)
                .and_modify(|value_a| value_a.merge(value_b.clone()))
                .or_insert_with(|| value_b.weaken());
        }
        a
    }

    fn kill(&self, mut input: Self::Domain, edge: &Arc<Edge<Id>>) -> Self::Domain {
        if let Some(identifier) = edge.label.as_var_assignment() {
            if !edge.label.is_map_assignment() {
                input.remove(identifier);
            }
        }

        input
    }

    fn gen(&self, mut input: Self::Domain, edge: &Arc<Edge<Id>>) -> Self::Domain {
        if let Some(identifier) = edge.label.as_var_assignment() {
            input.insert(identifier.clone(), Definition::All(edge.clone()));
        }
        input
    }

    fn with_reachability(&self) -> bool {
        true
    }
}
