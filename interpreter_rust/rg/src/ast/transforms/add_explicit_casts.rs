use crate::ast::{Edge, EdgeLabel, Error, ErrorReason, Expression, Game, Type};
use crate::position::Span;
use std::mem::{replace, take};
use std::sync::Arc;

impl<Id: Clone + PartialEq> Edge<Id> {
    pub fn add_explicit_casts(&mut self, game: &Game<Id>) -> Result<(), Error<Id>> {
        let mut label = take(&mut self.label);
        label.add_explicit_casts(game, Some(self))?;
        self.label = label;
        Ok(())
    }
}

impl<Id: Clone + PartialEq> EdgeLabel<Id> {
    pub fn add_explicit_casts(
        &mut self,
        game: &Game<Id>,
        edge: Option<&Edge<Id>>,
    ) -> Result<(), Error<Id>> {
        if let Self::Assignment { lhs, rhs } | Self::Comparison { lhs, rhs, .. } = self {
            let type_ = &lhs.infer(game, edge)?;
            Arc::make_mut(lhs).add_explicit_casts(game, edge, type_)?;
            Arc::make_mut(rhs).add_explicit_casts(game, edge, type_)?;
        }

        Ok(())
    }
}

impl<Id: Clone + PartialEq> Expression<Id> {
    pub fn add_explicit_casts(
        &mut self,
        game: &Game<Id>,
        edge: Option<&Edge<Id>>,
        type_: &Type<Id>,
    ) -> Result<(), Error<Id>> {
        self.add_explicit_casts_inner(game, edge, type_, None)
    }

    fn add_explicit_casts_inner(
        &mut self,
        game: &Game<Id>,
        edge: Option<&Edge<Id>>,
        type_: &Type<Id>,
        cast: Option<&Arc<Type<Id>>>,
    ) -> Result<(), Error<Id>> {
        match self {
            Self::Access { lhs, rhs, .. } => {
                let lhs_type = lhs.infer(game, edge)?;
                let Type::Arrow { lhs: key_type, .. } = lhs_type.resolve(game)? else {
                    return game.make_error(ErrorReason::ArrowTypeExpected { got: lhs_type });
                };

                Arc::make_mut(lhs).add_explicit_casts_inner(game, edge, &lhs_type, None)?;
                Arc::make_mut(rhs).add_explicit_casts_inner(game, edge, key_type, None)?;
            }
            Self::Cast { lhs, rhs, .. } => {
                Arc::make_mut(rhs).add_explicit_casts_inner(game, edge, type_, Some(lhs))?;
            }
            Self::Reference { .. } => {}
        }

        if !cast.is_some_and(|cast| *type_ == **cast) {
            if let Some(typedef) = game.resolve_type_or_fail(type_)? {
                let identifier = &typedef.identifier;
                if !matches!(self, Self::Cast { lhs, .. } if lhs.is_reference(identifier)) {
                    let identifier = identifier.clone();
                    *self = Self::Cast {
                        span: Span::none(),
                        lhs: Arc::new(typedef.to_type()),
                        rhs: Arc::new(replace(self, Self::Reference { identifier })),
                    };
                }
            }
        }

        Ok(())
    }
}

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn add_explicit_casts(&mut self) -> Result<(), Error<Id>> {
        let mut edges = take(&mut self.edges);
        for edge in &mut edges {
            edge.add_explicit_casts(self)?;
        }

        self.edges = edges;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;
    use crate::parsing::parser::parse_with_errors;
    use map_id::MapId;
    use std::sync::Arc;

    fn parse(input: &str) -> Game<Arc<str>> {
        let (game, errors) = parse_with_errors(input);
        assert!(errors.is_empty(), "Parse errors: {errors:?}");
        game.map_id(&mut |id| Arc::from(id.identifier.as_str()))
    }

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = parse($actual);
                actual.add_explicit_casts().unwrap();
                let expect = parse($expect);

                assert_eq!(
                    actual, expect,
                    "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n"
                );
            }
        };
    }

    test!(
        reference_1,
        "type T = { 1 }; var t: T = 1; x, y: t == t;",
        "type T = { 1 }; var t: T = 1; x, y: T(t) == T(t);"
    );

    test!(
        reference_2,
        "type T = { 1 }; var t: T = 1; x, y: T(t) == t;",
        "type T = { 1 }; var t: T = 1; x, y: T(t) == T(t);"
    );

    test!(
        reference_3,
        "type T = { 1 }; var t: T = 1; x, y: t == T(t);",
        "type T = { 1 }; var t: T = 1; x, y: T(t) == T(t);"
    );

    test!(
        reference_4,
        "type T = { 1 }; var t: T = 1; x, y: T(t) == T(t);",
        "type T = { 1 }; var t: T = 1; x, y: T(t) == T(t);"
    );
}
