use std::{collections::BTreeSet, sync::Arc};

use crate::ast::{Edge, Expression, Game, Label};

use super::framework::{Instance, Parameters};

pub struct ReachingDefinitions;

type Id = Arc<str>;
type Domain = BTreeSet<(Id, Option<Edge<Id>>)>;

impl Parameters<Domain> for ReachingDefinitions {
    fn bot() -> Domain {
        BTreeSet::new()
    }

    fn extreme(program: &Game<Id>) -> Domain {
        program
            .variables
            .iter()
            .map(|v| (v.identifier.clone(), None))
            .collect()
    }

    fn join(a: Domain, b: Domain) -> Domain {
        a.union(&b).cloned().collect()
    }

    fn kill(input: Domain, edge: &Edge<Id>) -> Domain {
        match &edge.label {
            Label::Assignment { lhs, .. } => {
                if let Expression::Reference { identifier } = lhs.as_ref() {
                    input
                        .into_iter()
                        .filter(|(id, _)| id != identifier)
                        .collect()
                } else {
                    input
                }
            }
            _ => input,
        }
    }

    fn gen(mut input: Domain, edge: &Edge<Id>) -> Domain {
        match &edge.label {
            Label::Assignment { lhs, .. } => {
                if let Expression::Reference { identifier } = lhs.as_ref() {
                    input.insert((identifier.clone(), Some(edge.clone())));
                    input
                } else {
                    input
                }
            }
            _ => input,
        }
    }
}

impl Instance<Domain> for ReachingDefinitions {
    fn name() -> &'static str {
        "Reaching Definitions"
    }
}
