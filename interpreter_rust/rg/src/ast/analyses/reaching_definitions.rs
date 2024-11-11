use super::Analysis;
use crate::ast::{Edge, Game};
use std::cmp::Ordering;
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachingDefinitions;

impl Analysis for ReachingDefinitions {
    type Context = ();
    type Domain = Vec<(Id, Option<Arc<Edge<Id>>>)>;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
        let mut domain: Vec<_> = program
            .variables
            .iter()
            .map(|v| (v.identifier.clone(), None))
            .collect();
        domain.sort_unstable();
        domain
    }

    fn join(mut a: Self::Domain, b: Self::Domain) -> Self::Domain {
        for x in b {
            if let Err(index) = a.binary_search(&x) {
                a.insert(index, x);
            }
        }
        a
    }

    fn kill(mut input: Self::Domain, edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        if let Some((identifier, _)) = edge.label.as_var_assignment() {
            if !edge.label.is_map_assignment() {
                input.retain(|(id, _)| id != identifier);
            }
        }
        input
    }

    fn gen(mut input: Self::Domain, edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        if let Some((identifier, _)) = edge.label.as_var_assignment() {
            if let Err(index) = position(&input, identifier, edge) {
                input.insert(index, (identifier.clone(), Some(Arc::from(edge.clone()))));
            }
        }
        input
    }
}

fn position(
    input: &<ReachingDefinitions as Analysis>::Domain,
    identifier: &Id,
    edge: &Edge<Id>,
) -> Result<usize, usize> {
    input.binary_search_by(|(id, e)| {
        id.cmp(identifier)
            .then_with(|| e.as_ref().map_or(Ordering::Equal, |e| e.as_ref().cmp(edge)))
    })
}
