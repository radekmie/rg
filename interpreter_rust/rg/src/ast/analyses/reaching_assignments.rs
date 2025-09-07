use super::Analysis;
use crate::ast::{Edge, Game, Label, Node, Pragma};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type Disjoints = BTreeMap<Node<Id>, BTreeSet<Node<Id>>>;

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
    fn add_condition(&mut self, condition: &Arc<Edge<Id>>, analysis: &ReachingAssignments) {
        self.is_conflicting = self.is_conflicting
            || self
                .conditions
                .iter()
                .any(|x| analysis.is_disjoint(x, condition));

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

    fn join(&mut self, other: &Self, analysis: &ReachingAssignments) {
        self.is_conflicting = self.is_conflicting
            || other.is_conflicting
            || self
                .conditions
                .iter()
                .any(|x| other.conditions.iter().any(|y| analysis.is_disjoint(x, y)));

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

pub enum Variables {
    Exclude(BTreeSet<Id>),
    Include(BTreeSet<Id>),
}

pub struct ReachingAssignments {
    disjoints: Disjoints,
    variables: Variables,
}

impl ReachingAssignments {
    pub fn is_tracked_variable(&self, identifier: &Id) -> bool {
        match &self.variables {
            Variables::Exclude(variables) => !variables.contains(identifier),
            Variables::Include(variables) => variables.contains(identifier),
        }
    }

    fn is_disjoint(&self, x: &Edge<Id>, y: &Edge<Id>) -> bool {
        // Either their labels are negated or they are marked with a `@disjoint`
        // or `@disjointExhaustive` pragma already.
        x.label.is_negated(&y.label)
            || x.lhs == y.lhs
                && x.rhs != y.rhs
                && self
                    .disjoints
                    .get(&x.lhs)
                    .is_some_and(|nodes| nodes.contains(&x.rhs) && nodes.contains(&y.rhs))
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
                .and_modify(|a_reached| a_reached.join(&b_reached, self))
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
                    assignment.add_condition(edge, self);
                }
            }
        }

        input
    }
}

impl From<&Game<Id>> for ReachingAssignments {
    fn from(game: &Game<Id>) -> Self {
        let disjoints =
            game.pragmas
                .iter()
                .fold(BTreeMap::new(), |mut disjoints: Disjoints, pragma| {
                    if let Pragma::Disjoint { node, nodes, .. }
                    | Pragma::DisjointExhaustive { node, nodes, .. } = pragma
                    {
                        disjoints
                            .entry(node.clone())
                            .or_default()
                            .extend(nodes.iter().cloned());
                    }
                    disjoints
                });

        let is_translated_from_rbg = game
            .pragmas
            .iter()
            .any(|pragma| matches!(pragma, Pragma::TranslatedFromRbg { .. }));
        let variables = if is_translated_from_rbg {
            Variables::Include(BTreeSet::from([Id::from("coord")]))
        } else {
            Variables::Exclude(BTreeSet::from(["player", "goals", "visible"].map(Id::from)))
        };

        Self {
            disjoints,
            variables,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Analysis, ReachingAssignments};
    use crate::ast::{Game, Node};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn format_analysis(
        analysis: BTreeMap<Node<Arc<str>>, <ReachingAssignments as Analysis>::Domain>,
    ) -> String {
        let mut result = String::new();
        result.push('\n');
        for (node, variables) in analysis {
            result.push_str(&format!("        {node}:\n"));
            for (variable, assignment) in variables {
                // assignment.
                result.push_str(&format!(
                    "            {}{}{}:\n                conditions:\n",
                    variable.unwrap_or_else(|| Arc::from("(none)")),
                    if assignment.is_conflicting {
                        " [conflicting]"
                    } else {
                        ""
                    },
                    if assignment.is_repeated {
                        " [repeated]"
                    } else {
                        ""
                    }
                ));
                for condition in &assignment.conditions {
                    result.push_str(&format!("                    {condition}\n"));
                }
                result.push_str("                sources:\n");
                for source in &assignment.sources {
                    result.push_str(&format!("                    {source}\n"));
                }
            }
        }
        result
    }

    macro_rules! test {
        ($name:ident, $source:expr, $expect:expr) => {
            #[test]
            fn $name() {
                Game::test_analysis(
                    $source,
                    $expect,
                    Box::new(|game| ReachingAssignments::from(game)),
                    Box::new(format_analysis),
                );
            }
        };
    }

    test!(
        hex_simple,
        "@disjointExhaustive a : b c;
        type Position = { 0, 1, 2 };
        const check: Position -> Bool = { :1 };
        const left: Position -> Position = { :0, 2: 1 };
        const right: Position -> Position = { :2, 0: 1 };
        var board: Position -> Bool = { :0 };
        var position: Position = 0;
        begin, end: ? a -> b;
        a, b: board[position] == 1;
        a, c: board[position] != 1;
        c, d: position = left[position];
        c, d: position = right[position];
        d, a: check[position] == 1;",
        "a:
            (none):
                conditions:
                    a, c: board[position] != 1;
                    d, a: check[position] == 1;
                sources:
                    d
            position:
                conditions:
                    a, c: board[position] != 1;
                    d, a: check[position] == 1;
                sources:
                    c
        b:
            (none) [conflicting]:
                conditions:
                    a, c: board[position] != 1;
                    d, a: check[position] == 1;
                sources:
                    d
            position [conflicting]:
                conditions:
                    a, c: board[position] != 1;
                    d, a: check[position] == 1;
                sources:
                    c
        begin:
        c:
            (none):
                conditions:
                    a, c: board[position] != 1;
                    d, a: check[position] == 1;
                sources:
                    d
            position:
                conditions:
                    a, c: board[position] != 1;
                    d, a: check[position] == 1;
                sources:
                    c
        d:
            (none) [repeated]:
                conditions:
                    a, c: board[position] != 1;
                    d, a: check[position] == 1;
                sources:
                    d
            position [repeated]:
                conditions:
                    a, c: board[position] != 1;
                    d, a: check[position] == 1;
                sources:
                    c
        end:
            (none):
                conditions:
                    begin, end: ? a -> b;
                sources:
                    begin"
    );
}
