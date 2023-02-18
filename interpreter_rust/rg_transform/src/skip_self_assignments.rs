use rg::ast::{EdgeDeclaration, EdgeLabel, GameDeclaration};
use std::rc::Rc;

pub fn skip_self_assignments<Id: PartialEq>(game_declaration: &mut GameDeclaration<Id>) {
    for edge in &mut game_declaration.edges {
        if edge.label.is_self_assignment() {
            *edge = Rc::new(EdgeDeclaration {
                label: Rc::new(EdgeLabel::Skip),
                lhs: edge.lhs.clone(),
                rhs: edge.rhs.clone(),
            });
        }
    }
}
