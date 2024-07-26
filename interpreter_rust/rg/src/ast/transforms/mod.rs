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
mod join_fork_prefixes;
mod join_fork_suffixes;
mod mangle_symbols;
mod normalize_types;
mod propagate_constants;
mod prune_singleton_types;
mod prune_unreachable_nodes;
mod prune_unused_constants;
mod prune_unused_variables;
mod skip_generator_comparisons;
mod skip_self_assignments;
mod skip_self_comparisons;
mod skip_unused_tags;

use super::Node;
use std::collections::BTreeSet;
use std::sync::Arc;

#[allow(clippy::needless_pass_by_value)]
pub fn gen_fresh_node(suffix: String, nodes: &BTreeSet<Node<Arc<str>>>) -> Node<Arc<str>> {
    for x in 1..nodes.len() {
        let fresh_node: Node<Arc<str>> = Node::new(Arc::from(format!("__gen_{x}_{suffix}")));
        if !nodes.contains(&fresh_node) {
            return fresh_node;
        }
    }
    let name = format!("__gen_{}_{suffix}", nodes.len());
    Node::new(Arc::from(name))
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
                use crate::ast::Game;
                use crate::parsing::parser::parse_with_errors;
                use map_id::MapId;
                use std::sync::Arc;

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
