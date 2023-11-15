use crate::ast::{Error, Game};
use std::rc::Rc;

impl Game<Rc<str>> {
    pub fn expand_generator_nodes(&mut self) -> Result<(), Error<Rc<str>>> {
        macro_rules! substitute_bindings {
            ($list:expr) => {
                for index in (0..$list.len()).rev() {
                    let mappings = self.create_mappings($list[index].bindings())?;
                    if !mappings.is_empty() {
                        for mapping in &mappings {
                            $list.push($list[index].substitute_bindings(mapping));
                        }

                        $list.remove(index);
                    }
                }
            };
        }

        substitute_bindings!(self.edges);
        substitute_bindings!(self.pragmas);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;
    use crate::parser::parse_with_errors;
    use map_id::MapId;
    use std::rc::Rc;

    fn parse(input: &str) -> Game<Rc<str>> {
        let (game, errors) = parse_with_errors(input);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        game.map_id(&mut |id| Rc::from(id.identifier.as_str()))
    }

    macro_rules! test {
        ($name:ident { $($actual:tt)* } { $($expect:tt)* }) => {
            #[test]
            fn $name() {
                let mut actual = parse(stringify!($($actual)*));
                actual.expand_generator_nodes().unwrap();
                let expect = parse(stringify!($($expect)*));

                assert_eq!(actual, expect, "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n");
            }
        };
    }

    test!(
        edge
        { type T = { a, b }; x(t: T), y: ; }
        { type T = { a, b }; x__bind__a, y: ; x__bind__b, y: ; }
    );

    test!(
        pragma
        { type T = { a, b }; @unique x(t: T); }
        { type T = { a, b }; @unique x__bind__a; @unique x__bind__b; }
    );
}
