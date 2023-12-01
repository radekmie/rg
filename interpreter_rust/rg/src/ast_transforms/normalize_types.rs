use crate::ast::{Constant, Error, Game, Type, Typedef, Variable};
use crate::position::Span;
use std::sync::Arc;

impl Constant<Arc<str>> {
    pub fn normalize_type(&self, game: &mut Game<Arc<str>>) -> Result<Self, Error<Arc<str>>> {
        Ok(Self {
            span: Span::none(),
            identifier: self.identifier.clone(),
            type_: Arc::new(self.type_.normalize(game)?),
            value: self.value.clone(),
        })
    }
}

impl Game<Arc<str>> {
    pub fn normalize_types(&mut self) -> Result<(), Error<Arc<str>>> {
        for (index, typedef) in self.typedefs.clone().into_iter().enumerate() {
            self.typedefs[index] = typedef.normalize_type(self)?;
        }

        for (index, constant) in self.constants.clone().into_iter().enumerate() {
            self.constants[index] = constant.normalize_type(self)?;
        }

        for (index, variable) in self.variables.clone().into_iter().enumerate() {
            self.variables[index] = variable.normalize_type(self)?;
        }

        Ok(())
    }
}

impl Type<Arc<str>> {
    pub fn normalize(&self, game: &mut Game<Arc<str>>) -> Result<Self, Error<Arc<str>>> {
        if matches!(self, Self::TypeReference { .. }) {
            return Ok(self.clone());
        }

        let self_normalized = self.normalize_direct(game)?;
        if let Some(typedef) = game.resolve_type_or_fail(&self_normalized)? {
            return Ok(Self::TypeReference {
                identifier: typedef.identifier.clone(),
            });
        }

        let mut index = 1;
        let identifier = loop {
            let identifier = Arc::from(format!("Type{}", index));
            if !game
                .typedefs
                .iter()
                .any(|typedef| typedef.identifier == identifier)
            {
                break identifier;
            }

            index += 1;
        };

        game.typedefs.push(Typedef {
            span: Span::none(),
            identifier: identifier.clone(),
            type_: Arc::new(self_normalized),
        });

        Ok(Self::TypeReference { identifier })
    }

    fn normalize_direct(&self, game: &mut Game<Arc<str>>) -> Result<Self, Error<Arc<str>>> {
        let Self::Arrow { lhs, rhs } = self else {
            return Ok(self.clone());
        };
        Ok(Self::Arrow {
            lhs: Arc::new(lhs.normalize(game)?),
            rhs: Arc::new(rhs.normalize(game)?),
        })
    }
}

impl Typedef<Arc<str>> {
    pub fn normalize_type(&self, game: &mut Game<Arc<str>>) -> Result<Self, Error<Arc<str>>> {
        Ok(Self {
            span: Span::none(),
            identifier: self.identifier.clone(),
            type_: Arc::new(self.type_.normalize_direct(game)?),
        })
    }
}

impl Variable<Arc<str>> {
    pub fn normalize_type(&self, game: &mut Game<Arc<str>>) -> Result<Self, Error<Arc<str>>> {
        Ok(Self {
            span: Span::none(),
            default_value: self.default_value.clone(),
            identifier: self.identifier.clone(),
            type_: Arc::new(self.type_.normalize(game)?),
        })
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
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        game.map_id(&mut |id| Arc::from(id.identifier.as_str()))
    }

    macro_rules! test {
        ($name:ident { $($actual:tt)* } { $($expect:tt)* }) => {
            #[test]
            fn $name() {
                let mut actual = parse(stringify!($($actual)*));
                actual.normalize_types().unwrap();
                let expect = parse(stringify!($($expect)*));

                assert_eq!(actual, expect, "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n");
            }
        };
    }

    test!(
        set
        { type X = { a }; }
        { type X = { a }; }
    );

    test!(
        arrow2
        { type X = { a } -> { b }; }
        {
            type X = Type1 -> Type2;
            type Type1 = { a };
            type Type2 = { b };
        }
    );

    test!(
        arrow3
        { type X = { a } -> { b } -> { c }; }
        {
            type X = Type1 -> Type4;
            type Type1 = { a };
            type Type2 = { b };
            type Type3 = { c };
            type Type4 = Type2 -> Type3;
        }
    );

    test!(
        arrow4
        { type X = { a } -> { b } -> { c } -> { d }; }
        {
            type X = Type1 -> Type6;
            type Type1 = { a };
            type Type2 = { b };
            type Type3 = { c };
            type Type4 = { d };
            type Type5 = Type3 -> Type4;
            type Type6 = Type2 -> Type5;
        }
    );
}
