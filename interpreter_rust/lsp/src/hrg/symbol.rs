use hrg::ast::{
    DomainDeclaration, DomainElement, DomainValue, Function, FunctionDeclaration, GameDeclaration,
    Statement, TypeDeclaration, VariableDeclaration,
};
use utils::Identifier;

use crate::common::symbol::{Flag, Symbol};

pub struct Symbols {
    pub symbols: Vec<Symbol>,
}

impl Symbols {
    fn add_from_statement(&mut self, stat: &Statement<Identifier>) {
        match stat {
            Statement::Forall {
                identifier, body, ..
            } => {
                if let Some(symbol) = Symbol::from_id(identifier, Flag::Param) {
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
        if let Some(symbol) = Symbol::from_id(&type_.identifier, Flag::Type) {
            self.symbols.push(symbol);
        }
    }

    fn add_from_domain_decl(&mut self, domain: &DomainDeclaration<Identifier>) {
        if let Some(symbol) = Symbol::from_id(&domain.identifier, Flag::Type) {
            self.symbols.push(symbol);
        }
        domain.elements.iter().for_each(|element| match element {
            DomainElement::Generator {
                identifier,
                args,
                values,
            } => {
                if let Some(symbol) = Symbol::from_id(identifier, Flag::Type) {
                    self.symbols.push(symbol);
                }
                args.iter().for_each(|arg| {
                    if let Some(symbol) = Symbol::from_id(arg, Flag::Param) {
                        self.symbols.push(symbol);
                    }
                });
                values.iter().for_each(|value| match value {
                    DomainValue::Range { .. } => {}
                    DomainValue::Set { values, .. } => {
                        values.iter().for_each(|id| {
                            if let Some(symbol) = Symbol::from_id(id, Flag::Member) {
                                self.symbols.push(symbol);
                            }
                        });
                    }
                });
            }
            DomainElement::Literal { identifier } => {
                if let Some(symbol) = Symbol::from_id(identifier, Flag::Member) {
                    self.symbols.push(symbol);
                }
            }
        });
    }

    fn add_from_variable_decl(&mut self, variable: &VariableDeclaration<Identifier>) {
        if let Some(symbol) = Symbol::from_id(&variable.identifier, Flag::Variable) {
            self.symbols.push(symbol);
        }
    }

    fn add_from_function(&mut self, func: &Function<Identifier>) {
        if let Some(symbol) = Symbol::from_id(&func.name, Flag::Edge) {
            self.symbols.push(symbol);
        }
        func.args.iter().for_each(|arg| {
            if let Some(symbol) = Symbol::from_id(&arg.identifier, Flag::Param) {
                self.symbols.push(symbol);
            }
        });
        func.body
            .iter()
            .for_each(|statement| self.add_from_statement(statement));
    }

    fn add_from_function_decl(&mut self, func: &FunctionDeclaration<Identifier>) {
        if let Some(symbol) = Symbol::from_id(&func.identifier, Flag::Edge) {
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
