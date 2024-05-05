use super::framework::{Instance, Parameters};
use crate::ast::{Edge, Game, Label};
use std::{collections::BTreeSet, sync::Arc};

pub struct ReachingDefinitions;

type Domain = BTreeSet<(Arc<str>, Option<Edge<Arc<str>>>)>;

impl Parameters<Domain> for ReachingDefinitions {
    fn bot(&self) -> Domain {
        BTreeSet::new()
    }

    fn extreme(&self, program: &Game<Arc<str>>) -> Domain {
        program
            .variables
            .iter()
            .map(|v| (v.identifier.clone(), None))
            .collect()
    }

    fn join(&self, a: Domain, b: Domain) -> Domain {
        a.union(&b).cloned().collect()
    }

    fn kill(&self, input: Domain, edge: &Edge<Arc<str>>) -> Domain {
        match &edge.label.as_var_assignment() {
            Some((identifier, _)) => input
                .into_iter()
                .filter(|(id, _)| id != *identifier)
                .collect(),
            _ => input,
        }
    }

    fn gen(&self, mut input: Domain, edge: &Edge<Arc<str>>) -> Domain {
        match &edge.label {
            Label::Assignment { lhs, .. } => {
                if let Some(identifier) = lhs.access_identifier() {
                    input.insert((identifier.clone(), Some(edge.clone())));
                    input
                } else {
                    input
                }
            }
            _ => input,
        }
    }
}

impl Instance<Domain> for ReachingDefinitions {
    fn name() -> &'static str {
        "Reaching Definitions"
    }
}
