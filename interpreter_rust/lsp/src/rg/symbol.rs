use crate::common;
use crate::common::symbol::{defined, Flag, Symbol};
use rg::ast::{Game, Type};
use std::sync::Arc;
use utils::Identifier;

pub struct Symbols {
    symbols: Vec<Symbol>,
}

impl Symbols {
    fn add_from_type(&mut self, type_: &Type<Identifier>, sym_type: Arc<Type<Identifier>>) {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                self.add_from_type(lhs, sym_type.clone());
                self.add_from_type(rhs, sym_type);
            }
            Type::TypeReference { .. } => (),
            Type::Set { identifiers, .. } => {
                for identifier in identifiers {
                    if let Some(symbol) = typed(identifier, Flag::Member, sym_type.clone()) {
                        self.symbols.push(symbol);
                    }
                }
            }
        }
    }

    fn add_if_not_defined(&mut self, symbol: Option<Symbol>) {
        if let Some(symbol) = symbol {
            if defined(&self.symbols, &symbol.id, &symbol.flag).is_none() {
                self.symbols.push(symbol);
            }
        }
    }

    pub fn from_game(game: &Game<Identifier>) -> Vec<Symbol> {
        let mut symbols: Self = Self { symbols: vec![] };
        game.constants.iter().for_each(|constant| {
            let id = &constant.identifier;
            if let Some(symbol) = typed(id, Flag::Constant, constant.type_.clone()) {
                symbols.symbols.push(symbol);
            }
        });

        game.variables.iter().for_each(|variable| {
            let id = &variable.identifier;
            if let Some(symbol) = typed(id, Flag::Variable, variable.type_.clone()) {
                symbols.symbols.push(symbol);
            }
        });
        game.typedefs.iter().for_each(|typedef| {
            let id = &typedef.identifier;
            if let Some(symbol) = untyped(id, Flag::Type) {
                symbols.symbols.push(symbol);
            }
            let sym_type = Arc::new(Type::new(id.clone()));
            symbols.add_from_type(&typedef.type_, sym_type);
        });
        game.edges.iter().for_each(|edge| {
            symbols.add_if_not_defined(untyped(&edge.lhs.identifier, Flag::Function));
            symbols.add_if_not_defined(untyped(&edge.rhs.identifier, Flag::Function));
        });
        symbols.symbols
    }
}

fn typed(identifier: &Identifier, flag: Flag, type_: Arc<Type<Identifier>>) -> Option<Symbol> {
    Symbol::from_id(identifier, flag, common::symbol::Type::Rg(type_))
}

fn untyped(identifier: &Identifier, flag: Flag) -> Option<Symbol> {
    Symbol::from_id(identifier, flag, common::symbol::Type::NoType)
}
