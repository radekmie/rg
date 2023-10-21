use crate::rg::ast::*;
use crate::rg::position::{Positioned, Span};

use super::position::Position;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol {
    pub id: String,
    pub pos: Span,
    pub flag: u32,
    pub owner: Option<usize>,
}

impl Symbol {
    pub fn new(id: String, pos: Span, flag: u32, owner: Option<usize>) -> Self {
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

impl Positioned for Symbol {
    fn span(&self) -> Span {
        self.pos.clone()
    }
}
