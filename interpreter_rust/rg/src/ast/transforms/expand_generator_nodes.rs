use crate::ast::{Error, Game};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn expand_generator_nodes(&mut self) -> Result<(), Error<Arc<str>>> {
        for index in (0..self.edges.len()).rev() {
            let mappings = self.create_mappings(self.edges[index].bindings())?;
            if !mappings.is_empty() {
                for mapping in &mappings {
                    self.edges
                        .push(self.edges[index].substitute_bindings(mapping));
                }

                self.edges.remove(index);
            }
        }

        // TODO: It does NOT handle the case there singular `edge_name` contains
        // bindings (only happens in `Disjoint` and `DisjointExhaustive`).
        for index in (0..self.pragmas.len()).rev() {
            let mappings = self.create_mappings(self.pragmas[index].bindings())?;
            if !mappings.is_empty() {
                for mapping in &mappings {
                    let edge_names = self.pragmas[index].edge_names_ref_mut();
                    for index in (0..edge_names.len()).rev() {
                        let edge_name = edge_names[index].substitute_bindings(mapping);
                        if !edge_names.contains(&edge_name) {
                            edge_names.insert(index, edge_name);
                        }
                    }
                }

                self.pragmas[index]
                    .edge_names_ref_mut()
                    .retain(|edge_name| !edge_name.has_bindings());
            }
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
