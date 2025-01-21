use crate::common;
use crate::common::symbol::{defined, Flag, Symbol};
use rg::ast::{Edge, Game, NodePart, Type};
use std::{collections::HashSet, sync::Arc};
use utils::{position::Positioned, Identifier};

struct EdgeParam {
    param: Identifier,
    type_: Arc<Type<Identifier>>,
    owners: HashSet<usize>,
}

pub struct Symbols {
    symbols: Vec<Symbol>,
    edge_params: Vec<EdgeParam>,
}

impl Symbols {
    fn add_from_type(&mut self, type_: &Type<Identifier>, sym_type: Arc<Type<Identifier>>) {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                self.add_from_type(lhs, sym_type.clone());
                self.add_from_type(rhs, sym_type);
            }
            Type::TypeReference { .. } => (),
            Type::Set { identifiers, .. } => {
                for identifier in identifiers {
                    if let Some(symbol) = typed(identifier, Flag::Member, sym_type.clone()) {
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

    fn sym_from_param(
        param: &Identifier,
        owners: Vec<usize>,
        type_: Arc<Type<Identifier>>,
    ) -> Option<Symbol> {
        if param.is_none() || param.is_numeric() {
            None
        } else {
            let id = param.identifier.clone();
            let pos = param.span();
            let type_ = common::symbol::Type::Rg(type_);
            Some(Symbol::new(id, pos, Flag::Param, Some(owners), type_))
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
                let left_idx = self.add_if_not_defined(untyped(left_id, Flag::Function));
                let right_idx = self.add_if_not_defined(untyped(right_id, Flag::Function));

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
                    self.add_if_not_defined(untyped(literal.identifier(), Flag::Function));
                }
                for literal in &right_literals {
                    self.add_if_not_defined(untyped(literal.identifier(), Flag::Function));
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
                                type_: bind.type_().unwrap(),
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
                                type_: bind.type_().unwrap(),
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
                                    let left_span = self.edge_params[left_param_idx].param.span();
                                    let right_span = self.edge_params[right_param_idx].param.span();
                                    if left_span < right_span {
                                        let right_params =
                                            self.edge_params[right_param_idx].owners.clone();
                                        self.edge_params[left_param_idx]
                                            .owners
                                            .extend(right_params);
                                        self.edge_params.remove(right_param_idx);
                                    } else {
                                        let left_params =
                                            self.edge_params[left_param_idx].owners.clone();
                                        self.edge_params[right_param_idx]
                                            .owners
                                            .extend(left_params);
                                        self.edge_params.remove(left_param_idx);
                                    }
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
                                    type_: bind.type_().unwrap(),
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
            if let Some(symbol) = typed(id, Flag::Constant, constant.type_.clone()) {
                symbols.symbols.push(symbol);
            }
        });

        game.variables.iter().for_each(|variable| {
            let id = &variable.identifier;
            if let Some(symbol) = typed(id, Flag::Variable, variable.type_.clone()) {
                symbols.symbols.push(symbol);
            }
        });
        game.typedefs.iter().for_each(|typedef| {
            let id = &typedef.identifier;
            if let Some(symbol) = untyped(id, Flag::Type) {
                symbols.symbols.push(symbol);
            }
            let sym_type = Arc::new(Type::new(id.clone()));
            symbols.add_from_type(&typedef.type_, sym_type);
        });
        game.edges
            .iter()
            .for_each(|edge| symbols.add_from_edge(edge));
        for bind in &symbols.edge_params {
            let id = &bind.param;
            let owners = bind.owners.iter().copied().collect();
            if let Some(symbol) = Self::sym_from_param(id, owners, bind.type_.clone()) {
                symbols.symbols.push(symbol);
            }
        }
        symbols.symbols
    }
}

fn typed(identifier: &Identifier, flag: Flag, type_: Arc<Type<Identifier>>) -> Option<Symbol> {
    Symbol::from_id(identifier, flag, common::symbol::Type::Rg(type_))
}

fn untyped(identifier: &Identifier, flag: Flag) -> Option<Symbol> {
    Symbol::from_id(identifier, flag, common::symbol::Type::NoType)
}
