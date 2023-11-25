use crate::rg::{
    stat::Stat,
    symbol::{Flag, Symbol},
};
use rg::{ast::*, position::*};

pub trait AstFeatures {
    fn enclosing_stat(&self, span: &Span) -> Option<Stat>;
    fn enclosing_stat_pos(&self, pos: &Position) -> Option<Stat>;
    fn symbol_type(&self, symbol: &Symbol) -> Option<String>;
}

impl AstFeatures for Game<Identifier> {
    fn enclosing_stat(&self, span: &Span) -> Option<Stat> {
        Stat::from_game(self)
            .into_iter()
            .find(|stat| stat.span().encloses_span(span))
    }

    fn enclosing_stat_pos(&self, pos: &Position) -> Option<Stat> {
        Stat::from_game(self)
            .into_iter()
            .find(|stat| stat.span().encloses_pos(pos))
    }

    fn symbol_type(&self, symbol: &Symbol) -> Option<String> {
        self.enclosing_stat(&symbol.span())
            .and_then(|stat| match stat {
                Stat::Constant(Constant { type_, .. }) => Some(format!("{}", type_)),
                Stat::Variable(Variable { type_, .. }) => Some(format!("{}", type_)),
                Stat::Typedef(Typedef { identifier, .. }) => Some(format!("{}", identifier)),
                Stat::Edge(Edge { lhs, rhs, .. }) => {
                    let left = edge_name_label(lhs, symbol);
                    left.or(edge_name_label(rhs, symbol))
                }
                _ => None,
            })
    }
}

pub fn hover_signature(game: &Game<Identifier>, symbol: &Symbol) -> Option<String> {
    let type_ = game.symbol_type(symbol)?;
    match symbol.flag {
        Flag::Constant => Some(format!("const {}: {}", symbol.id, type_)),
        Flag::Variable => Some(format!("var {}: {}", symbol.id, type_)),
        Flag::Type => Some(format!("type {}", symbol.id)),
        Flag::Member => Some(format!("{}: {}", symbol.id, type_)),
        Flag::Param => Some(format!("{}: {}", symbol.id, type_)),
        Flag::Edge => None,
    }
}

fn edge_name_label(edge_name: &EdgeName<Identifier>, symbol: &Symbol) -> Option<String> {
    match edge_name.parts.as_slice() {
        [_, bindings @ ..] => bindings.iter().find_map(|binding| match binding {
            EdgeNamePart::Binding { type_, .. } => {
                if binding.span().encloses_span(&symbol.pos) {
                    Some(format!("{}", type_))
                } else {
                    None
                }
            }
            _ => None,
        }),
        _ => None,
    }
}
