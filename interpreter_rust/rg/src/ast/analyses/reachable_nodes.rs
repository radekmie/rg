use super::Analysis;
use crate::ast::Game;
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
    type Domain = bool;

    fn bot(&self) -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(&self, _program: &Game<Id>) -> Self::Domain {
        true
    }

    fn join(&self, a: Self::Domain, b: Self::Domain) -> Self::Domain {
        a || b
    }

    fn with_reachability(&self) -> bool {
        self.with_reachability
    }
}
