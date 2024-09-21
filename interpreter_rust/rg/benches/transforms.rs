use criterion::{criterion_group, criterion_main, Criterion};
use map_id::MapId;
use rg::ast::Game;
use rg::parsing::parser::parse_with_errors;
use std::fs::read_to_string;
use std::sync::Arc;
use std::time::Duration;

fn load(path: &str) -> Game<Arc<str>> {
    let source = read_to_string(format!("../../examples/{path}")).unwrap();
    let (game, errors) = parse_with_errors(&source);
    assert!(errors.is_empty(), "Parse errors: {errors:?}");
    let mut game = game.map_id(&mut |id| Arc::from(id.identifier.as_str()));
    game.add_builtins().unwrap();
    game
}

fn scenario(criterion: &mut Criterion, path: &str) {
    let game = load(path);
    let mut group = criterion.benchmark_group(format!("transforms/{path}"));
    group.measurement_time(Duration::from_millis(400));
    group.warm_up_time(Duration::from_millis(100));

    macro_rules! bench {
        ($fn:ident) => {
            group.bench_with_input(stringify!($fn), &game, |bencher, game| {
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
    bench!(calculate_repeats);
    bench!(calculate_simple_apply);
    bench!(calculate_tag_indexes);
    bench!(calculate_uniques);
    bench!(compact_skip_edges);
    bench!(expand_generator_nodes);
    bench!(inline_assignment);
    bench!(inline_reachability);
    bench!(join_fork_suffixes);
    bench!(mangle_symbols);
    bench!(normalize_types);
    bench!(prune_singleton_types);
    bench!(prune_unreachable_nodes);
    bench!(skip_generator_comparisons);
    bench!(skip_self_assignments);
    bench!(skip_self_comparisons);
    bench!(skip_unused_tags);
}

fn transforms(criterion: &mut Criterion) {
    scenario(criterion, "breakthrough.hrg.reuse.rg");
    scenario(criterion, "breakthrough.hrg.rg");
    scenario(criterion, "breakthrough.rbg.rg");
    scenario(criterion, "breakthrough.rg");
    scenario(criterion, "ticTacToe.kif.rg");
    scenario(criterion, "ticTacToe.rbg.rg");
    scenario(criterion, "ticTacToe.rg");
}

criterion_group!(benches, transforms);
criterion_main!(benches);
