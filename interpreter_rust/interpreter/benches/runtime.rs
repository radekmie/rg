use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use interpreter::{analyze_inner, Flags, Game};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::fs::read_to_string;
use std::time::Duration;

fn scenario(criterion: &mut Criterion, name: &str, variants: &[&str]) {
    let mut rng = SmallRng::seed_from_u64(0);
    let mut group = criterion.benchmark_group(format!("runtime/{name}"));
    group.measurement_time(Duration::from_millis(400));
    group.warm_up_time(Duration::from_millis(100));

    for variant in variants {
        for (flags_name, flags) in [("all", Flags::all()), ("none", Flags::none())] {
            let mut ist = None;
            group.bench_function(
                BenchmarkId::new(variant.to_string(), flags_name),
                |bencher| {
                    let (ist, interner) = ist.get_or_insert_with(|| {
                        let source =
                            read_to_string(format!("../../examples/{name}{variant}")).unwrap();
                        let ast = analyze_inner(source, "rg", &flags, &mut None::<fn(_)>).unwrap();
                        Game::try_from(ast).unwrap()
                    });
                    bencher.iter(|| ist.run(&mut rng, &interner, 1, &None::<fn(_)>))
                },
            );
        }
    }
}

fn runtime(criterion: &mut Criterion) {
    scenario(
        criterion,
        "breakthrough",
        &[".hrg.reuse.rg", ".hrg.rg", ".rbg.rg", ".rg", "-opt.rg"],
    );
    scenario(
        criterion,
        "ticTacToe",
        &[".hrg.reuse.rg", ".hrg.rg", ".kif.rg", ".rbg.rg", ".rg"],
    );
}

criterion_group!(benches, runtime);
criterion_main!(benches);
