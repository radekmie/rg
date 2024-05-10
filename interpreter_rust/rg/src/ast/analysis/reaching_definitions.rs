use super::flow::Flow;
use super::framework::Analysis;
use crate::ast::{Edge, Game, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

pub struct ReachingDefinitions;

impl Analysis for ReachingDefinitions {
    type Domain = BTreeSet<(Id, Option<Edge<Id>>)>;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(program: &Game<Id>) -> Self::Domain {
        program
            .variables
            .iter()
            .map(|v| (v.identifier.clone(), None))
            .collect()
    }

    fn join(mut a: Self::Domain, b: Self::Domain) -> Self::Domain {
        a.extend(b);
        a
    }

    fn kill(input: Self::Domain, edge: &Edge<Id>) -> Self::Domain {
        match &edge.label.as_var_assignment() {
            Some((identifier, _)) if !edge.label.is_map_assignment() => input
                .into_iter()
                .filter(|(id, _)| id != *identifier)
                .collect(),
            _ => input,
        }
    }

    fn gen(mut input: Self::Domain, edge: &Edge<Id>) -> Self::Domain {
        if let Some((identifier, _)) = edge.label.as_var_assignment() {
            input.insert((identifier.clone(), Some(edge.clone())));
        }
        input
    }
}

impl Game<Id> {
    pub fn reaching_definitions(
        &self,
        debug: bool,
    ) -> BTreeMap<Node<Id>, <ReachingDefinitions as Analysis>::Domain> {
        let flow = Flow::new(self);
        let result = ReachingDefinitions.analyse(&flow, self);
        if debug {
            for (node, defs) in &result {
                println!("Node: {node}");
                println!("Definitions: ");
                for (id, edge) in defs {
                    edge.as_ref().map_or_else(
                        || println!("  {id} :  None"),
                        |edge| println!("  {id} :  \"{edge}\""),
                    );
                }
                println!();
            }
        }
        result
    }
}
