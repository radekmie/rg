use rg::ast::GameDeclaration;
use std::rc::Rc;

pub fn expand_generator_nodes(game_declaration: &mut GameDeclaration<String>) {
    game_declaration.edges = game_declaration
        .edges
        .iter()
        .flat_map(|edge| {
            game_declaration
                .create_mappings(edge.bindings())
                .into_iter()
                .map(|mapping| Rc::new(edge.substitute_bindings(&mapping)))
        })
        .collect()
}
