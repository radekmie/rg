mod add_builtins;
mod add_explicit_casts;
mod calculate_disjoints;
mod calculate_repeats;
mod calculate_simple_apply;
mod calculate_tag_indexes;
mod calculate_uniques;
mod compact_comparisons;
mod compact_skip_edges;
mod expand_generator_nodes;
mod inline_assignment;
mod inline_reachability;
mod join_exclusive_edges;
mod join_fork_prefixes;
mod join_fork_suffixes;
mod join_generators;
mod mangle_symbols;
mod merge_accesses;
mod normalize_constants;
mod normalize_types;
mod propagate_constants;
mod prune_singleton_types;
mod prune_unreachable_nodes;
mod prune_unused_bindings;
mod prune_unused_constants;
mod prune_unused_variables;
mod skip_generator_comparisons;
mod skip_self_assignments;
mod skip_self_comparisons;
mod skip_unused_tags;

use super::Node;
use std::collections::BTreeSet;
use std::sync::Arc;

pub fn gen_fresh_node(max_node_id: &mut usize) -> Node<Arc<str>> {
    *max_node_id += 1;
    Node::new(Arc::from(max_node_id.to_string()))
}

pub fn max_node_id(nodes: &BTreeSet<&Node<Arc<str>>>) -> usize {
    nodes
        .iter()
        .map(|node| node.literal().parse::<usize>().unwrap_or(0))
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod test {
    #[macro_export]
    macro_rules! test_transform {
        ($fn:ident, $name:ident, $actual:expr) => {
            test_transform!($fn, $name, $actual, $actual);
        };

        ($fn:ident, $name:ident, $actual:expr, adds $extra:expr) => {
            test_transform!($fn, $name, $actual, concat!($actual, $extra));
        };

        ($fn:ident, $name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                use map_id::MapId;
                use std::sync::Arc;
                use $crate::ast::Game;
                use $crate::parsing::parser::parse_with_errors;

                fn parse(source: &str) -> Game<Arc<str>> {
                    let (mut game, errors) = parse_with_errors(source);
                    assert!(errors.is_empty(), "Parse errors: {errors:?}");
                    game.pragmas.sort_unstable();
                    game.map_id(&mut |id| Arc::from(id.identifier.as_str()))
                }

                let mut actual = parse($actual);
                let expect = parse($expect);
                actual.$fn().unwrap();

                // `assert_eq` prints the entire structs and it's not helpful.
                assert!(
                    actual == expect,
                    "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n"
                );
            }
        };
    }
}
