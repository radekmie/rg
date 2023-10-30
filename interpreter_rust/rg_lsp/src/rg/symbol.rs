use std::fmt::Display;

use crate::rg::ast::*;
use crate::rg::position::{Positioned, Span};

use super::position::Position;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol {
    pub id: String,
    pub pos: Span,
    pub flag: Flag,
    pub owner: Option<usize>,
}

impl Symbol {
    pub fn new(id: String, pos: Span, flag: Flag, owner: Option<usize>) -> Self {
        Self {
            id,
            pos,
            flag,
            owner,
        }
    }

    fn from_id(identifier: &Identifier, flag: Flag, owner: Option<usize>) -> Self {
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
                    let symbol = Self::from_id(identifier, Flag::Member, None);
                    acc.push(symbol);
                }
                acc
            }
        }
    }

    fn from_edge_name(edge_name: &EdgeName, mut acc: Vec<Self>) -> Vec<Self> {
        match edge_name.parts.as_slice() {
            [EdgeNamePart::Literal { identifier }] => {
                acc.push(Self::from_id(identifier, Flag::Edge, None));
                acc
            }
            [EdgeNamePart::Literal { identifier }, bindings @ ..] => {
                let symbol = Self::from_id(identifier, Flag::Edge, None);
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
                let flag = Flag::Param;
                let symbol = Symbol::from_id(identifier, flag, Some(owner));
                symbol
            }
            EdgeNamePart::Literal { identifier } => Symbol::from_id(identifier, Flag::Edge, None),
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
                    let flag = Flag::Constant;
                    let symbol = Symbol::from_id(id, flag, None);
                    symbols.push(symbol);
                }
                Stat::Variable(variable) => {
                    let id = &variable.identifier;
                    let flag = Flag::Variable;
                    let symbol = Symbol::from_id(id, flag, None);
                    symbols.push(symbol);
                }

                Stat::Edge(edge) => {
                    symbols = Self::from_edge(&edge, symbols);
                }
                Stat::Typedef(typedef) => {
                    let id = &typedef.identifier;
                    let flag = Flag::Type;
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

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(owner) = self.owner {
            write!(f, "{}/{}{}", owner, self.id, self.flag)
        } else {
            write!(f, "{}{}", self.id, self.flag)
        }
    }
}

impl Positioned for Symbol {
    fn span(&self) -> Span {
        self.pos.clone()
    }
}


#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Flag {
    Type,
    Member,
    Constant,
    Variable,
    Edge,
    Param,
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flag::Type => write!(f, "#"),
            Flag::Member => write!(f, "."),
            Flag::Constant => write!(f, "!"),
            Flag::Variable => write!(f, "?"),
            Flag::Edge => write!(f, ":"),
            Flag::Param => write!(f, "$"),
        }
    }
}