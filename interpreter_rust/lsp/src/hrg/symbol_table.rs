use crate::common::symbol::{make_builtin, Flag};
use crate::common::symbol_table::{SymbolTable, SymbolTableBuilder};
use hrg::ast::{
    DomainDeclaration, DomainElement, DomainElementPattern, DomainValue, Expression, Function,
    FunctionArg, FunctionCase, FunctionDeclaration, Game, Pattern, Statement, Type,
    TypeDeclaration, VariableDeclaration,
};
use utils::position::Positioned;
use utils::{Error, Identifier};

use super::symbol::Symbols;

fn table_builder_from_game(game: &Game<Identifier>) -> SymbolTableBuilder {
    let mut table = SymbolTableBuilder {
        errors: vec![],
        occurrences: vec![],
        symbols: Symbols::from_game(game),
    };
    add_builtin_symbols(&mut table);
    table
        .symbols
        .sort_by(|a, b| a.span().start.cmp(&b.span().start));
    game.automaton
        .iter()
        .for_each(|function| add_from_function(&mut table, function));
    game.domains
        .iter()
        .for_each(|domain| add_from_domain_declaration(&mut table, domain));
    game.functions
        .iter()
        .for_each(|func| add_from_function_declaration(&mut table, func));
    game.variables
        .iter()
        .for_each(|variable| add_from_variable_declaration(&mut table, variable));
    game.types
        .iter()
        .for_each(|type_| add_from_type_declaration(&mut table, type_));
    table
}

fn add_from_statement(table: &mut SymbolTableBuilder, stat: &Statement<Identifier>) {
    match stat {
        Statement::Assignment {
            identifier,
            accessors,
            expression,
        } => {
            table.add_occ(identifier);
            for accessor in accessors {
                add_from_expression(table, accessor);
            }
            add_from_expression(table, expression);
        }
        Statement::Branch { arms } => {
            for arm in arms {
                for statement in arm {
                    add_from_statement(table, statement);
                }
            }
        }
        Statement::Call { identifier, args } => {
            table.add_occ(identifier);
            for arg in args {
                add_from_expression(table, arg);
            }
        }
        Statement::Forall {
            identifier,
            type_,
            body,
        } => {
            table.add_occ(identifier);
            add_from_type(table, type_);
            for statement in body {
                add_from_statement(table, statement);
            }
        }
        Statement::If {
            expression,
            then,
            else_,
        } => {
            add_from_expression(table, expression);
            for statement in then {
                add_from_statement(table, statement);
            }
            for statement in else_.iter().flatten() {
                add_from_statement(table, statement);
            }
        }
        Statement::Loop { body } => {
            for statement in body {
                add_from_statement(table, statement);
            }
        }
        Statement::Tag { symbol } => {
            if !symbol.is_none() && !symbol.is_numeric() {
                let occ = table.occ_from_id(symbol);
                if occ.symbol.is_some() {
                    table.occurrences.push(occ);
                }
            }
        }
        Statement::While { expression, body } => {
            add_from_expression(table, expression);
            for statement in body {
                add_from_statement(table, statement);
            }
        }
    }
}

fn add_from_function(table: &mut SymbolTableBuilder, func: &Function<Identifier>) {
    table.add_occ(&func.name);
    for arg in &func.args {
        add_from_function_arg(table, arg);
    }
    for statement in &func.body {
        add_from_statement(table, statement);
    }
}

fn add_from_function_arg(table: &mut SymbolTableBuilder, arg: &FunctionArg<Identifier>) {
    table.add_occ(&arg.identifier);
    add_from_type(table, &arg.type_);
}

fn add_from_function_declaration(
    table: &mut SymbolTableBuilder,
    func: &FunctionDeclaration<Identifier>,
) {
    table.add_occ(&func.identifier);
    add_from_type(table, &func.type_);
    for case in &func.cases {
        add_from_function_case(table, case);
    }
}

fn add_from_function_case(table: &mut SymbolTableBuilder, case: &FunctionCase<Identifier>) {
    table.add_occ(&case.identifier);
    for arg in &case.args {
        add_from_pattern(table, arg);
    }
    add_from_expression(table, &case.body);
}

fn add_from_domain_declaration(
    table: &mut SymbolTableBuilder,
    domain: &DomainDeclaration<Identifier>,
) {
    table.add_occ(&domain.identifier);
    for element in &domain.elements {
        add_from_domain_element(table, element);
    }
}

