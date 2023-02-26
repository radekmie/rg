use crate::ast::{
    EdgeDeclaration, EdgeLabel, Error, ErrorReason, Expression, GameDeclaration, Type,
};
use std::rc::Rc;

impl<Id: Clone + PartialEq> EdgeDeclaration<Id> {
    pub fn add_explicit_casts(
        &self,
        game_declaration: &GameDeclaration<Id>,
    ) -> Result<Self, Error<Id>> {
        Ok(EdgeDeclaration {
            label: Rc::new(self.label.add_explicit_casts(game_declaration, self)?),
            lhs: self.lhs.clone(),
            rhs: self.rhs.clone(),
        })
    }
}

impl<Id: Clone + PartialEq> EdgeLabel<Id> {
    pub fn add_explicit_casts(
        &self,
        game_declaration: &GameDeclaration<Id>,
        edge_declaration: &EdgeDeclaration<Id>,
    ) -> Result<Self, Error<Id>> {
        Ok(match self {
            Self::Assignment { lhs, rhs } => {
                let type_ = &game_declaration.infer_expression(edge_declaration, lhs)?;
                Self::Assignment {
                    lhs: Rc::new(lhs.add_explicit_casts(
                        game_declaration,
                        edge_declaration,
                        type_,
                    )?),
                    rhs: Rc::new(rhs.add_explicit_casts(
                        game_declaration,
                        edge_declaration,
                        type_,
                    )?),
                }
            }
            Self::Comparison { lhs, rhs, negated } => {
                let type_ = &game_declaration.infer_expression(edge_declaration, lhs)?;
                Self::Comparison {
                    lhs: Rc::new(lhs.add_explicit_casts(
                        game_declaration,
                        edge_declaration,
                        type_,
                    )?),
                    rhs: Rc::new(rhs.add_explicit_casts(
                        game_declaration,
                        edge_declaration,
                        type_,
                    )?),
                    negated: *negated,
                }
            }
            _ => self.clone(),
        })
    }
}

impl<Id: Clone + PartialEq> Expression<Id> {
    pub fn add_explicit_casts(
        &self,
        game_declaration: &GameDeclaration<Id>,
        edge_declaration: &EdgeDeclaration<Id>,
        type_: &Type<Id>,
    ) -> Result<Self, Error<Id>> {
        let self_with_casts_in_subexpressions = match self {
            Self::Access { lhs, rhs } => {
                let lhs_type = game_declaration.infer_expression(edge_declaration, lhs)?;
                match game_declaration.resolve_type_reference(&lhs_type)? {
                    Type::Arrow { lhs: key_type, .. } => {
                        let key_type = &game_declaration.resolve_type(key_type)?.type_;
                        Self::Access {
                            lhs: Rc::new(lhs.add_explicit_casts(
                                game_declaration,
                                edge_declaration,
                                &lhs_type,
                            )?),
                            rhs: Rc::new(rhs.add_explicit_casts(
                                game_declaration,
                                edge_declaration,
                                key_type,
                            )?),
                        }
                    }
                    _ => {
                        return game_declaration
                            .make_error(ErrorReason::ArrowTypeExpected { got: lhs_type })
                    }
                }
            }
            Self::Cast { lhs, rhs } => Self::Cast {
                lhs: lhs.clone(),
                rhs: Rc::new(rhs.add_explicit_casts(game_declaration, edge_declaration, type_)?),
            },
            Self::Reference { identifier } => Self::Reference {
                identifier: identifier.clone(),
            },
        };

        let self_with_casts = match game_declaration.resolve_type_declaration(type_)? {
            Some(type_declaration) => Self::Cast {
                lhs: Rc::new(Type::TypeReference {
                    identifier: type_declaration.identifier.clone(),
                }),
                rhs: Rc::new(self_with_casts_in_subexpressions),
            },
            _ => self_with_casts_in_subexpressions,
        };

        let mut self_with_casts_deduplicated = self_with_casts;
        loop {
            match self_with_casts_deduplicated {
                Self::Cast { lhs, mut rhs } if matches!(&*rhs, Self::Cast { lhs: rhs_lhs, .. } if &lhs == rhs_lhs) =>
                {
                    self_with_casts_deduplicated = Rc::make_mut(&mut rhs).clone();
                }
                _ => return Ok(self_with_casts_deduplicated),
            }
        }
    }
}

impl<Id: Clone + PartialEq> GameDeclaration<Id> {
    pub fn add_explicit_casts(mut self) -> Result<Self, Error<Id>> {
        self.edges = self
            .edges
            .iter()
            .map(|edge_declaration| edge_declaration.add_explicit_casts(&self))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Rc::new)
            .collect();

        Ok(self)
    }
}
