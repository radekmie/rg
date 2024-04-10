use super::symbol::Symbols;
use crate::common::symbol::{make_builtin_type, make_builtin_variable, Flag, Symbol};
use crate::common::symbol_table::{Occurrence, SymbolTable, SymbolTableBuilder};
use rg::ast::{Edge, Expression, Game, Label, Node, NodePart, Type, Value, ValueEntry};
use utils::{Error, Identifier};
use utils::position::{Positioned, Span};

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
    if !identifier.is_none() {
        let span = identifier.span();
        let sym_idx = table
            .find_symbol(&identifier.identifier, &Some(Flag::Param), owner)
            .or_else(|| table.find_symbol(&identifier.identifier, &None, &None));
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
        Label::Comparison { lhs, rhs, .. } => {
            add_from_expression(table, lhs, owner);
            add_from_expression(table, rhs, owner);
        }
        Label::Skip { .. } => (),
        Label::Tag { symbol } => add_maybe_edge_param(table, symbol, owner, false),
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
    edge_name: &Node<Identifier>,
) -> Option<usize> {
    match edge_name.parts.as_slice() {
        [NodePart::Literal { identifier }] => {
            table.add_occ_with_flag(identifier, Flag::Edge);
            None
        }
        [NodePart::Literal { identifier }, bindings @ ..] => {
            let occ = table.occ_with_flag(identifier, Flag::Edge);
            let sym_idx = occ.symbol;
            table.occurrences.push(occ);
            for binding in bindings {
                add_from_name_part(table, binding, &sym_idx);
            }
            sym_idx
        }
        _ => None,
    }
}

fn add_from_name_part(
    table: &mut SymbolTableBuilder,
    name_part: &NodePart<Identifier>,
    owner: &Option<usize>,
) {
    match name_part {
        NodePart::Binding {
            identifier, type_, ..
        } => {
            table.add_occ_with_flag_and_owner(identifier, Flag::Param, owner);
            add_from_type(table, type_);
        }
        NodePart::Literal { identifier } => {
            table.add_occ_with_flag(identifier, Flag::Edge);
        }
    }
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
        table.symbols.push(make_builtin_type("Bool"));
        table.symbols.push(Symbol::new(
            "0".to_string(),
            Span::none(),
            Flag::Member,
            None,
        ));
        table.symbols.push(Symbol::new(
            "1".to_string(),
            Span::none(),
            Flag::Member,
            None,
        ));
    }
    if !table.is_defined("Goals") {
        table.symbols.push(make_builtin_type("Goals"));
    }
    if !table.is_defined("Visibility") {
        table.symbols.push(make_builtin_type("Visibility"));
    }
    if !table.is_defined("keeper") {
        table.symbols.push(make_builtin_variable("keeper"));
    }
    if !table.is_defined("PlayerOrKeeper") {
        table.symbols.push(make_builtin_type("PlayerOrKeeper"));
    }
    if !table.is_defined("goals") {
        table.symbols.push(make_builtin_variable("goals"));
    }
    if !table.is_defined("player") {
        table.symbols.push(make_builtin_variable("player"));
    }
    if !table.is_defined("visible") {
        table.symbols.push(make_builtin_variable("visible"));
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
