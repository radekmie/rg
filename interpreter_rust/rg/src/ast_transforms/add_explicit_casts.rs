use crate::ast::{Edge, EdgeLabel, Error, ErrorReason, Expression, Game, Type};
use crate::position::Span;
use std::sync::Arc;

impl<Id: Clone + PartialEq> Edge<Id> {
    pub fn add_explicit_casts(&self, game: &Game<Id>) -> Result<Self, Error<Id>> {
        Ok(Edge {
            span: Span::none(),
            label: self.label.add_explicit_casts(game, Some(self))?,
            lhs: self.lhs.clone(),
            rhs: self.rhs.clone(),
        })
    }
}

impl<Id: Clone + PartialEq> EdgeLabel<Id> {
    pub fn add_explicit_casts(
        &self,
        game: &Game<Id>,
        edge: Option<&Edge<Id>>,
    ) -> Result<Self, Error<Id>> {
        Ok(match self {
            Self::Assignment { lhs, rhs } => {
                let type_ = &lhs.infer(game, edge)?;
                Self::Assignment {
                    lhs: Arc::new(lhs.add_explicit_casts(game, edge, type_)?),
                    rhs: Arc::new(rhs.add_explicit_casts(game, edge, type_)?),
                }
            }
            Self::Comparison { lhs, rhs, negated } => {
                let type_ = &lhs.infer(game, edge)?;
                Self::Comparison {
                    lhs: Arc::new(lhs.add_explicit_casts(game, edge, type_)?),
                    rhs: Arc::new(rhs.add_explicit_casts(game, edge, type_)?),
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
        edge: Option<&Edge<Id>>,
        type_: &Type<Id>,
    ) -> Result<Self, Error<Id>> {
        self.add_explicit_casts_in_subexpressions(game, edge, type_)?
            .add_explicit_cast(game, type_)
    }

    fn add_explicit_cast(self, game: &Game<Id>, type_: &Type<Id>) -> Result<Self, Error<Id>> {
        let Some(typedef) = game.resolve_type_or_fail(type_)? else {
            return Ok(self);
        };
        Ok(self.add_explicit_cast_if_needed(&Arc::new(typedef.to_type())))
    }

    fn add_explicit_cast_if_needed(self, type_: &Arc<Type<Id>>) -> Self {
        match self {
            Self::Cast { ref lhs, .. } if lhs == type_ => self,
            _ => Self::Cast {
                span: Span::none(),
                lhs: type_.clone(),
                rhs: Arc::new(self),
            },
        }
    }

    fn add_explicit_casts_in_subexpressions(
        &self,
        game: &Game<Id>,
        edge: Option<&Edge<Id>>,
        type_: &Type<Id>,
    ) -> Result<Self, Error<Id>> {
        Ok(match self {
            Self::Access { lhs, rhs, .. } => {
                let lhs_type = lhs.infer(game, edge)?;
                let Type::Arrow { lhs: key_type, .. } = lhs_type.resolve(game)? else {
                    return game.make_error(ErrorReason::ArrowTypeExpected { got: lhs_type });
                };

                Self::Access {
                    span: Span::none(),
                    lhs: Arc::new(lhs.add_explicit_casts(game, edge, &lhs_type)?),
                    rhs: Arc::new(rhs.add_explicit_casts(game, edge, key_type)?),
                }
            }
            Self::Cast { ref lhs, rhs, .. } => rhs
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
