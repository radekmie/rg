use crate::ast::analysis::ReachableNodes;
use crate::ast::{Error, Game};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn skip_unused_tags(&mut self) -> Result<(), Error<Arc<str>>> {
        let reaching_paths = self.analyse::<ReachableNodes>();
        for edge in &mut self.edges {
            if edge.label.is_tag()
                && !reaching_paths
                    .get(&edge.lhs)
                    .is_some_and(|reachable| *reachable)
            {
                edge.skip();
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
                let expect = parse($expect);
                actual.skip_unused_tags().unwrap();

                assert_eq!(
                    actual, expect,
                    "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n"
                );
            }
        };
    }

    test!(
        small,
        "begin, end: ;
        t1, t2: $ 1;
        t2, t3: $ 2;",
        "begin, end: ;
        t1, t2: ;
        t2, t3: ;"
    );

    test!(
        reachability,
        "begin, end: ? t1 -> t2;
        t1, t2: $ 1;
        t2, t3: $ 2;",
        "begin, end: ? t1 -> t2;
        t1, t2: ;
        t2, t3: ;"
    );

    test!(
        used_tag,
        "begin, t2: ? t1 -> t2;
        t1, t2: $ 1;
        t2, t3: $ 2;
        t3, end: ;",
        "begin, t2: ? t1 -> t2;
        t1, t2: ;
        t2, t3: $ 2;
        t3, end: ;"
    );
}
