use crate::common::symbol::{Flag, Symbol, Type as SymbolType};
use hrg::ast::{
    DomainDeclaration, DomainElement, DomainElementPattern, DomainValue, Expression, Function,
    FunctionDeclaration, Game, Pattern, Statement, Type, TypeDeclaration, VariableDeclaration,
};
use std::sync::Arc;
use utils::Identifier;

pub struct Symbols {
    pub symbols: Vec<Symbol>,
}

impl Symbols {
    fn add_from_statement(&mut self, stat: &Statement<Identifier>) {
        match stat {
            Statement::Assignment {
                accessors,
                expression,
                ..
            } => {
                accessors
                    .iter()
                    .for_each(|accessor| self.add_from_expression(accessor));
                self.add_from_expression(expression);
            }
            Statement::AssignmentAny { accessors, .. } => {
                accessors
                    .iter()
                    .for_each(|accessor| self.add_from_expression(accessor));
            }
            Statement::Branch { arms } => arms.iter().for_each(|arm| {
                arm.iter()
                    .for_each(|statement| self.add_from_statement(statement));
            }),
            Statement::Call { args, .. } => {
                args.iter().for_each(|arg| self.add_from_expression(arg));
            }
            Statement::If {
                expression,
                then,
                else_,
            } => {
                then.iter()
                    .for_each(|statement| self.add_from_statement(statement));
                self.add_from_expression(expression);
                else_
                    .iter()
                    .flatten()
                    .for_each(|statement| self.add_from_statement(statement));
            }
            Statement::Loop { body } | Statement::Repeat { body, .. } => body
                .iter()
                .for_each(|statement| self.add_from_statement(statement)),
            Statement::Tag { .. } => {}
            Statement::TagVariable { .. } => {}
            Statement::While { body, expression } => {
                body.iter()
                    .for_each(|statement| self.add_from_statement(statement));
                self.add_from_expression(expression);
            }
        }
    }

    fn add_from_expression(&mut self, expr: &Expression<Identifier>) {
        match expr {
            Expression::Access { lhs, rhs } | Expression::BinExpr { lhs, rhs, .. } => {
                self.add_from_expression(lhs);
                self.add_from_expression(rhs);
            }
            Expression::Call { expression, args } => {
                self.add_from_expression(expression);
                args.iter().for_each(|arg| self.add_from_expression(arg));
            }
            Expression::Constructor { args, .. } => {
                args.iter().for_each(|arg| self.add_from_expression(arg));
            }
            Expression::If { cond, then, else_ } => {
                self.add_from_expression(cond);
                self.add_from_expression(then);
                self.add_from_expression(else_);
            }
            Expression::Literal { .. } => {}
            Expression::Map {
                default_value,
                parts,
            } => {
                if let Some(default_value) = default_value {
                    self.add_from_expression(default_value);
                }

                for part in parts {
                    self.add_from_pattern(&part.pattern);
                    self.add_from_expression(&part.expression);
                }
            }
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
                for arg in args {
                    if let DomainElementPattern::Variable { identifier } = arg {
                        if let Some(symbol) = untyped(identifier, Flag::Param) {
                            self.symbols.push(symbol);
                        }
                    }
                }
                values.iter().for_each(|value| match value {
                    DomainValue::Range { .. } => {}
                    DomainValue::Set { elements, .. } => {
                        for id in elements {
                            if let Some(symbol) = untyped(id, Flag::Member) {
                                self.symbols.push(symbol);
                            }
                        }
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
        if let Some(default_value) = &variable.default_value.as_ref() {
            self.add_from_expression(default_value);
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
        for case in &func.cases {
            for arg in &case.args {
                self.add_from_pattern(arg);
            }
            self.add_from_expression(&case.body);
        }
    }

    fn add_from_pattern(&mut self, pattern: &Pattern<Identifier>) {
        match pattern {
            Pattern::Constructor { args, .. } => {
                args.iter().for_each(|arg| self.add_from_pattern(arg));
            }
            Pattern::Variable { identifier } => {
                if let Some(symbol) = untyped(identifier, Flag::Param) {
                    self.symbols.push(symbol);
                }
            }
            Pattern::Literal { .. } | Pattern::Wildcard => {}
        }
    }

    pub fn from_game(game: &Game<Identifier>) -> Vec<Symbol> {
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
    Symbol::from_id(identifier, flag, SymbolType::Hrg(type_))
}

fn untyped(identifier: &Identifier, flag: Flag) -> Option<Symbol> {
    Symbol::from_id(identifier, flag, SymbolType::NoType)
}
