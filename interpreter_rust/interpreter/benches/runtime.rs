use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use interpreter::{analyze_inner, Flags, Game};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::fs::read_to_string;
use std::time::Duration;

fn scenario(criterion: &mut Criterion, name: &str, paths: &[&str]) {
    let mut rng = SmallRng::seed_from_u64(0);
    let mut group = criterion.benchmark_group(format!("runtime/{name}"));
    group.measurement_time(Duration::from_millis(400));
    group.warm_up_time(Duration::from_millis(100));

    for path in paths {
        for (flags_name, flags) in [("all", Flags::all()), ("none", Flags::none())] {
            let mut ist = None;
            group.bench_function(
                BenchmarkId::new((*path).to_string(), flags_name),
                |bencher| {
                    let (ist, interner, _) = ist.get_or_insert_with(|| {
                        let source = read_to_string(format!("../../games/{path}")).unwrap();
                        let ast =
                            analyze_inner(source, "rg", &flags, false, &mut None::<fn(_)>).unwrap();
                        Game::try_from(ast).unwrap()
                    });
                    bencher.iter(|| ist.run(&mut rng, interner, 1, &None::<fn(_)>).unwrap());
                },
            );
        }
    }
}

fn runtime(criterion: &mut Criterion) {
    scenario(
        criterion,
        "breakthrough",
        &[
            "hrg/breakthrough.hrg.reuse.rg",
            "hrg/breakthrough.hrg.rg",
            "rbg/breakthrough.rbg.rg",
            "rg/breakthrough.rg",
            "rg/breakthrough-opt.rg",
        ],
    );
    scenario(
        criterion,
        "ticTacToe",
        &[
            "hrg/ticTacToe.hrg.reuse.rg",
            "hrg/ticTacToe.hrg.rg",
            "kif/ticTacToe.kif.rg",
            "rbg/ticTacToe.rbg.rg",
            "rg/ticTacToe.rg",
        ],
    );
}

criterion_group!(benches, runtime);
criterion_main!(benches);
