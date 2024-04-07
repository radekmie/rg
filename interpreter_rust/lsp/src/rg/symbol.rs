use rg::ast::{Edge, Game, Identifier, NodePart, Type};
use rg::position::{Positioned, Span};
use std::collections::HashSet;
use std::fmt::Display;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol {
    pub flag: Flag,
    pub id: String,
    pub owners: Option<Vec<usize>>,
    pub pos: Span,
}

impl Symbol {
    pub fn new(id: String, pos: Span, flag: Flag, owners: Option<Vec<usize>>) -> Self {
        Self {
            flag,
            id,
            owners,
            pos,
        }
    }

    fn from_id(identifier: &Identifier, flag: Flag) -> Option<Self> {
        if identifier.is_none() {
            None
        } else {
            let id = identifier.identifier.clone();
            let pos = identifier.span();
            Some(Self::new(id, pos, flag, None))
        }
    }

    pub fn is_owned_by(&self, owner: usize) -> bool {
        self.owners
            .as_ref()
            .is_some_and(|owners| owners.contains(&owner))
    }

    pub fn safe_pos(&self) -> Option<Span> {
        if self.pos.is_none() {
            None
        } else {
            Some(self.pos)
        }
    }
}

fn defined(symbols: &[Symbol], name: &str, flag: &Flag) -> Option<usize> {
    symbols
        .iter()
        .enumerate()
        .find(|(_, symbol)| symbol.id == name && symbol.flag == *flag)
        .map(|(idx, _)| idx)
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.id, self.flag)
    }
}

impl Positioned for Symbol {
    fn span(&self) -> Span {
        self.pos
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
            Self::Type => write!(f, "#"),
            Self::Member => write!(f, "."),
            Self::Constant => write!(f, "!"),
            Self::Variable => write!(f, "?"),
            Self::Edge => write!(f, "()"),
            Self::Param => write!(f, "$"),
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
    fn add_from_type(&mut self, type_: &Type<Identifier>) {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                self.add_from_type(lhs);
                self.add_from_type(rhs);
            }
            Type::TypeReference { .. } => (),
            Type::Set { identifiers, .. } => {
                for identifier in identifiers {
                    if let Some(symbol) = Symbol::from_id(identifier, Flag::Member) {
                        self.symbols.push(symbol);
                    }
                }
            }
        }
    }

    fn add_if_not_defined(&mut self, symbol: Option<Symbol>) -> Option<usize> {
        symbol.map(|symbol| {
            defined(&self.symbols, &symbol.id, &symbol.flag).unwrap_or_else(|| {
                self.symbols.push(symbol);
                self.symbols.len() - 1
            })
        })
    }

    fn sym_from_param(param: &Identifier, owners: Vec<usize>) -> Option<Symbol> {
        if param.is_none() {
            None
        } else {
            let id = param.identifier.clone();
            let pos = param.span();
            Some(Symbol::new(id, pos, Flag::Param, Some(owners)))
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

    fn add_from_edge(&mut self, edge: &Edge<Identifier>) {
        if let [NodePart::Literal {
            identifier: left_id,
        }, left_binds @ ..] = edge.lhs.parts.as_slice()
        {
            if let [NodePart::Literal {
                identifier: right_id,
            }, right_binds @ ..] = edge.rhs.parts.as_slice()
            {
                let left_idx = self.add_if_not_defined(Symbol::from_id(left_id, Flag::Edge));
                let right_idx = self.add_if_not_defined(Symbol::from_id(right_id, Flag::Edge));

                // Split binds into literals and bindings
                let (left_binds, left_literals) = left_binds
                    .iter()
                    .partition::<Vec<&NodePart<Identifier>>, _>(|bind| {
                        matches!(bind, NodePart::Binding { .. })
                    });
                let (right_binds, right_literals) = right_binds
                    .iter()
                    .partition::<Vec<&NodePart<Identifier>>, _>(|bind| {
                        matches!(bind, NodePart::Binding { .. })
                    });

                // Maybe add literals as edge symbols
                for literal in &left_literals {
                    self.add_if_not_defined(Symbol::from_id(literal.identifier(), Flag::Edge));
                }
                for literal in &right_literals {
                    self.add_if_not_defined(Symbol::from_id(literal.identifier(), Flag::Edge));
                }

                // Splits bindings into common, left and right
                let (common_binds, left_binds) = left_binds
                    .iter()
                    .partition::<Vec<&NodePart<Identifier>>, _>(|bind| {
                        let id = bind.identifier().identifier.as_str();
                        right_binds
                            .iter()
                            .any(|right| right.identifier().identifier == id)
                    });
                let right_binds: Vec<&NodePart<Identifier>> = right_binds
                    .into_iter()
                    .filter(|bind| {
                        let id = bind.identifier().identifier.as_str();
                        !common_binds
                            .iter()
                            .any(|common| common.identifier().identifier == id)
                    })
                    .collect();
                // Maybe add left bindings as param symbols
                if let Some(left_idx) = left_idx {
                    for bind in &left_binds {
                        if self
                            .is_defined_param(bind.identifier().identifier.as_str(), left_idx)
                            .is_none()
                        {
                            self.edge_params.push(EdgeParam {
                                param: bind.identifier().clone(),
                                owners: HashSet::from([left_idx]),
                            });
                        }
                    }
                }
                // Maybe add right bindings as param symbols
                if let Some(right_idx) = right_idx {
                    for bind in &right_binds {
                        if self
                            .is_defined_param(bind.identifier().identifier.as_str(), right_idx)
                            .is_none()
                        {
                            self.edge_params.push(EdgeParam {
                                param: bind.identifier().clone(),
                                owners: HashSet::from([right_idx]),
                            });
                        }
                    }
                }

                // Merge common bindings
                if let (Some(left_idx), Some(right_idx)) = (left_idx, right_idx) {
                    for bind in &common_binds {
                        let left_param_idx =
                            self.is_defined_param(bind.identifier().identifier.as_str(), left_idx);
                        let right_param_idx =
                            self.is_defined_param(bind.identifier().identifier.as_str(), right_idx);
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
                                    param: bind.identifier().clone(),
                                    owners: HashSet::from([left_idx, right_idx]),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn from_game(game: &Game<Identifier>) -> Vec<Symbol> {
        let mut symbols: Self = Self {
            symbols: vec![],
            edge_params: vec![],
        };
        game.constants.iter().for_each(|constant| {
            let id = &constant.identifier;
            if let Some(symbol) = Symbol::from_id(id, Flag::Constant) {
                symbols.symbols.push(symbol);
            }
        });

        game.variables.iter().for_each(|variable| {
            let id = &variable.identifier;
            if let Some(symbol) = Symbol::from_id(id, Flag::Variable) {
                symbols.symbols.push(symbol);
            }
        });
        game.typedefs.iter().for_each(|typedef| {
            let id = &typedef.identifier;
            if let Some(symbol) = Symbol::from_id(id, Flag::Type) {
                symbols.symbols.push(symbol);
            }
            symbols.add_from_type(&typedef.type_);
        });
        game.edges
            .iter()
            .for_each(|edge| symbols.add_from_edge(edge));
        for bind in &symbols.edge_params {
            let id = &bind.param;
            let owners = bind.owners.iter().copied().collect();
            if let Some(symbol) = Self::sym_from_param(id, owners) {
                symbols.symbols.push(symbol);
            }
        }
        symbols.symbols
    }
}

pub fn from_game(game: &Game<Identifier>) -> Vec<Symbol> {
    Symbols::from_game(game)
}
