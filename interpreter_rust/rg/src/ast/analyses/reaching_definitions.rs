use super::Analysis;
use crate::ast::{Edge, Game};
use std::collections::BTreeMap;
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachingDefinitions;

impl Analysis for ReachingDefinitions {
    type Context = ();
    // Mapping of variable name to an optional edge it was set in last.
    //   * Missing key means it was never reached.
    //   * `Some(edge)` means it was set once or more, and the set values were
    //     the same on all edges.
    //   * `None` means it was set twice or more, and the set values were
    //     different at least once.
    type Domain = BTreeMap<Id, Option<Arc<Edge<Id>>>>;

    fn bot(&self) -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(&self, _program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
        Self::Domain::default()
    }

    fn get_context(&self, _program: &Game<Id>) -> Self::Context {}

    fn join(&self, mut a: Self::Domain, b: Self::Domain, _ctx: &Self::Context) -> Self::Domain {
        for (key, value_b) in b {
            a.entry(key)
                .and_modify(|value_a| {
                    if *value_a != value_b {
                        *value_a = None;
                    }
                })
                .or_insert(value_b);
        }
        a
    }

    fn kill(
        &self,
        mut input: Self::Domain,
        edge: &Arc<Edge<Id>>,
        _ctx: &Self::Context,
    ) -> Self::Domain {
        if let Some(identifier) = edge.label.as_tag_variable() {
            input.remove(identifier);
        } else if let Some(identifier) = edge.label.as_var_assignment() {
            if !edge.label.is_map_assignment() {
                input.remove(identifier);
            }
        }

        input
    }

    fn gen(
        &self,
        mut input: Self::Domain,
        edge: &Arc<Edge<Id>>,
        _ctx: &Self::Context,
    ) -> Self::Domain {
        if let Some(identifier) = edge.label.as_var_assignment() {
            input.insert(identifier.clone(), Some(edge.clone()));
        }
        input
    }

    fn with_reachability(&self) -> bool {
        true
    }
}
