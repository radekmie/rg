use rg::ast::{Error, GameDeclaration};
use std::rc::Rc;

pub fn expand_generator_nodes(
    mut game_declaration: GameDeclaration<String>,
) -> Result<GameDeclaration<String>, Error<String>> {
    game_declaration.edges = game_declaration
        .edges
        .iter()
        .map(|edge| {
            game_declaration
                .create_mappings(edge.bindings())
                .map(|mappings| {
                    mappings
                        .into_iter()
                        .map(|mapping| Rc::new(edge.substitute_bindings(&mapping)))
                })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(game_declaration)
}
