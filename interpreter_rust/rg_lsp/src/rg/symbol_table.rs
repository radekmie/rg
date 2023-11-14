use std::fmt::Display;

use crate::rg::ast::*;
use crate::rg::position::{Positioned, Span};

use super::error::Error;
use super::position::Position;
use super::symbol::{from_game as symbols_from_game, Flag, Symbol};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Occurrence {
    pub pos: Span,
    pub symbol: Option<usize>,
}

impl Occurrence {
    fn new(pos: Span, symbol: Option<usize>) -> Self {
        Self { pos, symbol }
    }
}

impl Positioned for Occurrence {
    fn span(&self) -> Span {
        self.pos.clone()
    }
}

pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub occurrences: Vec<Occurrence>,
}

struct SymbolTableWithErrors {
    symbols: Vec<Symbol>,
    occurrences: Vec<Occurrence>,
    errors: Vec<Error>,
}

impl SymbolTableWithErrors {
    /*
     * The last symbol with matching id defined before this position is used.
     */
    fn find_symbol(
        &self,
        pos: &Span,
        id: &str,
        flag: Option<Flag>,
        owner: &Option<usize>,
    ) -> Option<usize> {
        let mut sym = None;
        for (idx, symbol) in self.symbols.iter().enumerate() {
            if symbol.id == id
                && flag.as_ref().map_or(true, |f| symbol.flag == *f)
                && owner.as_ref().map_or(true, |o| symbol.is_owned_by(*o))
            {
                sym = Some(idx);
            } else if symbol.pos > *pos && sym.is_some() {
                return sym;
            }
        }
        sym
    }

    fn occ_from_id(&self, identifier: &Identifier) -> Occurrence {
        let span = identifier.span();
        let symbol_idx = self.find_symbol(&span, &identifier.identifier, None, &None);
        Occurrence::new(span, symbol_idx)
    }

    fn occ_with_flag(&self, identifier: &Identifier, flag: Flag) -> Occurrence {
        let span = identifier.span();
        let symbol_idx = self.find_symbol(&span, &identifier.identifier, Some(flag), &None);
        Occurrence::new(span, symbol_idx)
    }

    fn add_occ(&mut self, identifier: &Identifier) {
        if !identifier.is_none() {
            let occ = self.occ_from_id(identifier);
            if occ.symbol.is_none() {
                self.errors.push(Error::symbol_table_error(identifier));
            } else {
                self.occurrences.push(occ);
            }
        }
    }

    fn add_occ_with_flag(&mut self, identifier: &Identifier, flag: Flag) {
        if !identifier.is_none() {
            let occ = self.occ_with_flag(identifier, flag);
            if occ.symbol.is_none() {
                self.errors.push(Error::symbol_table_error(identifier));
            } else {
                self.occurrences.push(occ);
            }
        }
    }

    fn add_occ_with_flag_and_owner(
        &mut self,
        identifier: &Identifier,
        flag: Flag,
        owner: &Option<usize>,
    ) {
        if !identifier.is_none() {
            let span = identifier.span();
            let symbol_idx = self.find_symbol(&span, &identifier.identifier, Some(flag), owner);
            if symbol_idx.is_none() {
                self.errors.push(Error::symbol_table_error(identifier));
            } else {
                self.occurrences.push(Occurrence::new(span, symbol_idx));
            }
        }
    }

