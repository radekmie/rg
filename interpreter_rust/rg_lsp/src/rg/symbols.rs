use crate::rg::ast::*;
use crate::rg::position::{Positioned, Span};

pub struct Symbol {
    pub id: String,
    pub pos: Span,
    pub flags: u32,
    pub owner: Option<String>,
}

impl Symbol {
    fn new(id: String, pos: Span, flags: u32, owner: Option<String>) -> Self {
        Self {
            id,
            pos,
            flags,
            owner,
        }
    }

    fn from_id(identifier: &Identifier, flags: u32, owner: Option<String>) -> Self {
        let id = identifier.identifier.clone();
        let pos = identifier.span().clone();
        Self::new(id, pos, flags, owner)
    }

    fn from_type(type_: &Type, owner: &str) -> Vec<Self> {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                let mut symbols = Self::from_type(lhs, owner);
                symbols.append(&mut Self::from_type(rhs, owner));
                symbols
            }
            Type::TypeReference { .. } => vec![],
            Type::Set { identifiers, .. } => identifiers
                .iter()
                .map(|id| Self::from_id(id, Flag::to_u32(&Flag::Member), None))
                .collect::<Vec<Self>>(),
        }
    }

    fn from_edge_name(edge_name: &EdgeName) -> Vec<Self> {
        match edge_name.parts.as_slice() {
            [EdgeNamePart::Literal { identifier }] => vec![Self::from_id(identifier, 0, None)],
            [EdgeNamePart::Literal { identifier }, bindings @ ..] => {
                let symbol = Self::from_id(identifier, 0, None);
                let mut bindings = bindings
                    .iter()
                    .map(|binding| Self::from_name_part(binding, &symbol.id))
                    .collect::<Vec<Self>>();
                let mut symbols = vec![symbol];
                symbols.append(&mut bindings);
                symbols
            }
            _ => vec![],
        }
    }

    fn from_name_part(name_part: &EdgeNamePart, owner: &str) -> Self {
        match name_part {
            EdgeNamePart::Binding { identifier, .. } => {
                let flag = Flag::to_u32(&Flag::Param);
                let symbol = Symbol::from_id(identifier, flag, Some(owner.to_string()));
                symbol
            }
            EdgeNamePart::Literal { identifier } => Symbol::from_id(identifier, 0, None),
        }
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
                    let mut l_symbols = Symbol::from_edge_name(&edge.lhs);
                    let mut r_symbols = Symbol::from_edge_name(&edge.rhs);
                    symbols.append(&mut l_symbols);
                    symbols.append(&mut r_symbols);
                }
                Stat::Typedef(typedef) => {
                    let id = &typedef.identifier;
                    let flag = Flag::to_u32(&Flag::Type);
                    let symbol = Symbol::from_id(id, flag, None);
                    symbols.push(symbol);
                    symbols.append(&mut Symbol::from_type(&typedef.type_, &id.identifier));
                }
                Stat::Pragma(_) => (),
            }
        }
        symbols
    }
}

pub struct Occurrence {
    pub id: String,
    pub pos: Span,
}

impl Occurrence {
    fn new(id: String, pos: Span) -> Self {
        Self { id, pos }
    }

    fn from_id(identifier: &Identifier) -> Self {
        let id = identifier.identifier.clone();
        let pos = identifier.span().clone();
        Self::new(id, pos)
    }

