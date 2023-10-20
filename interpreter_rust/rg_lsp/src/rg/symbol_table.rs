use crate::rg::ast::*;
use crate::rg::position::{Positioned, Span};

use super::position::Position;
use super::symbol::{Flag, Symbol};

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

pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub occurrences: Vec<Occurrence>,
}

impl SymbolTable {
    fn find_symbol(&self, pos: &Span, id: &str) -> Option<usize> {
        let mut sym = None;
        for (idx, symbol) in self.symbols.iter().enumerate() {
            if symbol.id == id && symbol.pos <= *pos {
                sym = Some(idx);
            } else if symbol.pos > *pos {
                return sym;
            }
        }
        sym
    }

    fn add_occ_from_id(&mut self, identifier: &Identifier) {
        self.occurrences.push(self.occ_from_id(identifier));
    }

    fn occ_from_id(&self, identifier: &Identifier) -> Occurrence {
        let span = identifier.span();
        let symbol_idx = self.find_symbol(&span, &identifier.identifier);
        Occurrence::new(span, symbol_idx)
    }

    fn add_from_type(&mut self, type_: &Type) {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                self.add_from_type(lhs);
                self.add_from_type(rhs);
            }
            Type::TypeReference { identifier } => {
                self.add_occ_from_id(identifier);
            }
            Type::Set { identifiers, .. } => {
                for identifier in identifiers.iter() {
                    self.add_occ_from_id(identifier);
                }
            }
        }
    }

    fn add_from_edge(&mut self, edge: &Edge) {
        self.add_from_edge_name(&edge.rhs);
        self.add_from_edge_name(&edge.rhs);
        self.add_from_edge_label(&edge.label);
    }

    fn add_from_edge_label(&mut self, label: &EdgeLabel) {
        match label {
            EdgeLabel::Assignment { lhs, rhs } => {
                self.add_from_expression(lhs);
                self.add_from_expression(rhs);
            }
            EdgeLabel::Comparison { lhs, rhs, .. } => {
                self.add_from_expression(lhs);
                self.add_from_expression(rhs);
            }
            EdgeLabel::Skip { .. } => (),
            EdgeLabel::Tag { symbol } => {
                self.add_occ_from_id(symbol);
            }
            EdgeLabel::Reachability { lhs, rhs, .. } => {
                self.add_from_edge_name(lhs);
                self.add_from_edge_name(rhs);
            }
        }
    }

    fn add_from_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Reference { identifier } => {
                self.add_occ_from_id(identifier);
            }
            Expression::Access { lhs, rhs, .. } => {
                self.add_from_expression(lhs);
                self.add_from_expression(rhs);
            }
            Expression::Cast { lhs, rhs, .. } => {
                self.add_from_type(lhs);
                self.add_from_expression(rhs);
            }
        }
    }

    fn add_from_edge_name(&mut self, edge_name: &EdgeName) {
        match edge_name.parts.as_slice() {
            [EdgeNamePart::Literal { identifier }] => {
                self.add_occ_from_id(identifier);
            }
            [EdgeNamePart::Literal { identifier }, bindings @ ..] => {
                self.add_occ_from_id(identifier);
                for binding in bindings.iter() {
                    self.add_from_name_part(binding);
                }
            }
            _ => (),
        }
    }

    fn add_from_name_part(&mut self, name_part: &EdgeNamePart) {
        match name_part {
            EdgeNamePart::Binding { identifier, .. } => {
                self.add_occ_from_id(identifier);
            }
            EdgeNamePart::Literal { identifier } => {
                self.add_occ_from_id(identifier);
            }
        }
    }

    fn add_from_value(&mut self, value: &Value) {
        match value {
            Value::Element { identifier } => {
                self.add_occ_from_id(identifier);
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
            self.add_occ_from_id(identifier);
        }
        self.add_from_value(&entry.value);
    }

    pub fn from_game(game: &Game) -> Self {
        let mut table: Self = Self {
            symbols: Symbol::from_game(game),
            occurrences: Vec::new(),
        };
        table.add_builtin_symbols();
        for stat in game.stats.iter() {
            match stat {
                Stat::Constant(constant) => {
                    table.add_occ_from_id(&constant.identifier);
                    table.add_from_type(&constant.type_);
                    table.add_from_value(&constant.value);
                }
                Stat::Variable(variable) => {
                    table.add_occ_from_id(&variable.identifier);
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
                    table.add_occ_from_id(&typedef.identifier);
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
        Symbol::new(
            symbol.to_string(),
            Span::none(),
            Flag::to_u32(&Flag::Type),
            None,
        )
    }

    fn make_builtin_variable(symbol: &str) -> Symbol {
        Symbol::new(
            symbol.to_string(),
            Span::none(),
            Flag::to_u32(&Flag::Variable),
            None,
        )
    }

    fn add_builtin_symbols(&mut self) {
        if !self.is_defined("Bool") {
            self.symbols.push(Self::make_builtin_type("Bool"));
            let bool_idx = self.symbols.len() - 1;
            self.symbols.push(Symbol::new(
                "0".to_string(),
                Span::none(),
                Flag::to_u32(&Flag::Member),
                Some(bool_idx),
            ));
            self.symbols.push(Symbol::new(
                "1".to_string(),
                Span::none(),
                Flag::to_u32(&Flag::Member),
                Some(bool_idx),
            ));
        }
        if !self.is_defined("Goals") {
            self.symbols.push(Self::make_builtin_type("Goals"));
        }
        if !self.is_defined("Visibility") {
            self.symbols.push(Self::make_builtin_type("Visibility"));
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

    pub fn get_symbol_owner(&self, symbol: &Symbol) -> Option<&Symbol> {
        match symbol.owner {
            Some(idx) => self.symbols.get(idx),
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
}
