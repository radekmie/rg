use super::Analysis;
use crate::ast::{Edge, Game};
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachableNodes;

impl Analysis for ReachableNodes {
    type Context = ();
    type Domain = bool;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(_program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
        true
    }

    fn get_context(_program: &Game<Id>) -> Self::Context {}

    fn join(a: Self::Domain, b: Self::Domain, _ctx: &Self::Context) -> Self::Domain {
        a || b
    }

    fn kill(input: Self::Domain, _edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        input
    }

    fn gen(input: Self::Domain, _edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        input
    }
}
