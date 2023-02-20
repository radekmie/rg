use rg::ast::{EdgeDeclaration, EdgeLabel, Error, GameDeclaration};
use std::rc::Rc;

pub fn skip_self_assignments<Id: PartialEq>(
    mut game_declaration: GameDeclaration<Id>,
) -> Result<GameDeclaration<Id>, Error<Id>> {
    for edge in &mut game_declaration.edges {
        if edge.label.is_self_assignment() {
            *edge = Rc::new(EdgeDeclaration {
                label: Rc::new(EdgeLabel::Skip),
                lhs: edge.lhs.clone(),
                rhs: edge.rhs.clone(),
            });
        }
    }

    Ok(game_declaration)
}
