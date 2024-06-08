use super::Analysis;
use crate::ast::{Edge, Game, Label};
use std::collections::BTreeMap;
use std::sync::Arc;

type Id = Arc<str>;

const IMPORTANT_VARIABLES: [&str; 3] = ["player", "goals", "visible"];

pub struct ReachingPaths;

impl Analysis for ReachingPaths {
    type Domain = BTreeMap<Option<Id>, bool>;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(_program: &Game<Id>) -> Self::Domain {
        Self::Domain::default()
    }

    fn join(mut a: Self::Domain, b: Self::Domain) -> Self::Domain {
        for (variable, is_repeated) in b.into_iter() {
            a.entry(variable)
                .and_modify(|is_repeated| *is_repeated = true)
                .or_insert(is_repeated);
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
                    .and_modify(|is_repeated| *is_repeated = true)
                    .or_default();
            }
        } else {
            input.entry(None).or_default();
        }

        input
    }
}
