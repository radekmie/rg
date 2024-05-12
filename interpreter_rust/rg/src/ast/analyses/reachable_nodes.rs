use super::Analysis;
use crate::ast::{Edge, Game};
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachableNodes;

impl Analysis for ReachableNodes {
    type Domain = bool;

    const FLOW_WITH_REACHABILITY: bool = false;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(_program: &Game<Id>) -> Self::Domain {
        true
    }

    fn join(a: Self::Domain, b: Self::Domain) -> Self::Domain {
        a || b
    }

    fn kill(input: Self::Domain, _edge: &Edge<Id>) -> Self::Domain {
        input
    }

    fn gen(input: Self::Domain, _edge: &Edge<Id>) -> Self::Domain {
        input
    }
}
