use super::Analysis;
use crate::ast::{Edge, Game, Label};
use std::collections::BTreeMap;
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachingBindingAssignments;

impl Analysis for ReachingBindingAssignments {
    type Context = ();
    type Domain = BTreeMap<Id, (Arc<Edge<Id>>, Id)>;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(_program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
        Self::Domain::default()
    }

    fn join(mut a: Self::Domain, b: Self::Domain, _ctx: &Self::Context) -> Self::Domain {
        // Keep only keys present in both maps with the same value.
        a.retain(|key, value| b.get(key) == Some(value));
        a
    }

    fn kill(mut input: Self::Domain, edge: &Arc<Edge<Id>>, _ctx: &Self::Context) -> Self::Domain {
        if let Some((identifier, _)) = edge.label.as_var_assignment() {
            input.remove(identifier);
        }
        input
    }

    fn gen(mut input: Self::Domain, edge: &Arc<Edge<Id>>, _ctx: &Self::Context) -> Self::Domain {
        if let Label::Assignment { lhs, rhs } = &edge.label {
            if let (Some(lhs), Some(rhs)) =
                (lhs.uncast().as_reference(), rhs.uncast().as_reference())
            {
                if edge.has_binding(rhs) {
                    input.insert(lhs.clone(), (edge.clone(), rhs.clone()));
                }
            }
        }
        input
    }

    fn get_context(_program: &Game<super::Id>) -> Self::Context {}
}
