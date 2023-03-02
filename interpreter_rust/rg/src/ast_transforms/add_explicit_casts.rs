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
        self.add_explicit_casts_in_subexpressions(game, edge, type_)?
            .add_explicit_cast(game, type_)
    }

    fn add_explicit_cast(self, game: &Game<Id>, type_: &Type<Id>) -> Result<Self, Error<Id>> {
        Ok(match game.resolve_typedef_by_type(type_)? {
            Some(typedef) => self.add_explicit_cast_if_needed(&Rc::new(Type::TypeReference {
                identifier: typedef.identifier.clone(),
            })),
            _ => self,
        })
    }

    fn add_explicit_cast_if_needed(self, type_: &Rc<Type<Id>>) -> Self {
        match self {
            Self::Cast { ref lhs, .. } if lhs == type_ => self,
            _ => Self::Cast {
                lhs: type_.clone(),
                rhs: Rc::new(self),
            },
        }
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
            Self::Cast { ref lhs, rhs } => rhs
                .add_explicit_casts(game, edge, type_)?
                .add_explicit_cast_if_needed(lhs),
            Self::Reference { identifier } => Self::Reference {
                identifier: identifier.clone(),
            },
        })
    }
}

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn add_explicit_casts(&mut self) -> Result<(), Error<Id>> {
        self.edges = self
            .edges
            .iter()
            .map(|edge| edge.add_explicit_casts(self))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }
}
