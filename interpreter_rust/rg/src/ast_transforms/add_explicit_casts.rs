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
                let type_ = &lhs.infer(game, edge)?;
                Self::Assignment {
                    lhs: Rc::new(lhs.add_explicit_casts(game, edge, type_)?),
                    rhs: Rc::new(rhs.add_explicit_casts(game, edge, type_)?),
                }
            }
            Self::Comparison { lhs, rhs, negated } => {
                let type_ = &lhs.infer(game, edge)?;
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
        Ok(self
            .add_explicit_casts_in_subexpressions(game, edge, type_)?
            .add_explicit_cast(game, type_)?
            .deduplicate_casts())
    }

    fn add_explicit_cast(self, game: &Game<Id>, type_: &Type<Id>) -> Result<Self, Error<Id>> {
        Ok(match game.resolve_typedef_by_type(type_)? {
            Some(typedef) => Self::Cast {
                lhs: Rc::new(Type::TypeReference {
                    identifier: typedef.identifier.clone(),
                }),
                rhs: Rc::new(self),
            },
            _ => self,
        })
    }

    fn add_explicit_casts_in_subexpressions(
        &self,
        game: &Game<Id>,
        edge: &Edge<Id>,
        type_: &Type<Id>,
    ) -> Result<Self, Error<Id>> {
        Ok(match self {
            Self::Access { lhs, rhs } => {
                let lhs_type = lhs.infer(game, edge)?;
                match lhs_type.resolve(game)? {
                    Type::Arrow { lhs: key_type, .. } => {
                        let key_type = &game.resolve_typedef(key_type)?.type_;
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
        })
    }

    fn deduplicate_casts(self) -> Self {
        match self {
            Self::Cast { lhs, mut rhs } if matches!(&*rhs, Self::Cast { lhs: rhs_lhs, .. } if &lhs == rhs_lhs) => {
                Rc::make_mut(&mut rhs).clone().deduplicate_casts()
            }
            _ => self,
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