    fn add_from_type(&mut self, type_: &Type) {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                self.add_from_type(lhs);
                self.add_from_type(rhs);
            }
            Type::TypeReference { identifier } => {
                self.add_occ_with_flag(identifier, Flag::Type);
            }
            Type::Set { identifiers, .. } => {
                for identifier in identifiers.iter() {
                    self.add_occ_with_flag(identifier, Flag::Member);
                }
            }
        }
    }

    fn add_from_edge(&mut self, edge: &Edge) {
        let left_owner = self.add_from_edge_name(&edge.lhs);
        let right_owner = self.add_from_edge_name(&edge.rhs);
        let owner = left_owner.or_else(|| right_owner);
        self.add_from_edge_label(&edge.label, &owner);
    }

    fn add_maybe_edge_param(
        &mut self,
        identifier: &Identifier,
        owner: &Option<usize>,
        create_error: bool,
    ) {
        if !identifier.is_none() {
            let span = identifier.span();
            let sym_idx =
                match self.find_symbol(&span, &identifier.identifier, Some(Flag::Param), owner) {
                    Some(idx) => Some(idx),
                    None => self.find_symbol(&span, &identifier.identifier, None, &None),
                };
            if sym_idx.is_none() && create_error {
                self.errors.push(Error::symbol_table_error(identifier));
            } else if sym_idx.is_some() {
                self.occurrences.push(Occurrence::new(span, sym_idx));
            }
        }
    }

    fn add_from_edge_label(&mut self, label: &EdgeLabel, owner: &Option<usize>) {
        match label {
            EdgeLabel::Assignment { lhs, rhs } => {
                self.add_from_expression(lhs, owner);
                self.add_from_expression(rhs, owner);
            }
            EdgeLabel::Comparison { lhs, rhs, .. } => {
                self.add_from_expression(lhs, owner);
                self.add_from_expression(rhs, owner);
            }
            EdgeLabel::Skip { .. } => (),
            EdgeLabel::Tag { symbol } => self.add_maybe_edge_param(symbol, owner, false),
            EdgeLabel::Reachability { lhs, rhs, .. } => {
                self.add_from_edge_name(lhs);
                self.add_from_edge_name(rhs);
            }
        }
    }

    fn add_from_expression(&mut self, expr: &Expression, owner: &Option<usize>) {
        match expr {
            Expression::Reference { identifier } => {
                self.add_maybe_edge_param(identifier, owner, true)
            }
            Expression::Access { lhs, rhs, .. } => {
                self.add_from_expression(lhs, owner);
                self.add_from_expression(rhs, owner);
            }
            Expression::Cast { lhs, rhs, .. } => {
                self.add_from_type(lhs);
                self.add_from_expression(rhs, owner);
            }
        }
    }

    // Returns symbol idx for edge name if it has parameters
    fn add_from_edge_name(&mut self, edge_name: &EdgeName) -> Option<usize> {
        match edge_name.parts.as_slice() {
            [EdgeNamePart::Literal { identifier }] => {
                self.add_occ_with_flag(identifier, Flag::Edge);
                None
            }
            [EdgeNamePart::Literal { identifier }, bindings @ ..] => {
                let occ = self.occ_with_flag(identifier, Flag::Edge);
                let sym_idx = occ.symbol;
                self.occurrences.push(occ);
                for binding in bindings.iter() {
                    self.add_from_name_part(binding, &sym_idx);
                }
                sym_idx
            }
            _ => None,
        }
    }

    fn add_from_name_part(&mut self, name_part: &EdgeNamePart, owner: &Option<usize>) {
        match name_part {
            EdgeNamePart::Binding {
                identifier, type_, ..
            } => {
                self.add_occ_with_flag_and_owner(identifier, Flag::Param, owner);
                self.add_from_type(type_);
            }
            EdgeNamePart::Literal { identifier } => {
                self.add_occ_with_flag(identifier, Flag::Edge);
            }
        }
    }

    fn add_from_value(&mut self, value: &Value) {
        match value {
            Value::Element { identifier } => {
                self.add_occ(identifier);
            }
            Value::Map { entries, .. } => {
                for entry in entries.iter() {
                    self.add_from_value_entry(entry);
                }
            }
        }
    }

    fn add_from_value_entry(&mut self, entry: &ValueEntry) {
        if let Some(identifier) = entry.identifier.as_ref() {
            self.add_occ(identifier);
        }
        self.add_from_value(&entry.value);
    }

    pub fn from_game(game: &Game) -> Self {
        let mut table: Self = Self {
            symbols: symbols_from_game(game),
            occurrences: Vec::new(),
            errors: Vec::new(),
        };
        table.add_builtin_symbols();
        for stat in game.stats.iter() {
            match stat {
                Stat::Constant(constant) => {
                    table.add_occ_with_flag(&constant.identifier, Flag::Constant);
                    table.add_from_type(&constant.type_);
                    table.add_from_value(&constant.value);
                }
                Stat::Variable(variable) => {
                    table.add_occ_with_flag(&variable.identifier, Flag::Variable);
                    table.add_from_type(&variable.type_);
                    table.add_from_value(&variable.default_value);
                }
                Stat::Pragma(pragma) => {
                    table.add_from_edge_name(&pragma.edge_name);
                }
                Stat::Edge(edge) => {
                    table.add_from_edge(edge);
                }
                Stat::Typedef(typedef) => {
                    table.add_occ_with_flag(&typedef.identifier, Flag::Type);
                    table.add_from_type(&typedef.type_);
                }
            }
        }
        table
    }

    fn is_defined(&self, symbol: &str) -> bool {
        self.symbols.iter().any(|sym| sym.id == symbol)
    }

    fn make_builtin_type(symbol: &str) -> Symbol {
        Symbol::new(symbol.to_string(), Span::none(), Flag::Type, None)
    }

    fn make_builtin_variable(symbol: &str) -> Symbol {
        Symbol::new(symbol.to_string(), Span::none(), Flag::Variable, None)
    }

    fn add_builtin_symbols(&mut self) {
        if !self.is_defined("Bool") {
            self.symbols.push(Self::make_builtin_type("Bool"));
            self.symbols.push(Symbol::new(
                "0".to_string(),
                Span::none(),
                Flag::Member,
                None,
            ));
            self.symbols.push(Symbol::new(
                "1".to_string(),
                Span::none(),
                Flag::Member,
                None,
            ));
        }
        if !self.is_defined("Goals") {
            self.symbols.push(Self::make_builtin_type("Goals"));
        }
        if !self.is_defined("Visibility") {
            self.symbols.push(Self::make_builtin_type("Visibility"));
        }
        if !self.is_defined("keeper") {
            self.symbols.push(Self::make_builtin_variable("keeper"));
        }
        if !self.is_defined("PlayerOrKeeper") {
            self.symbols.push(Self::make_builtin_type("PlayerOrKeeper"));
        }
        if !self.is_defined("goals") {
            self.symbols.push(Self::make_builtin_variable("goals"));
        }
        if !self.is_defined("player") {
            self.symbols.push(Self::make_builtin_variable("player"));
        }
        if !self.is_defined("visible") {
            self.symbols.push(Self::make_builtin_variable("visible"));
        }
    }
}

