use cli::{analyze, Flags};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rg_interpreter::Game;
use std::fs::read_to_string;
use std::time::Duration;

fn scenario(criterion: &mut Criterion, game: &str, paths: &[&str]) {
    // Minimum benchmark time just to get some sense about the scale.
    let mut group = criterion.benchmark_group(format!("runtime/{game}"));
    group.measurement_time(Duration::from_millis(1));
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(1));

    for path in paths {
        let extension = path.split('.').last().unwrap();
        let source = read_to_string(format!("../../games/{path}")).unwrap();
        for (flags_name, flags) in [
            ("optimized", Flags::optimized()),
            ("default", Flags::default()),
        ] {
            let mut cache = None;
            group.bench_function(BenchmarkId::new(extension, flags_name), |bencher| {
                let (initial_state, interner, ist, ref mut rng) = cache.get_or_insert_with(|| {
                    let ast = analyze(source.clone(), extension, &flags, false, &mut None::<fn(_)>);
                    let (ist, interner, _) = Game::try_from(ast.unwrap()).unwrap();
                    let initial_state = ist.initial_state_after(&interner, "/").unwrap();
                    let rng = SmallRng::seed_from_u64(0);
                    (initial_state, interner, ist, rng)
                });

                bencher.iter(|| {
                    ist.run(rng, interner, initial_state, 1, &None::<fn(_)>)
                        .unwrap()
                });
            });
        }
    }

    group.finish();
}

fn runtime(criterion: &mut Criterion) {
    scenario(
        criterion,
        "breakthrough",
        &[
            "hrg/breakthrough.hrg",
            "rbg/breakthrough.rbg",
            "rg/breakthrough.rg",
        ],
    );
    scenario(
        criterion,
        "connect4",
        &["hrg/connect4.hrg", "kif/connect4.kif", "rbg/connect4.rbg"],
    );
    scenario(
        criterion,
        "ticTacToe",
        &[
            "hrg/ticTacToe.hrg",
            "kif/ticTacToe.kif",
            "rbg/ticTacToe.rbg",
            "rg/ticTacToe.rg",
        ],
    );
}

criterion_group!(benches, runtime);
criterion_main!(benches);
