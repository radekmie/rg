use crate::rg::{
    stat::Statement,
    symbol::{Flag, Symbol},
};
use rg::{ast::*, position::*};

pub trait AstFeatures {
    fn stats(&self) -> Vec<&dyn Statement>;
    fn find_stat(&self, predicate: impl Fn(&&dyn Statement) -> bool) -> Option<&dyn Statement>;
    fn stat_enclosing_span(&self, span: &Span) -> Option<&dyn Statement>;
    fn stat_enclosing_pos(&self, pos: &Position) -> Option<&dyn Statement>;
    fn symbol_type(&self, symbol: &Symbol) -> Option<String>;
}

impl AstFeatures for Game<Identifier> {
    fn stats(&self) -> Vec<&dyn Statement> {
        self.typedefs
            .iter()
            .map(|typedef| typedef as &dyn Statement)
            .chain(
                self.constants
                    .iter()
                    .map(|constant| constant as &dyn Statement),
            )
            .chain(
                self.variables
                    .iter()
                    .map(|variable| variable as &dyn Statement),
            )
            .chain(self.edges.iter().map(|edge| edge as &dyn Statement))
            .chain(self.pragmas.iter().map(|pragma| pragma as &dyn Statement))
            .collect()
    }

    fn find_stat(&self, predicate: impl Fn(&&dyn Statement) -> bool) -> Option<&dyn Statement> {
        self.typedefs
            .iter()
            .map(|typedef| typedef as &dyn Statement)
            .chain(
                self.constants
                    .iter()
                    .map(|constant| constant as &dyn Statement),
            )
            .chain(
                self.variables
                    .iter()
                    .map(|variable| variable as &dyn Statement),
            )
            .chain(self.edges.iter().map(|edge| edge as &dyn Statement))
            .chain(self.pragmas.iter().map(|pragma| pragma as &dyn Statement))
            .find(|stat| predicate(stat))
    }

    fn stat_enclosing_span(&self, span: &Span) -> Option<&dyn Statement> {
        self.find_stat(|stat| stat.span().encloses_span(span))
    }

    fn stat_enclosing_pos(&self, pos: &Position) -> Option<&dyn Statement> {
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
