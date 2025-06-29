use super::Analysis;
use crate::ast::{Edge, Game};
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachableNodes {
    with_reachability: bool,
}

impl ReachableNodes {
    #[allow(clippy::new_without_default, reason = "It should be explicit.")]
    pub fn new() -> Self {
        Self {
            with_reachability: false,
        }
    }

    pub fn new_with_reachability() -> Self {
        Self {
            with_reachability: true,
        }
    }
}

impl Analysis for ReachableNodes {
    type Context = ();
    type Domain = bool;

    fn bot(&self) -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(&self, _program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
        true
    }

    fn get_context(&self, _program: &Game<Id>) -> Self::Context {}

    fn join(&self, a: Self::Domain, b: Self::Domain, _ctx: &Self::Context) -> Self::Domain {
        a || b
    }

    fn kill(
        &self,
        input: Self::Domain,
        _edge: &Arc<Edge<Id>>,
        _ctx: &Self::Context,
    ) -> Self::Domain {
        input
    }

    fn gen(
        &self,
        input: Self::Domain,
        _edge: &Arc<Edge<Id>>,
        _ctx: &Self::Context,
    ) -> Self::Domain {
        input
    }

    fn with_reachability(&self) -> bool {
        self.with_reachability
    }
}
