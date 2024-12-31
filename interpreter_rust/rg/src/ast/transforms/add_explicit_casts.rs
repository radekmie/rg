use crate::ast::{Edge, Error, ErrorReason, Expression, Game, Label, Type, Typedef};
use std::mem::{replace, take};
use std::sync::Arc;
use utils::position::Span;

impl<Id: Clone + PartialEq> Edge<Id> {
    fn add_explicit_casts(&mut self, game: &Game<Id>) -> Result<(), Error<Id>> {
        let mut label = replace(&mut self.label, Label::new_skip());
        label.add_explicit_casts(game, Some(self))?;
        self.label = label;
        Ok(())
    }
}

impl<Id: Clone + PartialEq> Label<Id> {
    fn add_explicit_casts(
        &mut self,
        game: &Game<Id>,
        edge: Option<&Edge<Id>>,
    ) -> Result<(), Error<Id>> {
        if let Self::Assignment { lhs, rhs } | Self::Comparison { lhs, rhs, .. } = self {
            let type_ = &lhs.infer(game, edge)?;
            if let Some(Typedef { identifier, .. }) = game.resolve_type_or_fail(type_)? {
                Arc::make_mut(lhs).add_explicit_casts_typedef(game, edge, identifier, None)?;
                Arc::make_mut(rhs).add_explicit_casts_typedef(game, edge, identifier, None)?;
            }
        }

        Ok(())
    }
}

impl<Id: Clone + PartialEq> Expression<Id> {
    fn add_explicit_casts_type(
        &mut self,
        game: &Game<Id>,
        edge: Option<&Edge<Id>>,
        type_: &Type<Id>,
    ) -> Result<(), Error<Id>> {
        if let Some(Typedef { identifier, .. }) = game.resolve_type_or_fail(type_)? {
            self.add_explicit_casts_typedef(game, edge, identifier, None)?;
        }

        Ok(())
    }

    fn add_explicit_casts_typedef(
        &mut self,
        game: &Game<Id>,
        edge: Option<&Edge<Id>>,
        cast_as: &Id,
        cast_to: Option<&Arc<Type<Id>>>,
    ) -> Result<(), Error<Id>> {
        match self {
            Self::Access { lhs, rhs, .. } => {
                let lhs_type = lhs.infer(game, edge)?;
                let Type::Arrow { lhs: key_type, .. } = lhs_type.resolve(game)? else {
                    return game.make_error(ErrorReason::ArrowTypeExpected { got: lhs_type });
                };

                Arc::make_mut(lhs).add_explicit_casts_type(game, edge, &lhs_type)?;
                Arc::make_mut(rhs).add_explicit_casts_type(game, edge, key_type)?;
            }
            Self::Cast { lhs, rhs, .. } => {
                let cast_to = Some(&*lhs).filter(|type_| type_.is_identifier(cast_as));
                Arc::make_mut(rhs).add_explicit_casts_typedef(game, edge, cast_as, cast_to)?;
            }
            Self::Reference { .. } => {}
        }

        if cast_to.is_none() && !self.is_cast_and(|lhs, _| lhs.is_identifier(cast_as)) {
            let identifier = cast_as.clone();
            *self = Self::Cast {
                span: Span::none(),
                lhs: Arc::new(Type::from(cast_as.clone())),
                rhs: Arc::new(replace(self, Self::Reference { identifier })),
            };
        }

        Ok(())
    }
}

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn add_explicit_casts(&mut self) -> Result<(), Error<Id>> {
        let mut edges = take(&mut self.edges);
        for edge in &mut edges {
            Arc::make_mut(edge).add_explicit_casts(self)?;
        }

        self.edges = edges;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        add_explicit_casts,
        reference_1,
        "type T = { 1 }; var t: T = 1; x, y: t == t;",
        "type T = { 1 }; var t: T = 1; x, y: T(t) == T(t);"
    );

    test_transform!(
        add_explicit_casts,
        reference_2,
        "type T = { 1 }; var t: T = 1; x, y: T(t) == t;",
        "type T = { 1 }; var t: T = 1; x, y: T(t) == T(t);"
    );

    test_transform!(
        add_explicit_casts,
        reference_3,
        "type T = { 1 }; var t: T = 1; x, y: t == T(t);",
        "type T = { 1 }; var t: T = 1; x, y: T(t) == T(t);"
    );

    test_transform!(
        add_explicit_casts,
        reference_4,
        "type T = { 1 }; var t: T = 1; x, y: T(t) == T(t);",
        "type T = { 1 }; var t: T = 1; x, y: T(t) == T(t);"
    );
}
