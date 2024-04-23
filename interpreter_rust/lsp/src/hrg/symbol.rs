use std::sync::Arc;

use hrg::ast::{
    DomainDeclaration, DomainElement, DomainValue, Function, FunctionDeclaration, GameDeclaration,
    Statement, Type, TypeDeclaration, VariableDeclaration,
};
use utils::Identifier;

use crate::common::{
    self,
    symbol::{Flag, Symbol},
};

pub struct Symbols {
    pub symbols: Vec<Symbol>,
}

impl Symbols {
    fn add_from_statement(&mut self, stat: &Statement<Identifier>) {
        match stat {
            Statement::Forall {
                identifier,
                body,
                type_,
            } => {
                if let Some(symbol) = typed(identifier, Flag::Param, (*type_).clone()) {
                    self.symbols.push(symbol);
                }
                body.iter()
                    .for_each(|statement| self.add_from_statement(statement));
            }
            Statement::Branch { arms } => arms.iter().for_each(|arm| {
                arm.iter()
                    .for_each(|statement| self.add_from_statement(statement))
            }),
            Statement::Loop { body }
            | Statement::When { body, .. }
            | Statement::While { body, .. } => body
                .iter()
                .for_each(|statement| self.add_from_statement(statement)),

            _ => {}
        }
    }

    fn add_from_type_decl(&mut self, type_: &TypeDeclaration<Identifier>) {
        if let Some(symbol) = untyped(&type_.identifier, Flag::Type) {
            self.symbols.push(symbol);
        }
    }

    fn add_from_domain_decl(&mut self, domain: &DomainDeclaration<Identifier>) {
        if let Some(symbol) = untyped(&domain.identifier, Flag::Type) {
            self.symbols.push(symbol);
        }
        let type_ = Arc::new(Type::new(domain.identifier.clone()));
        domain.elements.iter().for_each(|element| match element {
            DomainElement::Generator {
                identifier,
                args,
                values,
            } => {
                if let Some(symbol) = typed(identifier, Flag::Type, type_.clone()) {
                    self.symbols.push(symbol);
                }
                args.iter().for_each(|arg| {
                    if let Some(symbol) = untyped(arg, Flag::Param) {
                        self.symbols.push(symbol);
                    }
                });
                values.iter().for_each(|value| match value {
                    DomainValue::Range { .. } => {}
                    DomainValue::Set { elements, .. } => {
                        elements.iter().for_each(|id| {
                            if let Some(symbol) = untyped(id, Flag::Member) {
                                self.symbols.push(symbol);
                            }
                        });
                    }
                });
            }
            DomainElement::Literal { identifier } => {
                if let Some(symbol) = typed(identifier, Flag::Member, type_.clone()) {
                    self.symbols.push(symbol);
                }
            }
        });
    }

    fn add_from_variable_decl(&mut self, variable: &VariableDeclaration<Identifier>) {
        if let Some(symbol) = typed(&variable.identifier, Flag::Variable, variable.type_.clone()) {
            self.symbols.push(symbol);
        }
    }

    fn add_from_function(&mut self, func: &Function<Identifier>) {
        if let Some(symbol) = untyped(&func.name, Flag::Function) {
            self.symbols.push(symbol);
        }
        func.args.iter().for_each(|arg| {
            if let Some(symbol) = typed(&arg.identifier, Flag::Param, arg.type_.clone()) {
                self.symbols.push(symbol);
            }
        });
        func.body
            .iter()
            .for_each(|statement| self.add_from_statement(statement));
    }

    fn add_from_function_decl(&mut self, func: &FunctionDeclaration<Identifier>) {
        if let Some(symbol) = typed(&func.identifier, Flag::Function, func.type_.clone()) {
            self.symbols.push(symbol);
        }
    }

    pub fn from_game(game: &GameDeclaration<Identifier>) -> Vec<Symbol> {
        let mut symbols = Self { symbols: vec![] };
        game.types
            .iter()
            .for_each(|type_| symbols.add_from_type_decl(type_));
        game.domains
            .iter()
            .for_each(|domain| symbols.add_from_domain_decl(domain));
        game.variables
            .iter()
            .for_each(|variable| symbols.add_from_variable_decl(variable));
        game.automaton
            .iter()
            .for_each(|func| symbols.add_from_function(func));
        game.functions
            .iter()
            .for_each(|func| symbols.add_from_function_decl(func));
        symbols.symbols
    }
}

fn typed(identifier: &Identifier, flag: Flag, type_: Arc<Type<Identifier>>) -> Option<Symbol> {
    Symbol::from_id(identifier, flag, common::symbol::Type::Hrg(type_))
}

fn untyped(identifier: &Identifier, flag: Flag) -> Option<Symbol> {
    Symbol::from_id(identifier, flag, common::symbol::Type::NoType)
}
