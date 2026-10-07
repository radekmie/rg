use super::symbol::Symbols;
use crate::common::symbol::{make_builtin, Flag};
use crate::common::symbol_table::{Occurrence, SymbolTable, SymbolTableBuilder};
use rg::ast::{Edge, Expression, Game, Label, Node, Type, Value, ValueEntry};
use utils::position::Positioned;
use utils::{Identifier, ParserError};

fn add_from_type(table: &mut SymbolTableBuilder, type_: &Type<Identifier>) {
    match type_ {
        Type::Arrow { lhs, rhs } => {
            add_from_type(table, lhs);
            add_from_type(table, rhs);
        }
        Type::TypeReference { identifier } => {
            table.add_occ_with_flag(identifier, Flag::Type);
        }
        Type::Set { identifiers, .. } => {
            for identifier in identifiers {
                table.add_occ_with_flag(identifier, Flag::Member);
            }
        }
    }
}

fn add_from_edge(table: &mut SymbolTableBuilder, edge: &Edge<Identifier>) {
    add_from_node(table, &edge.lhs);
    add_from_node(table, &edge.rhs);
    add_from_edge_label(table, &edge.label);
}

fn add(table: &mut SymbolTableBuilder, identifier: &Identifier, create_error: bool) {
    if !identifier.is_none() && !identifier.is_numeric() {
        let span = identifier.span();
        let sym_idx = table.find_symbol(identifier, &None, &None);
        if sym_idx.is_some() {
            table.occurrences.push(Occurrence::new(span, sym_idx));
        } else if create_error {
            table
                .errors
                .push(ParserError::new_unknown_identifier(identifier));
        }
    }
}

fn add_from_edge_label(table: &mut SymbolTableBuilder, label: &Label<Identifier>) {
    match label {
        Label::Assignment { lhs, rhs } => {
            add_from_expression(table, lhs);
            add_from_expression(table, rhs);
        }
        Label::AssignmentAny { lhs, rhs } => {
            add_from_expression(table, lhs);
            add_from_type(table, rhs);
        }
        Label::Comparison { lhs, rhs, .. } => {
            add_from_expression(table, lhs);
            add_from_expression(table, rhs);
        }
        Label::Skip { .. } => (),
        Label::Tag { symbol } => add(table, symbol, false),
        Label::TagVariable { identifier } => add(table, identifier, false),
        Label::Reachability { lhs, rhs, .. } => {
            add_from_node(table, lhs);
            add_from_node(table, rhs);
        }
    }
}

fn add_from_expression(table: &mut SymbolTableBuilder, expr: &Expression<Identifier>) {
    match expr {
        Expression::Reference { identifier } => {
            add(table, identifier, true);
        }
        Expression::Access { lhs, rhs, .. } => {
            add_from_expression(table, lhs);
            add_from_expression(table, rhs);
        }
        Expression::Cast { lhs, rhs, .. } => {
            add_from_type(table, lhs);
            add_from_expression(table, rhs);
        }
    }
}

fn add_from_node(table: &mut SymbolTableBuilder, node: &Node<Identifier>) {
    table.add_occ_with_flag(&node.identifier, Flag::Function);
}

fn add_from_value(table: &mut SymbolTableBuilder, value: &Value<Identifier>) {
    match value {
        Value::Element { identifier } => {
            table.add_occ(identifier);
        }
        Value::Map { entries, .. } => {
            for entry in entries {
                add_from_value_entry(table, entry);
            }
        }
    }
}

fn add_from_value_entry(table: &mut SymbolTableBuilder, entry: &ValueEntry<Identifier>) {
    if let Some(identifier) = entry.identifier.as_ref() {
        table.add_occ(identifier);
    }
    add_from_value(table, &entry.value);
}

pub fn table_builder_from_game(game: &Game<Identifier>) -> SymbolTableBuilder {
    let mut table: SymbolTableBuilder = SymbolTableBuilder {
        symbols: Symbols::from_game(game),
        occurrences: Vec::new(),
        errors: Vec::new(),
    };
    add_builtin_symbols(&mut table);
    game.constants.iter().for_each(|constant| {
        table.add_occ_with_flag(&constant.identifier, Flag::Constant);
        add_from_type(&mut table, &constant.type_);
        add_from_value(&mut table, &constant.value);
    });
    game.variables.iter().for_each(|variable| {
        table.add_occ_with_flag(&variable.identifier, Flag::Variable);
        add_from_type(&mut table, &variable.type_);
        add_from_value(&mut table, &variable.default_value);
    });
    game.typedefs.iter().for_each(|typedef| {
        table.add_occ_with_flag(&typedef.identifier, Flag::Type);
        add_from_type(&mut table, &typedef.type_);
    });
    game.edges.iter().for_each(|edge| {
        add_from_edge(&mut table, edge);
    });
    game.pragmas.iter().for_each(|pragma| {
        for expression in pragma.expressions().into_iter().flatten() {
            add_from_expression(&mut table, expression);
        }
        for node in pragma.nodes().into_iter().flatten() {
            add_from_node(&mut table, node);
        }
        for (identifier, type_) in pragma.variables().into_iter().flatten() {
            add(&mut table, identifier, true);
            add_from_type(&mut table, type_);
        }
    });
    table
}

pub const BUILTIN_SYMBOLS: [(&str, Flag); 11] = [
    ("0", Flag::Member),
    ("1", Flag::Member),
    ("Bool", Flag::Type),
    ("Goals", Flag::Type),
    ("PlayerOrSystem", Flag::Type),
    ("Visibility", Flag::Type),
    ("goals", Flag::Variable),
    ("keeper", Flag::Variable),
    ("player", Flag::Variable),
    ("random", Flag::Variable),
    ("visible", Flag::Variable),
];

fn add_builtin_symbols(table: &mut SymbolTableBuilder) {
    for (symbol, flag) in BUILTIN_SYMBOLS {
        if !table.is_defined(symbol) {
            table.symbols.push(make_builtin(symbol, flag));
        }
    }
}

pub fn from_game(game: &Game<Identifier>) -> (SymbolTable, Vec<ParserError>) {
    let table = table_builder_from_game(game);
    (
        SymbolTable {
            symbols: table.symbols,
            occurrences: table.occurrences,
        },
        table.errors,
    )
}
