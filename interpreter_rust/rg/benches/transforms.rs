use criterion::{criterion_group, criterion_main, Criterion};
use map_id::MapId;
use rg::ast::Game;
use rg::parsing::parser::parse_with_errors;
use std::fs::read_to_string;
use std::sync::Arc;
use std::time::Duration;

fn load(path: &str) -> Game<Arc<str>> {
    let source = read_to_string(format!("../../games/{path}")).unwrap();
    let (game, errors) = parse_with_errors(&source);
    assert!(errors.is_empty(), "Parse errors: {errors:?}");
    let mut game = game.map_id(&mut |id| Arc::from(id.identifier.as_str()));
    game.add_builtins().unwrap();
    game
}

fn scenario(criterion: &mut Criterion, path: &str) {
    let mut group = criterion.benchmark_group(format!("transforms/{path}"));
    group.measurement_time(Duration::from_millis(400));
    group.warm_up_time(Duration::from_millis(100));

    let mut game = None;
    macro_rules! bench {
        ($fn:ident) => {
            group.bench_function(stringify!($fn), |bencher| {
                let game = game.get_or_insert_with(|| load(path));
                bencher.iter(|| {
                    if let Err(error) = game.clone().$fn() {
                        panic!("{error}");
                    }
                });
            });
        };
    }

    bench!(add_explicit_casts);
    bench!(calculate_disjoints);
    bench!(calculate_iterators);
    bench!(calculate_simple_apply);
    bench!(calculate_tag_indexes);
    bench!(compact_comparisons);
    bench!(compact_skip_edges);
    bench!(expand_assignment_any);
    bench!(inline_assignment);
    bench!(inline_reachability);
    bench!(join_exclusive_edges);
    bench!(join_fork_prefixes);
    bench!(join_fork_suffixes);
    bench!(mangle_symbols);
    bench!(merge_accesses);
    bench!(normalize_constants);
    bench!(normalize_types);
    bench!(propagate_constants);
    bench!(prune_singleton_types);
    bench!(prune_unreachable_nodes);
    bench!(prune_unused_constants);
    bench!(prune_unused_variables);
    bench!(skip_self_assignments);
    bench!(skip_self_comparisons);
    bench!(skip_unused_tags);
}

fn transforms(criterion: &mut Criterion) {
    scenario(criterion, "hrg/breakthrough.hrg.reuse.rg");
    scenario(criterion, "hrg/breakthrough.hrg.rg");
    scenario(criterion, "rbg/breakthrough.rbg.rg");
    scenario(criterion, "rg/breakthrough.rg");
    scenario(criterion, "hrg/connect4.hrg.reuse.rg");
    scenario(criterion, "hrg/connect4.hrg.rg");
    scenario(criterion, "kif/connect4.kif.rg");
    scenario(criterion, "rbg/connect4.rbg.rg");
    scenario(criterion, "hrg/ticTacToe.hrg.reuse.rg");
    scenario(criterion, "hrg/ticTacToe.hrg.rg");
    scenario(criterion, "kif/ticTacToe.kif.rg");
    scenario(criterion, "rbg/ticTacToe.rbg.rg");
    scenario(criterion, "rg/ticTacToe.rg");
}

criterion_group!(benches, transforms);
criterion_main!(benches);
