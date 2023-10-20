use crate::rg::ast::*;
use crate::rg::position::{Positioned, Span};

use super::position::Position;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol {
    pub id: String,
    pub pos: Span,
    pub flag: u32,
    owner: Option<usize>,
}

impl Symbol {
    fn new(id: String, pos: Span, flag: u32, owner: Option<usize>) -> Self {
        Self {
            id,
            pos,
            flag,
            owner,
        }
    }

    fn from_id(identifier: &Identifier, flag: u32, owner: Option<usize>) -> Self {
        let id = identifier.identifier.clone();
        let pos = identifier.span().clone();
        Self::new(id, pos, flag, owner)
    }

    fn from_type(type_: &Type, owner: usize, mut acc: Vec<Self>) -> Vec<Self> {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                Self::from_type(rhs, owner, Self::from_type(lhs, owner, acc))
            }
            Type::TypeReference { .. } => acc,
            Type::Set { identifiers, .. } => {
                for identifier in identifiers.iter() {
                    let symbol = Self::from_id(identifier, Flag::to_u32(&Flag::Member), None);
                    acc.push(symbol);
                }
                acc
            }
        }
    }

    fn from_edge_name(edge_name: &EdgeName, mut acc: Vec<Self>) -> Vec<Self> {
        match edge_name.parts.as_slice() {
            [EdgeNamePart::Literal { identifier }] => {
                acc.push(Self::from_id(identifier, 0, None));
                acc
            }
            [EdgeNamePart::Literal { identifier }, bindings @ ..] => {
                let symbol = Self::from_id(identifier, 0, None);
                acc.push(symbol);
                let symbol_idx = acc.len() - 1;
                for binding in bindings.iter() {
                    acc.push(Self::from_name_part(binding, symbol_idx));
                }
                acc
            }
            _ => acc,
        }
    }

    fn from_name_part(name_part: &EdgeNamePart, owner: usize) -> Self {
        match name_part {
            EdgeNamePart::Binding { identifier, .. } => {
                let flag = Flag::to_u32(&Flag::Param);
                let symbol = Symbol::from_id(identifier, flag, Some(owner));
                symbol
            }
            EdgeNamePart::Literal { identifier } => Symbol::from_id(identifier, 0, None),
        }
    }

    fn from_edge(edge: &Edge, mut acc: Vec<Self>) -> Vec<Self> {
        let mut left_defined: Vec<Self> = Self::from_edge_name(&edge.lhs, Vec::new());
        let mut right_defined: Vec<Self> = Self::from_edge_name(&edge.rhs, Vec::new())
            .into_iter()
            .filter(|right| !left_defined.iter().any(|left| left.id == right.id))
            .collect();
        acc.append(&mut left_defined);
        acc.append(&mut right_defined);
        acc
    }

    pub fn from_game(game: &Game) -> Vec<Symbol> {
        let mut symbols: Vec<Symbol> = Vec::new();
        for stat in game.stats.iter() {
            match stat {
                Stat::Constant(constant) => {
                    let id = &constant.identifier;
                    let flag = Flag::to_u32(&Flag::Constant);
                    let symbol = Symbol::from_id(id, flag, None);
                    symbols.push(symbol);
                }
                Stat::Variable(variable) => {
                    let id = &variable.identifier;
                    let flag = Flag::to_u32(&Flag::Variable);
                    let symbol = Symbol::from_id(id, flag, None);
                    symbols.push(symbol);
                }

                Stat::Edge(edge) => {
                    symbols = Self::from_edge(&edge, symbols);
                }
                Stat::Typedef(typedef) => {
                    let id = &typedef.identifier;
                    let flag = Flag::to_u32(&Flag::Type);
                    let symbol = Symbol::from_id(id, flag, None);
                    symbols.push(symbol);
                    let symbol_idx = symbols.len() - 1;
                    symbols = Symbol::from_type(&typedef.type_, symbol_idx, symbols);
                }
                Stat::Pragma(_) => (),
            }
        }
        symbols
    }
}

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
        for (idx, symbol) in self.symbols.iter().enumerate() {
            if symbol.id == id && symbol.pos <= *pos {
                return Some(idx);
            } else if symbol.pos > *pos {
                return None;
            }
        }
        None
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

pub enum Flag {
    Type,
    Member,
    Constant,
    Variable,
    Edge,
    Param,
}

impl Flag {
    pub fn to_u32(&self) -> u32 {
        match self {
            Flag::Type => 1,
            Flag::Member => 2,
            Flag::Constant => 4,
            Flag::Variable => 8,
            Flag::Edge => 16,
            Flag::Param => 32,
        }
    }

    pub fn is_type(flag: u32) -> bool {
        flag & Flag::Type.to_u32() != 0
    }

    pub fn is_member(flag: u32) -> bool {
        flag & Flag::Member.to_u32() != 0
    }

    pub fn is_constant(flag: u32) -> bool {
        flag & Flag::Constant.to_u32() != 0
    }

    pub fn is_variable(flag: u32) -> bool {
        flag & Flag::Variable.to_u32() != 0
    }

    pub fn is_edge(flag: u32) -> bool {
        flag & Flag::Edge.to_u32() != 0
    }

    pub fn is_param(flag: u32) -> bool {
        flag & Flag::Param.to_u32() != 0
    }

    pub fn from_u32(flag_set: u32) -> Flag {
        if Self::is_type(flag_set) {
            return Flag::Type;
        }
        if Self::is_member(flag_set) {
            return Flag::Member;
        }
        if Self::is_constant(flag_set) {
            return Flag::Constant;
        }
        if Self::is_variable(flag_set) {
            return Flag::Variable;
        }
        if Self::is_edge(flag_set) {
            return Flag::Edge;
        }
        if Self::is_param(flag_set) {
            return Flag::Param;
        }
        panic!("Invalid flag set: {}", flag_set);
    }
}
