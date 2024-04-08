use crate::ast::{Error, Game};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn expand_generator_nodes(&mut self) -> Result<(), Error<Arc<str>>> {
        for index in (0..self.edges.len()).rev() {
            let bindings = self.edges[index].bindings();
            if bindings.is_empty() {
                continue;
            }

            let mappings = self.create_mappings(bindings.into_iter())?;
            let item = self.edges.remove(index);
            self.edges.splice(
                index..index,
                mappings
                    .into_iter()
                    .map(|mapping| item.substitute_bindings(&mapping)),
            );
        }

        for index in 0..self.pragmas.len() {
            let bindings = self.pragmas[index].bindings();
            if bindings.is_empty() {
                continue;
            }

            let mappings = self.create_mappings(bindings.into_iter())?;
            self.pragmas[index].substitute_bindings_mut(&mappings);
        }

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
                actual.expand_generator_nodes().unwrap();
                let expect = parse($expect);

                assert_eq!(
                    actual, expect,
                    "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n"
                );
            }
        };
    }

    test!(
        edge,
        "type T = { a, b }; x(t: T), y: ;",
        "type T = { a, b }; x__bind__a, y: ; x__bind__b, y: ;"
    );

    test!(
        pragma1,
        "type T = { a, b }; @unique x(t: T);",
        "type T = { a, b }; @unique x__bind__a x__bind__b;"
    );

    test!(
        pragma2,
        "type T = { a, b }; @unique y(t1: T)(t2: T);",
        "type T = { a, b }; @unique y__bind__a__bind__a y__bind__a__bind__b y__bind__b__bind__a y__bind__b__bind__b;"
    );
}