    fn from_type(type_: &Type) -> Vec<Self> {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                let mut occurrences = Self::from_type(lhs);
                occurrences.append(&mut Self::from_type(rhs));
                occurrences
            }
            Type::TypeReference { identifier } => vec![Self::from_id(identifier)],
            Type::Set { identifiers, .. } => identifiers
                .iter()
                .map(|id| Self::from_id(id))
                .collect::<Vec<Self>>(),
        }
    }

    fn from_edge(edge: &Edge) -> Vec<Self> {
        let mut occurrences = Self::from_edge_name(&edge.lhs);
        occurrences.append(&mut Self::from_edge_name(&edge.rhs));
        occurrences.append(&mut Self::from_edge_label(&edge.label));
        occurrences
    }

    fn from_edge_label(label: &EdgeLabel) -> Vec<Self> {
        match label {
            EdgeLabel::Assignment { lhs, rhs } => {
                let mut occurrences = Self::from_expression(lhs);
                occurrences.append(&mut Self::from_expression(rhs));
                occurrences
            }
            EdgeLabel::Comparison { lhs, rhs, .. } => {
                let mut occurrences = Self::from_expression(lhs);
                occurrences.append(&mut Self::from_expression(rhs));
                occurrences
            }
            EdgeLabel::Skip { .. } => vec![],
            EdgeLabel::Tag { symbol } => vec![Self::from_id(symbol)],
            EdgeLabel::Reachability { lhs, rhs, .. } => {
                let mut occurrences = Self::from_edge_name(lhs);
                occurrences.append(&mut Self::from_edge_name(rhs));
                occurrences
            }
        }
    }

    fn from_expression(expr: &Expression) -> Vec<Self> {
        match expr {
            Expression::Reference { identifier } => vec![Self::from_id(identifier)],
            Expression::Access { lhs, rhs, .. } => {
                let mut occurrences = Self::from_expression(lhs);
                occurrences.append(&mut Self::from_expression(rhs));
                occurrences
            }
            Expression::Cast { lhs, rhs, .. } => {
                let mut occurrences = Self::from_type(lhs);
                occurrences.append(&mut Self::from_expression(rhs));
                occurrences
            }
        }
    }

    fn from_edge_name(edge_name: &EdgeName) -> Vec<Self> {
        match edge_name.parts.as_slice() {
            [EdgeNamePart::Literal { identifier }] => vec![Self::from_id(identifier)],
            [EdgeNamePart::Literal { identifier }, bindings @ ..] => {
                let symbol = Self::from_id(identifier);
                let mut bindings = bindings
                    .iter()
                    .map(|binding| Self::from_name_part(binding))
                    .collect::<Vec<Self>>();
                let mut occurrences = vec![symbol];
                occurrences.append(&mut bindings);
                occurrences
            }
            _ => vec![],
        }
    }

    fn from_name_part(name_part: &EdgeNamePart) -> Self {
        match name_part {
            EdgeNamePart::Binding { identifier, .. } => Self::from_id(identifier),
            EdgeNamePart::Literal { identifier } => Self::from_id(identifier),
        }
    }

    pub fn from_game(game: &Game) -> Vec<Occurrence> {
        let mut occurrences: Vec<Occurrence> = Vec::new();
        for stat in game.stats.iter() {
            match stat {
                Stat::Constant(constant) => {
                    let id = &constant.identifier;
                    let symbol = Occurrence::from_id(id);
                    occurrences.push(symbol);
                }
                Stat::Variable(variable) => {
                    let id = &variable.identifier;
                    let symbol = Occurrence::from_id(id);
                    occurrences.push(symbol);
                }
                Stat::Pragma(pragma) => {
                    occurrences.append(&mut Occurrence::from_edge_name(&pragma.edge_name))
                }
                Stat::Edge(edge) => {
                    occurrences.append(&mut Occurrence::from_edge(edge));
                }
                Stat::Typedef(typedef) => {
                    let id = &typedef.identifier;
                    let symbol = Occurrence::from_id(id);
                    occurrences.push(symbol);
                    occurrences.append(&mut Occurrence::from_type(&typedef.type_));
                }
            }
        }
        occurrences
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

    fn is_type(flag: u32) -> bool {
        flag & Flag::Type.to_u32() != 0
    }

    fn is_member(flag: u32) -> bool {
        flag & Flag::Member.to_u32() != 0
    }

    fn is_constant(flag: u32) -> bool {
        flag & Flag::Constant.to_u32() != 0
    }

    fn is_variable(flag: u32) -> bool {
        flag & Flag::Variable.to_u32() != 0
    }

    fn is_edge(flag: u32) -> bool {
        flag & Flag::Edge.to_u32() != 0
    }

    fn is_param(flag: u32) -> bool {
        flag & Flag::Param.to_u32() != 0
    }

    pub fn from_u32(flag_set: u32) -> Vec<Flag> {
        let mut flags = Vec::new();
        if Self::is_type(flag_set) {
            flags.push(Flag::Type);
        }
        if Self::is_member(flag_set) {
            flags.push(Flag::Member);
        }
        if Self::is_constant(flag_set) {
            flags.push(Flag::Constant);
        }
        if Self::is_variable(flag_set) {
            flags.push(Flag::Variable);
        }
        if Self::is_edge(flag_set) {
            flags.push(Flag::Edge);
        }
        if Self::is_param(flag_set) {
            flags.push(Flag::Param);
        }
        flags
    }
}
