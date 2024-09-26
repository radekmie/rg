use super::Analysis;
use crate::ast::{Edge, Game};
use std::collections::BTreeSet;
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachingDefinitions;

impl Analysis for ReachingDefinitions {
    type Domain = BTreeSet<(Id, Option<Edge<Id>>)>;
    type Context = ();

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
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

    fn kill(input: Self::Domain, edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        match &edge.label.as_var_assignment() {
            Some((identifier, _)) if !edge.label.is_map_assignment() => input
                .into_iter()
                .filter(|(id, _)| id != *identifier)
                .collect(),
            _ => input,
        }
    }

    fn gen(mut input: Self::Domain, edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        if let Some((identifier, _)) = edge.label.as_var_assignment() {
            input.insert((identifier.clone(), Some(edge.clone())));
        }
        input
    }
}
