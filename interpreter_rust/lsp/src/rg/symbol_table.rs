use super::symbol::Symbols;
use crate::common::symbol::{make_builtin, Flag};
use crate::common::symbol_table::{Occurrence, SymbolTable, SymbolTableBuilder};
use rg::ast::{Edge, Expression, Game, Label, Node, Type, Value, ValueEntry};
use utils::position::Positioned;
use utils::{Error, Identifier};

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
    // TODO: Clean these "owner"
    let left_owner = add_from_edge_name(table, &edge.lhs);
    let right_owner = add_from_edge_name(table, &edge.rhs);
    let owner = left_owner.or(right_owner);
    add_from_edge_label(table, &edge.label, &owner);
}

fn add_maybe_edge_param(
    table: &mut SymbolTableBuilder,
    identifier: &Identifier,
    owner: &Option<usize>,
    create_error: bool,
) {
    if !identifier.is_none() && !identifier.is_numeric() {
        let span = identifier.span();
        let sym_idx = table
            .find_symbol(identifier, &Some(Flag::Param), owner)
            .or_else(|| table.find_symbol(identifier, &None, &None));
        if sym_idx.is_some() {
            table.occurrences.push(Occurrence::new(span, sym_idx));
        } else if create_error {
            table.errors.push(Error::symbol_table_error(
                &identifier.identifier,
                &identifier.span,
            ));
        }
    }
}

fn add_from_edge_label(
    table: &mut SymbolTableBuilder,
    label: &Label<Identifier>,
    owner: &Option<usize>,
) {
    match label {
        Label::Assignment { lhs, rhs } => {
            add_from_expression(table, lhs, owner);
            add_from_expression(table, rhs, owner);
        }
        Label::AssignmentAny { lhs, rhs } => {
            add_from_expression(table, lhs, owner);
            add_from_type(table, rhs);
        }
        Label::Comparison { lhs, rhs, .. } => {
            add_from_expression(table, lhs, owner);
            add_from_expression(table, rhs, owner);
        }
        Label::Skip { .. } => (),
        Label::Tag { symbol } => add_maybe_edge_param(table, symbol, owner, false),
        Label::TagVariable { symbol } => add_maybe_edge_param(table, symbol, owner, false),
        Label::Reachability { lhs, rhs, .. } => {
            add_from_edge_name(table, lhs);
            add_from_edge_name(table, rhs);
        }
    }
}

fn add_from_expression(
    table: &mut SymbolTableBuilder,
    expr: &Expression<Identifier>,
    owner: &Option<usize>,
) {
    match expr {
        Expression::Reference { identifier } => {
            add_maybe_edge_param(table, identifier, owner, true);
        }
        Expression::Access { lhs, rhs, .. } => {
            add_from_expression(table, lhs, owner);
            add_from_expression(table, rhs, owner);
        }
        Expression::Cast { lhs, rhs, .. } => {
            add_from_type(table, lhs);
            add_from_expression(table, rhs, owner);
        }
    }
}

// Returns symbol idx for edge name if it has parameters
fn add_from_edge_name(
    table: &mut SymbolTableBuilder,
    node: &Node<Identifier>,
) -> Option<usize> {
    table.add_occ_with_flag(&node.identifier, Flag::Function);
    None
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
        for edge_name in pragma.nodes() {
            add_from_edge_name(&mut table, edge_name);
        }
    });
    table
}

fn add_builtin_symbols(table: &mut SymbolTableBuilder) {
    if !table.is_defined("Bool") {
        table.symbols.push(make_builtin("Bool", Flag::Type));
        table.symbols.push(make_builtin("0", Flag::Member));
        table.symbols.push(make_builtin("1", Flag::Member));
    }
    if !table.is_defined("Goals") {
        table.symbols.push(make_builtin("Goals", Flag::Type));
    }
    if !table.is_defined("Visibility") {
        table.symbols.push(make_builtin("Visibility", Flag::Type));
    }
    if !table.is_defined("keeper") {
        table.symbols.push(make_builtin("keeper", Flag::Variable));
    }
    if !table.is_defined("random") {
        table.symbols.push(make_builtin("random", Flag::Variable));
    }
    if !table.is_defined("PlayerOrSystem") {
        table
            .symbols
            .push(make_builtin("PlayerOrSystem", Flag::Type));
    }
    if !table.is_defined("goals") {
        table.symbols.push(make_builtin("goals", Flag::Variable));
    }
    if !table.is_defined("player") {
        table.symbols.push(make_builtin("player", Flag::Variable));
    }
    if !table.is_defined("visible") {
        table.symbols.push(make_builtin("visible", Flag::Variable));
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
