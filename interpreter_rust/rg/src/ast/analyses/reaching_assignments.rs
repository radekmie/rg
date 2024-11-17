use super::Analysis;
use crate::ast::{Edge, Game, Label, Node, Pragma};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

const IMPORTANT_VARIABLES: [&str; 3] = ["player", "goals", "visible"];

#[derive(Clone, Eq, PartialEq)]
pub struct Assignment {
    /// Set of edges with `Label::Comparison` and `Label::Reachability` labels
    /// that led to this assignment from any of the `sources`.
    conditions: BTreeSet<Arc<Edge<Id>>>,
    /// Whether any of the `conditions` is already conflicting.
    is_conflicting: bool,
    /// Whether any of the `sources` is repeated with the same conditions.
    pub is_repeated: bool,
    /// Set of nodes that led to this assignment.
    sources: BTreeSet<Arc<Node<Id>>>,
}

impl Assignment {
    fn add_condition(&mut self, condition: &Arc<Edge<Id>>, disjoints: &Disjoints) {
        self.is_conflicting = self.is_conflicting
            || self
                .conditions
                .iter()
                .any(|x| disjoints.is_disjoint(x, condition));

        // Add condition only if it may cause a conflict in the future.
        if !self.is_conflicting {
            self.conditions.insert(condition.clone());
        }
    }

    fn add_source(&mut self, source: &Node<Id>) {
        self.is_repeated = self.is_repeated || self.sources.contains(source);

        // Add source only if it may cause a repeat in the future.
        if !self.is_repeated {
            self.sources.insert(Arc::from(source.clone()));
        }
    }

    fn join(&mut self, other: &Self, disjoints: &Disjoints) {
        self.is_conflicting = self.is_conflicting
            || other.is_conflicting
            || self
                .conditions
                .iter()
                .any(|x| other.conditions.iter().any(|y| disjoints.is_disjoint(x, y)));

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
            sources: BTreeSet::from([Arc::from(source.clone())]),
        }
    }
}

pub struct Disjoints(Vec<(Node<Id>, Vec<Node<Id>>)>);

impl Disjoints {
    fn is_disjoint(&self, x: &Edge<Id>, y: &Edge<Id>) -> bool {
        // Either their labels are negated or they are marked with a `@disjoint`
        // or `@disjointExhaustive` pragma already.
        x.label.is_negated(&y.label)
            || x.lhs == y.lhs
                && self.0.iter().any(|(node, nodes)| {
                    *node == x.lhs && nodes.contains(&x.rhs) && nodes.contains(&y.rhs)
                })
    }

    fn new(game: &Game<Id>) -> Self {
        Self(
            game.pragmas
                .iter()
                .filter_map(|pragma| match pragma {
                    Pragma::Disjoint { node, nodes, .. }
                    | Pragma::DisjointExhaustive { node, nodes, .. } => {
                        Some((node.clone(), nodes.clone()))
                    }
                    _ => None,
                })
                .collect(),
        )
    }
}

pub struct ReachingAssignments;

impl Analysis for ReachingAssignments {
    type Context = Disjoints;
    type Domain = BTreeMap<Option<Id>, Assignment>;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(_program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
        Self::Domain::default()
    }

    fn get_context(program: &Game<Id>) -> Self::Context {
        Self::Context::new(program)
    }

    fn join(mut a: Self::Domain, b: Self::Domain, ctx: &Self::Context) -> Self::Domain {
        for (variable, b_reached) in b.into_iter() {
            a.entry(variable)
                .and_modify(|a_reached| a_reached.join(&b_reached, ctx))
                .or_insert(b_reached);
        }
        a
    }

    fn kill(mut input: Self::Domain, edge: &Arc<Edge<Id>>, _ctx: &Self::Context) -> Self::Domain {
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

        // If it was repeated in the previous node, it does not have to be
        // repeated here.
        for assignment in input.values_mut() {
            assignment.is_repeated = false;
        }

        input
    }

    fn gen(mut input: Self::Domain, edge: &Arc<Edge<Id>>, ctx: &Self::Context) -> Self::Domain {
        if edge.label.is_assignment() {
            let variable = edge.label.as_var_assignment().unwrap().0;
            if !IMPORTANT_VARIABLES.contains(&variable.as_ref()) {
                input
                    .entry(Some(variable.clone()))
                    .and_modify(|a_reached| a_reached.add_source(&edge.lhs))
                    .or_insert_with(|| Assignment::new(&edge.lhs));
            }
        } else if !edge.label.is_tag() {
            input
                .entry(None)
                .or_insert_with(|| Assignment::new(&edge.lhs));
            if !edge.label.is_skip() {
                for assignment in input.values_mut() {
                    assignment.add_condition(edge, ctx);
                }
            }
        }

        input
    }
}
