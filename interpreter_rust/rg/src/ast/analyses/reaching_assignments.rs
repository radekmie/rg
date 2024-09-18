use super::Analysis;
use crate::ast::{Edge, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

const IMPORTANT_VARIABLES: [&str; 3] = ["player", "goals", "visible"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assignment {
    /// Set of `Label::Comparison` and `Label::Reachability` that led to this
    /// assignment from any of the `sources`.
    conditions: BTreeSet<Label<Id>>,
    /// Whether any of the `conditions` is already conflicting.
    is_conflicting: bool,
    /// Whether any of the `sources` is repeated with the same conditions.
    pub is_repeated: bool,
    /// Set of nodes that led to this assignment.
    sources: BTreeSet<Node<Id>>,
}

impl Assignment {
    fn add_condition(&mut self, condition: &Label<Id>) {
        self.is_conflicting =
            self.is_conflicting || self.conditions.iter().any(|x| x.is_negated(condition));

        // Add condition only if it may cause a conflict in the future.
        if !self.is_conflicting {
            self.conditions.insert(condition.clone());
        }
    }

    fn add_source(&mut self, source: &Node<Id>) {
        self.is_repeated = self.is_repeated || self.sources.contains(source);

        // Add source only if it may cause a repeat in the future.
        if !self.is_repeated {
            self.sources.insert(source.clone());
        }
    }

    fn join(&mut self, other: &Self) {
        self.is_conflicting = self.is_conflicting
            || other.is_conflicting
            || self
                .conditions
                .iter()
                .any(|x| other.conditions.iter().any(|y| x.is_negated(y)));

        // Add conditions only if they may cause a conflict in the future.
        if !self.is_conflicting {
            self.conditions.extend(other.conditions.clone());
        }

        self.is_repeated = self.is_repeated
            || other.is_repeated
            || !self.is_conflicting && !self.sources.is_disjoint(&other.sources);

        // Add sources only if they may cause a repeat in the future.
        if !self.is_repeated {
            self.sources.extend(other.sources.clone());
        }
    }

    fn new(source: &Node<Id>) -> Self {
        Self {
            conditions: BTreeSet::new(),
            is_conflicting: false,
            is_repeated: false,
            sources: BTreeSet::from([source.clone()]),
        }
    }
}

pub struct ReachingAssignments;

impl Analysis for ReachingAssignments {
    type Domain = BTreeMap<Option<Id>, Assignment>;
    type Context = ();

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(_program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
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

    fn kill(mut input: Self::Domain, edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
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
        }
        input
    }

    fn gen(mut input: Self::Domain, edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        match &edge.label {
            Label::Assignment { .. } => {
                let variable = edge.label.as_var_assignment().unwrap().0;
                if !IMPORTANT_VARIABLES.contains(&variable.as_ref()) {
                    input
                        .entry(Some(variable.clone()))
                        .and_modify(|a_reached| a_reached.add_source(&edge.lhs))
                        .or_insert_with(|| Assignment::new(&edge.lhs));
                }
            }
            Label::Comparison { .. } | Label::Reachability { .. } => {
                for assignment in input.values_mut() {
                    assignment.add_condition(&edge.label);
                }
            }
            Label::Skip { .. } => {
                , _ctx: &Self::Context                input
                    .entry(None)
                    .or_insert_with(|| Assignment::new(&edge.lhs));
            }
            Label::Tag { .. } => {}
        }
        input
    }

    fn get_context(_program: &Game<super::Id>) -> Self::Context {}
}
