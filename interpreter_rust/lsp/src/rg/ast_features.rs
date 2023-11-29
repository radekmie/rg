use crate::rg::{
    stat::Stat,
    symbol::{Flag, Symbol},
};
use rg::{ast::*, position::*};

pub trait AstFeatures {
    fn stats(&self) -> Vec<&dyn Stat>;
    fn find_stat(&self, predicate: impl Fn(&&dyn Stat) -> bool) -> Option<&dyn Stat>;
    fn stat_enclosing_span(&self, span: &Span) -> Option<&dyn Stat>;
    fn stat_enclosing_pos(&self, pos: &Position) -> Option<&dyn Stat>;
    fn symbol_type(&self, symbol: &Symbol) -> Option<String>;
}

impl AstFeatures for Game<Identifier> {
    fn stats(&self) -> Vec<&dyn Stat> {
        self.typedefs
            .iter()
            .map(|typedef| typedef as &dyn Stat)
            .chain(self.constants.iter().map(|constant| constant as &dyn Stat))
            .chain(self.variables.iter().map(|variable| variable as &dyn Stat))
            .chain(self.edges.iter().map(|edge| edge as &dyn Stat))
            .chain(self.pragmas.iter().map(|pragma| pragma as &dyn Stat))
            .collect()
    }

    fn find_stat(&self, predicate: impl Fn(&&dyn Stat) -> bool) -> Option<&dyn Stat> {
        self.typedefs
            .iter()
            .map(|typedef| typedef as &dyn Stat)
            .chain(self.constants.iter().map(|constant| constant as &dyn Stat))
            .chain(self.variables.iter().map(|variable| variable as &dyn Stat))
            .chain(self.edges.iter().map(|edge| edge as &dyn Stat))
            .chain(self.pragmas.iter().map(|pragma| pragma as &dyn Stat))
            .find(|stat| predicate(stat))
    }

    fn stat_enclosing_span(&self, span: &Span) -> Option<&dyn Stat> {
        self.find_stat(|stat| stat.span().encloses_span(span))
    }

    fn stat_enclosing_pos(&self, pos: &Position) -> Option<&dyn Stat> {
        self.find_stat(|stat| stat.span().encloses_pos(pos))
    }

    fn symbol_type(&self, symbol: &Symbol) -> Option<String> {
        self.stat_enclosing_span(&symbol.span())
            .and_then(|stat| stat.symbol_type(symbol))
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
