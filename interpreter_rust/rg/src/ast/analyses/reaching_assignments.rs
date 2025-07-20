use super::Analysis;
use crate::ast::{Edge, Game, Label, Node, Pragma};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

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
    fn add_condition(&mut self, condition: &Arc<Edge<Id>>, context: &Context) {
        self.is_conflicting = self.is_conflicting
            || self
                .conditions
                .iter()
                .any(|x| context.is_disjoint(x, condition));

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

    fn join(&mut self, other: &Self, context: &Context) {
        self.is_conflicting = self.is_conflicting
            || other.is_conflicting
            || self
                .conditions
                .iter()
                .any(|x| other.conditions.iter().any(|y| context.is_disjoint(x, y)));

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

pub struct Context {
    disjoints: BTreeMap<Node<Id>, BTreeSet<Node<Id>>>,
}

impl Context {
    fn is_disjoint(&self, x: &Edge<Id>, y: &Edge<Id>) -> bool {
        // Either their labels are negated or they are marked with a `@disjoint`
        // or `@disjointExhaustive` pragma already.
        x.label.is_negated(&y.label)
            || x.lhs == y.lhs
                && self
                    .disjoints
                    .get(&x.lhs)
                    .is_some_and(|nodes| nodes.contains(&x.rhs) && nodes.contains(&y.rhs))
    }
}

pub struct ReachingAssignments {
    variables: Variables,
    ctx: Context,
}

pub enum Variables {
    Exclude(BTreeSet<Id>),
    Include(BTreeSet<Id>),
}

impl ReachingAssignments {
    pub fn is_tracked_variable(&self, identifier: &Id) -> bool {
        match &self.variables {
            Variables::Exclude(variables) => !variables.contains(identifier),
            Variables::Include(variables) => variables.contains(identifier),
        }
    }
}

impl Analysis for ReachingAssignments {
    type Domain = BTreeMap<Option<Id>, Assignment>;

    fn bot(&self) -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(&self, _program: &Game<Id>) -> Self::Domain {
        Self::Domain::default()
    }

    fn join(&self, mut a: Self::Domain, b: Self::Domain) -> Self::Domain {
        for (variable, b_reached) in b.into_iter() {
            a.entry(variable)
                .and_modify(|a_reached| a_reached.join(&b_reached, &self.ctx))
                .or_insert(b_reached);
        }
        a
    }

    fn kill(&self, mut input: Self::Domain, edge: &Arc<Edge<Id>>) -> Self::Domain {
        if edge.label.is_player_assignment() || edge.label.is_tag() || edge.label.is_tag_variable()
        {
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
            if !assignment.is_conflicting {
                assignment.is_repeated = false;
            }
        }

        input
    }

    fn gen(&self, mut input: Self::Domain, edge: &Arc<Edge<Id>>) -> Self::Domain {
        if edge.label.is_assignment() {
            let variable = edge.label.as_var_assignment().unwrap();
            if self.is_tracked_variable(variable) {
                input
                    .entry(Some(variable.clone()))
                    .and_modify(|a_reached| a_reached.add_source(&edge.lhs))
                    .or_insert_with(|| Assignment::new(&edge.lhs));
            }
        } else if !edge.label.is_tag() && !edge.label.is_tag_variable() {
            input
                .entry(None)
                .or_insert_with(|| Assignment::new(&edge.lhs));
            if !edge.label.is_skip() {
                for assignment in input.values_mut() {
                    assignment.add_condition(edge, &self.ctx);
                }
            }
        }

        input
    }
}

impl From<&Game<Id>> for ReachingAssignments {
    fn from(game: &Game<Id>) -> Self {
        let is_translated_from_rbg = game
            .pragmas
            .iter()
            .any(|pragma| matches!(pragma, Pragma::TranslatedFromRbg { .. }));
        let ctx = Context {
            disjoints: game
                .pragmas
                .iter()
                .fold(BTreeMap::new(), |mut disjoints, pragma| {
                    if let Pragma::Disjoint { node, nodes, .. }
                    | Pragma::DisjointExhaustive { node, nodes, .. } = pragma
                    {
                        disjoints
                            .entry(node.clone())
                            .or_default()
                            .extend(nodes.iter().cloned());
                    }
                    disjoints
                }),
        };

        let variables = if is_translated_from_rbg {
            Variables::Include(BTreeSet::from([Id::from("coord")]))
        } else {
            Variables::Exclude(BTreeSet::from(["player", "goals", "visible"].map(Id::from)))
        };
        Self { variables, ctx }
    }
}
