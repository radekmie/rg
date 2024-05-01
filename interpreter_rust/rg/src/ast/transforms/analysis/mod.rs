use std::sync::Arc;

use crate::ast::Game;

use self::{flow_graph::GameFlow, framework::Instance, reaching_definitions::ReachingDefinitions};

mod flow_graph;
mod framework;
mod reaching_definitions;

impl Game<Arc<str>> {
    pub fn reaching_definitions(&self) {
        let params = reaching_definitions::ReachingDefinitions;
        let flow = GameFlow::new(self);
        let result = ReachingDefinitions.analyse(&flow, params, self);
        dbg!(result);
    }
}
