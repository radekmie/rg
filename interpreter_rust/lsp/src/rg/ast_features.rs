use crate::common::symbol::{Flag, Symbol};
use crate::rg::statement::Statement;
use rg::ast::Game;
use std::sync::Arc;
use utils::position::{Position, Span};
use utils::Identifier;

pub trait AstFeatures {
    fn find_stat(&self, predicate: impl Fn(&&dyn Statement) -> bool) -> Option<&dyn Statement>;
    fn stat_enclosing_position(&self, position: &Position) -> Option<&dyn Statement>;
    fn stat_enclosing_span(&self, span: &Span) -> Option<&dyn Statement>;
    fn stats(&self) -> Vec<&dyn Statement>;
}

impl AstFeatures for Game<Identifier> {
    fn find_stat(&self, predicate: impl Fn(&&dyn Statement) -> bool) -> Option<&dyn Statement> {
        stats(self).find(predicate)
    }

    fn stat_enclosing_position(&self, position: &Position) -> Option<&dyn Statement> {
        self.find_stat(|stat| stat.span().encloses_position(position))
    }

    fn stat_enclosing_span(&self, span: &Span) -> Option<&dyn Statement> {
        self.find_stat(|stat| stat.span().encloses_span(span))
    }

    fn stats(&self) -> Vec<&dyn Statement> {
        stats(self).collect()
    }
}

pub fn hover_signature(symbol: &Symbol) -> Option<String> {
    let type_ = &symbol.type_;
    match symbol.flag {
        Flag::Constant => Some(format!("const {}: {}", symbol.id, type_)),
        Flag::Function => type_
            .to_option()
            .map(|type_| format!("{}: {}", symbol.id, type_)),
        Flag::Member => Some(format!("{}: {}", symbol.id, type_)),
        Flag::Param => Some(format!("{}: {}", symbol.id, type_)),
        Flag::Type => Some(format!("type {}", symbol.id)),
        Flag::Variable => Some(format!("var {}: {}", symbol.id, type_)),
    }
}

fn stats(game: &Game<Identifier>) -> impl Iterator<Item = &dyn Statement> {
    macro_rules! mapper {
        () => {
            |x| x as &dyn Statement
        };
    }

    None.into_iter()
        .chain(game.typedefs.iter().map(mapper!()))
        .chain(game.constants.iter().map(mapper!()))
        .chain(game.variables.iter().map(mapper!()))
        .chain(game.edges.iter().map(Arc::as_ref).map(mapper!()))
        .chain(game.pragmas.iter().map(mapper!()))
}
