mod add_builtins;
mod add_explicit_casts;
mod calculate_simple_apply;
mod calculate_tag_indexes;
mod calculate_uniques;
mod compact_skip_edges;
mod expand_generator_nodes;
mod inline_assignment;
mod inline_reachability;
mod join_fork_prefixes;
mod join_fork_suffixes;
mod mangle_symbols;
mod normalize_types;
mod prune_singleton_types;
mod prune_unreachable_nodes;
mod prune_unused_constants;
mod prune_unused_variables;
mod skip_generator_comparisons;
mod skip_self_assignments;
mod skip_self_comparisons;
mod skip_unused_tags;

#[cfg(test)]
mod test {
    #[macro_export]
    macro_rules! test_transform {
        ($fn:ident, $name:ident, $actual:expr) => {
            test_transform!($fn, $name, $actual, $actual);
        };

        ($fn:ident, $name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                use crate::ast::Game;
                use crate::parsing::parser::parse_with_errors;
                use map_id::MapId;
                use std::sync::Arc;

                fn parse(source: &str) -> Game<Arc<str>> {
                    let (game, errors) = parse_with_errors(source);
                    assert!(errors.is_empty(), "Parse errors: {errors:?}");
                    game.map_id(&mut |id| Arc::from(id.identifier.as_str()))
                }

                let mut actual = parse($actual);
                let expect = parse($expect);
                actual.$fn().unwrap();

                assert_eq!(
                    actual, expect,
                    "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n"
                );
            }
        };
    }
}
