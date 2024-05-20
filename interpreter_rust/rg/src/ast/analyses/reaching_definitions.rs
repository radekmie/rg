use super::Analysis;
use crate::ast::{Edge, Game};
use std::collections::BTreeMap;
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachingDefinitions;

impl Analysis for ReachingDefinitions {
    type Domain = BTreeMap<Id, Option<Edge<Id>>>;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(program: &Game<Id>) -> Self::Domain {
        program
            .variables
            .iter()
            .map(|v| (v.identifier.clone(), None))
            .collect()
    }

    fn join(mut a: Self::Domain, b: Self::Domain) -> Self::Domain {
        a.extend(b);
        a
    }

    fn kill(input: Self::Domain, _edge: &Edge<Id>) -> Self::Domain {
        input
    }

    fn gen(mut input: Self::Domain, edge: &Edge<Id>) -> Self::Domain {
        if let Some((identifier, _)) = edge.label.as_var_assignment() {
            input.insert(identifier.clone(), Some(edge.clone()));
        }
        input
    }
}
