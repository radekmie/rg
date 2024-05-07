use super::flow::Flow;
use super::framework::{Instance, Parameters};
use crate::ast::{Edge, Game, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub struct ReachingDefinitions;

type Id = Arc<str>;
type Domain = BTreeSet<(Id, Option<Edge<Id>>)>;

impl Parameters<Domain> for ReachingDefinitions {
    fn bot() -> Domain {
        Domain::default()
    }

    fn extreme(program: &Game<Id>) -> Domain {
        program
            .variables
            .iter()
            .map(|v| (v.identifier.clone(), None))
            .collect()
    }

    fn join(mut a: Domain, b: Domain) -> Domain {
        a.extend(b);
        a
    }

    fn kill(input: Domain, edge: &Edge<Id>) -> Domain {
        match &edge.label.as_var_assignment() {
            Some((identifier, _)) if !&edge.label.is_map_assignment() => input
                .into_iter()
                .filter(|(id, _)| id != *identifier)
                .collect(),
            _ => input,
        }
    }

    fn gen(mut input: Domain, edge: &Edge<Id>) -> Domain {
        if let Some((identifier, _)) = edge.label.as_var_assignment() {
            input.insert((identifier.clone(), Some(edge.clone())));
        }
        input
    }
}

impl Instance<Domain, Self> for ReachingDefinitions {}

impl Game<Id> {
    pub fn reaching_definitions(&self, debug: bool) -> BTreeMap<Node<Id>, Domain> {
        let flow = Flow::new(self);
        let result = ReachingDefinitions.analyse(&flow, self);
        if debug {
            for (node, defs) in &result {
                println!("Node: {node}");
                println!("Definitions: ");
                for (id, edge) in defs {
                    if let Some(edge) = edge {
                        println!("  {id} :  \"{edge}\"");
                    } else {
                        println!("  {id} : None");
                    }
                }
                println!();
            }
        }
        result
    }
}
