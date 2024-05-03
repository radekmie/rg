use self::flow::Flow;
use self::framework::Instance;
use self::reaching_definitions::ReachingDefinitions;
use crate::ast::{Edge, Game, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod flow;
mod framework;
mod reaching_definitions;

impl Game<Arc<str>> {
    pub fn reaching_definitions(
        &self,
    ) -> BTreeMap<Node<Arc<str>>, BTreeSet<(Arc<str>, Option<Edge<Arc<str>>>)>> {
        let params = reaching_definitions::ReachingDefinitions;
        let flow = Flow::new(self);

        // result.iter().for_each(|(node, defs)| {
        //     println!("Node: {node}");
        //     println!("Definitions: ");
        //     defs.iter().for_each(|(id, edge)| {
        //         if let Some(edge) = edge {
        //             println!("  {id} :  \"{edge}\"");
        //         } else {
        //             println!("  {id} : None");
        //         }
        //     });
        //     println!("");
        // })
        ReachingDefinitions.analyse(&flow, params, self)
    }
}
