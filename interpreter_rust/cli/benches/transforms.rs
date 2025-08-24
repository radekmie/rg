use cli::{analyze, Flags};
use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use std::fs::read_to_string;
use std::time::Duration;

pub fn scenario(group: &mut BenchmarkGroup<'_, WallTime>, path: &str) {
    let extension = path.split('.').last().unwrap();
    let source = read_to_string(format!("../../games/{path}")).unwrap();
    let mut cache = None;
    macro_rules! bench {
        ($flag:ident) => {
            group.bench_function(BenchmarkId::new(stringify!($flag), path), |bencher| {
                let game = cache.get_or_insert_with(|| {
                    analyze(
                        source.clone(),
                        extension,
                        &Flags::default(),
                        false,
                        &mut None::<fn(_)>,
                    )
                    .unwrap()
                });
                bencher.iter(|| game.clone().$flag().unwrap());
            });
        };
    }

    bench!(add_explicit_casts);
    bench!(calculate_disjoints);
    bench!(calculate_iterators);
    bench!(calculate_repeats_and_uniques);
    bench!(calculate_simple_apply);
    bench!(calculate_tag_indexes);
    bench!(compact_comparisons);
    bench!(compact_reachability);
    bench!(compact_skip_edges);
    bench!(expand_assignment_any);
    bench!(expand_tag_variable);
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
    bench!(prune_self_loops);
    bench!(prune_singleton_types);
    bench!(prune_unreachable_nodes);
    bench!(prune_unused_constants);
    bench!(prune_unused_variables);
    bench!(reorder_conditions);
    bench!(skip_artificial_tags);
    bench!(skip_redundant_tags);
    bench!(skip_self_assignments);
    bench!(skip_self_comparisons);
    bench!(skip_unused_tags);
}

fn transforms(criterion: &mut Criterion) {
    // Minimum benchmark time just to get some sense about the scale.
    let mut group = criterion.benchmark_group("transforms");
    group.measurement_time(Duration::from_millis(1));
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(1));

    scenario(&mut group, "hrg/breakthrough.hrg");
    scenario(&mut group, "rbg/breakthrough.rbg");
    scenario(&mut group, "rg/breakthrough.rg");
    scenario(&mut group, "hrg/connect4.hrg");
    scenario(&mut group, "kif/connect4.kif");
    scenario(&mut group, "rbg/connect4.rbg");
    scenario(&mut group, "hrg/ticTacToe.hrg");
    scenario(&mut group, "kif/ticTacToe.kif");
    scenario(&mut group, "rbg/ticTacToe.rbg");
    scenario(&mut group, "rg/ticTacToe.rg");

    group.finish();
}

criterion_group!(benches, transforms);
criterion_main!(benches);
