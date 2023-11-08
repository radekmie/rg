use std::collections::HashSet;
use std::fmt::Display;

use crate::rg::ast::*;
use crate::rg::position::{Positioned, Span};

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

    fn from_id(identifier: &Identifier, flag: Flag, owner: Option<usize>) -> Option<Self> {
        if identifier.is_none() {
            None
        } else {
            let id = identifier.identifier.clone();
            let pos = identifier.span().clone();
            Some(Self::new(id, pos, flag, owner))
        }
    }
}

fn defined(symbols: &Vec<Symbol>, name: &str, flag: &Flag) -> Option<usize> {
    symbols
        .iter()
        .enumerate()
        .find(|(_, symbol)| symbol.id == name && symbol.flag == *flag)
        .map(|(idx, _)| idx)
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
            Flag::Edge => write!(f, "()"),
            Flag::Param => write!(f, "$"),
        }
    }
}

struct EdgeParam {
    param: Identifier,
    owners: HashSet<usize>,
}

struct Symbols {
    symbols: Vec<Symbol>,
    edge_params: Vec<EdgeParam>,
}

impl Symbols {
    fn add_from_type(&mut self, type_: &Type, owner: usize) {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                self.add_from_type(lhs, owner);
                self.add_from_type(rhs, owner);
            }
            Type::TypeReference { .. } => (),
            Type::Set { identifiers, .. } => {
                for identifier in identifiers.iter() {
                    let symbol = Symbol::from_id(identifier, Flag::Member, None);
                    if let Some(symbol) = symbol {
                        self.symbols.push(symbol);
                    }
                }
            }
        }
    }

    fn add_if_not_defined(&mut self, symbol: Option<Symbol>) -> Option<usize> {
        if let Some(symbol) = symbol {
            match defined(&self.symbols, &symbol.id, &symbol.flag) {
                Some(idx) => Some(idx),
                None => {
                    self.symbols.push(symbol);
                    Some(self.symbols.len() - 1)
                }
            }
        } else {
            None
        }
    }

    fn is_defined_param(&self, identifier: &str, owner: usize) -> Option<usize> {
        self.edge_params
            .iter()
            .enumerate()
            .find(|(_, param)| {
                param.param.identifier == identifier && param.owners.contains(&owner)
            })
            .map(|(idx, _)| idx)
    }

    fn add_from_edge(&mut self, edge: &Edge) {
        if let [EdgeNamePart::Literal {
            identifier: left_id,
        }, left_binds @ ..] = edge.lhs.parts.as_slice()
        {
            if let [EdgeNamePart::Literal {
                identifier: right_id,
            }, right_binds @ ..] = edge.rhs.parts.as_slice()
            {
                let left_idx = self.add_if_not_defined(Symbol::from_id(left_id, Flag::Edge, None));
                let right_idx =
                    self.add_if_not_defined(Symbol::from_id(right_id, Flag::Edge, None));
                let left_binds = left_binds
                    .iter()
                    .map(|bind| bind.identifier())
                    .collect::<Vec<&Identifier>>();
                let right_bind = right_binds
                    .iter()
                    .map(|bind| bind.identifier())
                    .collect::<Vec<&Identifier>>();
                let common_binds = left_binds
                    .iter()
                    .filter(|left| {
                        right_bind
                            .iter()
                            .any(|right| right.identifier == left.identifier)
                    })
                    .collect::<Vec<&&Identifier>>();
                let left_binds = left_binds
                    .iter()
                    .filter(|left| {
                        !common_binds
                            .iter()
                            .any(|common| common.identifier == left.identifier)
                    })
                    .collect::<Vec<&&Identifier>>();
                let right_bind = right_bind
                    .iter()
                    .filter(|right| {
                        !common_binds
                            .iter()
                            .any(|common| common.identifier == right.identifier)
                    })
                    .collect::<Vec<&&Identifier>>();
                if let Some(left_idx) = left_idx {
                    for bind in left_binds.iter() {
                        if self
                            .is_defined_param(bind.identifier.as_str(), left_idx)
                            .is_none()
                        {
                            self.edge_params.push(EdgeParam {
                                param: (**bind).clone(),
                                owners: HashSet::from([left_idx]),
                            });
                        }
                    }
                }
                if let Some(right_idx) = right_idx {
                    for bind in right_bind.iter() {
                        if self
                            .is_defined_param(bind.identifier.as_str(), right_idx)
                            .is_none()
                        {
                            self.edge_params.push(EdgeParam {
                                param: (**bind).clone(),
                                owners: HashSet::from([right_idx]),
                            });
                        }
                    }
                }

                if let (Some(left_idx), Some(right_idx)) = (left_idx, right_idx) {
                    for bind in common_binds.iter() {
                        let left_param_idx =
                            self.is_defined_param(bind.identifier.as_str(), left_idx);
                        let right_param_idx =
                            self.is_defined_param(bind.identifier.as_str(), right_idx);
                        match (left_param_idx, right_param_idx) {
                            (Some(left_param_idx), Some(right_param_idx)) => {
                                if left_param_idx != right_param_idx {
                                    let right_params =
                                        self.edge_params[right_param_idx].owners.clone();
                                    self.edge_params[left_param_idx].owners.extend(right_params);
                                    self.edge_params.remove(right_param_idx);
                                }
                            }
                            (Some(left_param_idx), None) => {
                                self.edge_params[left_param_idx].owners.extend([right_idx]);
                            }
                            (None, Some(right_param_idx)) => {
                                self.edge_params[right_param_idx].owners.extend([left_idx]);
                            }
                            (None, None) => {
                                self.edge_params.push(EdgeParam {
                                    param: (**bind).clone(),
                                    owners: HashSet::from([left_idx, right_idx]),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn from_game(game: &Game) -> Vec<Symbol> {
        let mut symbols: Self = Self {
            symbols: vec![],
            edge_params: vec![],
        };
        for stat in game.stats.iter() {
            match stat {
                Stat::Constant(constant) => {
                    let id = &constant.identifier;
                    let flag = Flag::Constant;
                    let symbol = Symbol::from_id(id, flag, None);
                    if let Some(symbol) = symbol {
                        symbols.symbols.push(symbol);
                    }
                }
                Stat::Variable(variable) => {
                    let id = &variable.identifier;
                    let flag = Flag::Variable;
                    let symbol = Symbol::from_id(id, flag, None);
                    if let Some(symbol) = symbol {
                        symbols.symbols.push(symbol);
                    }
                }

                Stat::Edge(edge) => {
                    symbols.add_from_edge(edge);
                }
                Stat::Typedef(typedef) => {
                    let id = &typedef.identifier;
                    let flag = Flag::Type;
                    let symbol = Symbol::from_id(id, flag, None);
                    if let Some(symbol) = symbol {
                        symbols.symbols.push(symbol);
                        let symbol_idx = symbols.symbols.len() - 1;
                        symbols.add_from_type(&typedef.type_, symbol_idx);
                    } else {
                        symbols.add_from_type(&typedef.type_, 0);
                    }
                }
                Stat::Pragma(_) => (),
            }
        }
        for bind in symbols.edge_params.iter() {
            let id = &bind.param;
            let owner = bind.owners.iter().next().unwrap();
            let flag = Flag::Param;
            let symbol = Symbol::from_id(id, flag, Some(*owner));
            if let Some(symbol) = symbol {
                symbols.symbols.push(symbol);
            }
        }
        symbols.symbols
    }
}

pub fn from_game(game: &Game) -> Vec<Symbol> {
    Symbols::from_game(game)
}
