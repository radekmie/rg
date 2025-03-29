use super::Analysis;
use crate::ast::{Edge, Game};
use std::collections::BTreeMap;
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachingDefinitions;

impl Analysis for ReachingDefinitions {
    type Context = ();
    type Domain = BTreeMap<Id, Arc<Edge<Id>>>;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(_program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
        Self::Domain::default()
    }

    fn get_context(_program: &Game<Id>) -> Self::Context {}

    fn join(mut a: Self::Domain, b: Self::Domain, _ctx: &Self::Context) -> Self::Domain {
        a.retain(|key, value| b.get(key) == Some(value));
        a
    }

    fn kill(mut input: Self::Domain, edge: &Arc<Edge<Id>>, _ctx: &Self::Context) -> Self::Domain {
        if let Some(identifier) = edge.label.as_tag_variable() {
            input.remove(identifier);
        } else if let Some(identifier) = edge.label.as_var_assignment() {
            if !edge.label.is_map_assignment() {
                input.remove(identifier);
            }
        }

        input
    }

    fn gen(mut input: Self::Domain, edge: &Arc<Edge<Id>>, _ctx: &Self::Context) -> Self::Domain {
        if let Some(identifier) = edge.label.as_var_assignment() {
            input.insert(identifier.clone(), edge.clone());
        }
        input
    }
}
