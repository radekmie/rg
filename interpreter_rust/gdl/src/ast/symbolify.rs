use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn symbolify(self) -> Self {
        Self(self.0.into_iter().map(Rule::symbolify).collect())
    }
}

impl Predicate<Arc<str>> {
    pub fn symbolify(self) -> Self {
        Self {
            is_negated: self.is_negated,
            term: Arc::new(self.term.symbolify()),
        }
    }
}

impl Rule<Arc<str>> {
    pub fn symbolify(self) -> Self {
        Self {
            term: Arc::new(self.term.symbolify()),
            predicates: self
                .predicates
                .into_iter()
                .map(Predicate::symbolify)
                .collect(),
        }
    }
}

impl Term<Arc<str>> {
    pub fn symbolify(&self) -> Self {
        use Term::{
            Base, Custom0, CustomN, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True,
        };
        self.maybe_symbolify().map_or_else(
            || match self {
                Base(proposition) => Base(Arc::new(proposition.symbolify())),
                Custom0(name) => Custom0(name.clone()),
                CustomN(name, arguments) => CustomN(
                    name.clone(),
                    arguments
                        .iter()
                        .map(|argument| Arc::new(argument.symbolify()))
                        .collect(),
                ),
                Does(role, action) => Does(role.clone(), Arc::new(action.symbolify())),
                Goal(role, utility) => Goal(role.clone(), utility.clone()),
                Init(proposition) => Init(Arc::new(proposition.symbolify())),
                Input(role, action) => Input(role.clone(), Arc::new(action.symbolify())),
                Legal(role, action) => Legal(role.clone(), Arc::new(action.symbolify())),
                Next(proposition) => Next(Arc::new(proposition.symbolify())),
                Role(role) => Role(role.clone()),
                Terminal => Terminal,
                True(proposition) => True(Arc::new(proposition.symbolify())),
            },
            |id| Custom0(AtomOrVariable::Atom(Arc::from(id))),
        )
    }

    // TODO: We do not handle name collisions yet, so `a(b(c))` and `a(b, c)` do
    // have the same symbolified form of `a_b_c`.
    fn maybe_symbolify(&self) -> Option<String> {
        match self {
            // Self::Custom(AtomOrVariable::Atom(id), _) if id.as_ref() == "distinct" => None,
            Self::Custom0(AtomOrVariable::Atom(id)) => Some(id.to_string()),
            Self::CustomN(AtomOrVariable::Atom(id), arguments) => {
                let mut symbolified_id = id.to_string();
                for argument in arguments {
                    symbolified_id.push('_');
                    symbolified_id.push_str(&argument.maybe_symbolify()?);
                }
                Some(symbolified_id)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;
    use crate::parser::game;
    use map_id::MapId;
    use nom::combinator::all_consuming;
    use std::sync::Arc;

    fn parse(input: &str) -> Game<Arc<str>> {
        all_consuming(game)(input)
            .unwrap()
            .1
            .map_id(&mut |id| Arc::from(*id))
    }

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = parse($actual).symbolify();
                let mut expect = parse($expect);

                // TODO: `&str` is not `Ord`.
                actual.0.sort_unstable_by_key(|x| format!("{x:?}"));
                expect.0.sort_unstable_by_key(|x| format!("{x:?}"));

                assert_eq!(actual.as_infix().to_string(), expect.as_infix().to_string());
            }
        };
    }

    test!(atom, "a", "a");
    test!(unary, "a(x)", "a_x");
    test!(binary, "a(x, y)", "a_x_y");
    test!(nested_1, "a(b(x), y)", "a_b_x_y");
    test!(nested_2, "a(x, c(y))", "a_x_c_y");
    test!(nested_3, "a(b(x), c(y))", "a_b_x_c_y");
    test!(collision_1, "a(b(c))", "a_b_c");
    test!(collision_2, "a(b, c)", "a_b_c");
}
