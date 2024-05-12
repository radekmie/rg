use crate::ast::analysis::ReachingPaths;
use crate::ast::{Error, Game, Pragma};
use std::collections::BTreeSet;
use std::sync::Arc;
use utils::position::Span;

impl Game<Arc<str>> {
    pub fn calculate_uniques(&mut self) -> Result<(), Error<Arc<str>>> {
        let reaching_paths = self.analyse::<ReachingPaths>();
        let mut unique_nodes: BTreeSet<_> = reaching_paths
            .into_iter()
            .filter(|(_, paths)| paths.iter().all(|path| !path.has_duplicate()))
            .map(|(node, _)| node)
            .collect();

        self.pragmas.retain(|pragma| {
            if let Pragma::Unique { nodes, .. } = pragma {
                unique_nodes.extend(nodes.iter().cloned());
                false
            } else {
                true
            }
        });

        let pragma = Pragma::Unique {
            span: Span::none(),
            nodes: unique_nodes.into_iter().collect(),
        };

        let index = self.pragmas.partition_point(|x| *x < pragma);
        self.pragmas.insert(index, pragma);

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
                actual.calculate_uniques().unwrap();

                assert_eq!(
                    actual, expect,
                    "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n"
                );
            }
        };
    }

    test!(
        small_unique,
        "begin, x: ; x, end: ;",
        "begin, x: ; x, end: ; @unique begin end x;"
    );

    test!(
        small_loop,
        "begin, x: ; x, y: ; y, x: ; y, end: ;",
        "begin, x: ; x, y: ; y, x: ; y, end: ; @unique begin;"
    );

    test!(
        tictactoe,
        include_str!("../../../../../examples/ticTacToe.rg"),
        concat!(
            "@unique begin check checkline checklineH1 checklineH2 checklineLR1 checklineLR2 checklineRL1 checklineRL2 checklineV1 checklineV2 checkwin chooseX chooseX(coordX: Coord) chooseY chooseY(coordY: Coord) end endcheckline endmove move nextturn preend set turn win win1 win2;",
            include_str!("../../../../../examples/ticTacToe.rg")
        )
    );
}