impl SymbolTable {
    pub fn get_occ_at(&self, pos: &Position) -> Option<&Occurrence> {
        for occurrence in self.occurrences.iter() {
            if occurrence.pos.encloses_pos(pos) {
                return Some(occurrence);
            }
        }
        None
    }

    pub fn get_occ_at_span(&self, span: &Span) -> Option<&Occurrence> {
        for occurrence in self.occurrences.iter() {
            if occurrence.pos.encloses_span(span) {
                return Some(occurrence);
            }
        }
        None
    }

    pub fn get_occ_symbol(&self, occ: &Occurrence) -> Option<&Symbol> {
        match occ.symbol {
            Some(idx) => self.symbols.get(idx),
            None => None,
        }
    }

    pub fn get_symbol_at(&self, pos: &Position) -> Option<&Symbol> {
        match self.get_occ_at(pos) {
            Some(occ) => self.get_occ_symbol(occ),
            None => None,
        }
    }

    pub fn get_symbol_at_span(&self, span: &Span) -> Option<&Symbol> {
        match self.get_occ_at_span(span) {
            Some(occ) => self.get_occ_symbol(occ),
            None => None,
        }
    }

    pub fn sym_idx(&self, symbol: &Symbol) -> Option<usize> {
        for (idx, sym) in self.symbols.iter().enumerate() {
            if sym == symbol {
                return Some(idx);
            }
        }
        None
    }

    pub fn from_game(game: &Game) -> (Self, Vec<Error>) {
        let table = SymbolTableWithErrors::from_game(game);
        (
            Self {
                symbols: table.symbols,
                occurrences: table.occurrences,
            },
            table.errors,
        )
    }
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Symbols:\n")?;
        for (idx, symbol) in self.symbols.iter().enumerate() {
            writeln!(f, "{}. {}  {}", idx, symbol.pos, symbol)?;
        }
        writeln!(f, "Occurrences:\n")?;
        for occ in self.occurrences.iter() {
            let symbol = self.get_occ_symbol(occ).unwrap();
            writeln!(f, "{}  {}", occ.pos, symbol)?;
        }
        Ok(())
    }
}
