use crate::ast::{Edge, EdgeLabel, Error, ErrorReason, Expression, Game, Type};
use std::rc::Rc;

impl<Id: Clone + PartialEq> Edge<Id> {
    pub fn add_explicit_casts(&self, game: &Game<Id>) -> Result<Self, Error<Id>> {
        Ok(Edge {
            label: self.label.add_explicit_casts(game, self)?,
            lhs: self.lhs.clone(),
            rhs: self.rhs.clone(),
        })
    }
}

impl<Id: Clone + PartialEq> EdgeLabel<Id> {
    pub fn add_explicit_casts(&self, game: &Game<Id>, edge: &Edge<Id>) -> Result<Self, Error<Id>> {
        Ok(match self {
            Self::Assignment { lhs, rhs } => {
                let type_ = &game.infer_expression(edge, lhs)?;
                Self::Assignment {
                    lhs: Rc::new(lhs.add_explicit_casts(game, edge, type_)?),
                    rhs: Rc::new(rhs.add_explicit_casts(game, edge, type_)?),
                }
            }
            Self::Comparison { lhs, rhs, negated } => {
                let type_ = &game.infer_expression(edge, lhs)?;
                Self::Comparison {
                    lhs: Rc::new(lhs.add_explicit_casts(game, edge, type_)?),
                    rhs: Rc::new(rhs.add_explicit_casts(game, edge, type_)?),
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
        game: &Game<Id>,
        edge: &Edge<Id>,
        type_: &Type<Id>,
    ) -> Result<Self, Error<Id>> {
        let self_with_casts_in_subexpressions = match self {
            Self::Access { lhs, rhs } => {
                let lhs_type = game.infer_expression(edge, lhs)?;
                match game.resolve_type_reference(&lhs_type)? {
                    Type::Arrow { lhs: key_type, .. } => {
                        let key_type = &game.resolve_type(key_type)?.type_;
                        Self::Access {
                            lhs: Rc::new(lhs.add_explicit_casts(game, edge, &lhs_type)?),
                            rhs: Rc::new(rhs.add_explicit_casts(game, edge, key_type)?),
                        }
                    }
                    _ => return game.make_error(ErrorReason::ArrowTypeExpected { got: lhs_type }),
                }
            }
            Self::Cast { lhs, rhs } => Self::Cast {
                lhs: lhs.clone(),
                rhs: Rc::new(rhs.add_explicit_casts(game, edge, type_)?),
            },
            Self::Reference { identifier } => Self::Reference {
                identifier: identifier.clone(),
            },
        };

        let self_with_casts = match game.resolve_typedef(type_)? {
            Some(typedef) => Self::Cast {
                lhs: Rc::new(Type::TypeReference {
                    identifier: typedef.identifier.clone(),
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

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn add_explicit_casts(mut self) -> Result<Self, Error<Id>> {
        self.edges = self
            .edges
            .iter()
            .map(|edge| edge.add_explicit_casts(&self))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(self)
    }
}