fn add_from_domain_element(table: &mut SymbolTableBuilder, element: &DomainElement<Identifier>) {
    match element {
        DomainElement::Generator {
            identifier,
            args,
            values,
        } => {
            table.add_occ(identifier);
            for arg in args {
                if let DomainElementPattern::Variable { identifier } = arg {
                    table.add_occ(identifier);
                }
            }
            for value in values {
                add_from_domain_value(table, value);
            }
        }
        DomainElement::Literal { identifier } => table.add_occ(identifier),
    }
}

fn add_from_domain_value(table: &mut SymbolTableBuilder, domain: &DomainValue<Identifier>) {
    match domain {
        DomainValue::Range { identifier, .. } => {
            table.add_occ(identifier);
        }
        DomainValue::Set { identifier, .. } => {
            table.add_occ(identifier);
        }
    }
}

fn add_from_expression(table: &mut SymbolTableBuilder, expr: &Expression<Identifier>) {
    match expr {
        Expression::Access { lhs, rhs } => {
            add_from_expression(table, lhs);
            add_from_expression(table, rhs);
        }
        Expression::BinExpr { lhs, rhs, .. } => {
            add_from_expression(table, lhs);
            add_from_expression(table, rhs);
        }
        Expression::Call { expression, args } => {
            add_from_expression(table, expression);
            for arg in args {
                add_from_expression(table, arg);
            }
        }
        Expression::Constructor { identifier, args } => {
            table.add_occ(identifier);
            for arg in args {
                add_from_expression(table, arg);
            }
        }
        Expression::If { cond, then, else_ } => {
            add_from_expression(table, cond);
            add_from_expression(table, then);
            add_from_expression(table, else_);
        }
        Expression::Literal { identifier } => {
            table.add_occ(identifier);
        }
        Expression::Map { parts } => {
            for part in parts {
                add_from_pattern(table, &part.pattern);
                add_from_expression(table, &part.expression);
                for domain in &part.domains {
                    add_from_domain_value(table, domain);
                }
            }
        }
    }
}

fn add_from_pattern(table: &mut SymbolTableBuilder, pattern: &Pattern<Identifier>) {
    match pattern {
        Pattern::Constructor { identifier, args } => {
            table.add_occ(identifier);
            for arg in args {
                add_from_pattern(table, arg);
            }
        }
        Pattern::Literal { identifier } | Pattern::Variable { identifier } => {
            table.add_occ(identifier);
        }
        Pattern::Wildcard => {}
    }
}

fn add_from_type_declaration(table: &mut SymbolTableBuilder, type_: &TypeDeclaration<Identifier>) {
    table.add_occ(&type_.identifier);
    add_from_type(table, &type_.type_);
}

fn add_from_type(table: &mut SymbolTableBuilder, type_: &Type<Identifier>) {
    match type_ {
        Type::Function { lhs, rhs } => {
            add_from_type(table, lhs);
            add_from_type(table, rhs);
        }
        Type::Name { identifier } => {
            table.add_occ_with_flag(identifier, Flag::Type);
        }
    }
}

fn add_from_variable_declaration(
    table: &mut SymbolTableBuilder,
    variable: &VariableDeclaration<Identifier>,
) {
    table.add_occ(&variable.identifier);
    add_from_type(table, &variable.type_);
    if let Some(default_value) = variable.default_value.as_ref() {
        add_from_expression(table, default_value);
    }
}

const BUILDIN_SYMBOLS: [(&str, Flag); 10] = [
    ("keeper", Flag::Variable),
    ("goals", Flag::Variable),
    ("player", Flag::Variable),
    ("not", Flag::Function),
    ("check", Flag::Function),
    ("reachable", Flag::Function),
    ("end", Flag::Function),
    ("return", Flag::Function),
    ("continue", Flag::Function),
    ("break", Flag::Function),
];

fn add_builtin_symbols(table: &mut SymbolTableBuilder) {
    for (symbol, flag) in BUILDIN_SYMBOLS {
        if !table.is_defined(symbol) {
            table.symbols.push(make_builtin(symbol, flag));
        }
    }
}

pub fn from_game(game: &Game<Identifier>) -> (SymbolTable, Vec<Error>) {
    let table = table_builder_from_game(game);
    (
        SymbolTable {
            symbols: table.symbols,
            occurrences: table.occurrences,
        },
        table.errors,
    )
}
