use criterion::{criterion_group, criterion_main, Criterion};
use interpreter::ist::{Game, RuntimeId};
use interpreter::{analyze_rg_inner, prepare_ist, Flags};
use std::fs::read_to_string;

fn prepare(path: &str, flags: Flags) -> Game<RuntimeId> {
    let source = read_to_string(format!("../../examples/{path}")).unwrap();
    let game = analyze_rg_inner(source.as_str(), &flags, None::<fn(_)>).unwrap();
    prepare_ist(game).unwrap().0
}

fn scenario(criterion: &mut Criterion, path: &str, counts: Vec<usize>) {
    let mut group = criterion.benchmark_group(format!("runtime/{path}"));

    macro_rules! bench {
        ($name:ident, $flags:expr) => {
            let game = prepare(path, $flags);
            group.bench_function(stringify!($name), |bencher| {
                bencher.iter(|| {
                    for (depth, expected_count) in counts.iter().enumerate() {
                        game.perf(depth, &|count| {
                            assert_eq!(count, *expected_count);
                        });
                    }
                });
            });
        };
    }

    bench!(all_flags, Flags::all());
    bench!(no_flags, Flags::none());
}

fn runtime(criterion: &mut Criterion) {
    scenario(criterion, "breakthrough.rg", vec![1, 22, 484, 11132]);
    scenario(criterion, "ticTacToe.rg", vec![1, 9, 72, 504, 3024, 15120]);
}

criterion_group!(benches, runtime);
criterion_main!(benches);
