use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use interpreter::{analyze_rg_inner, prepare_ist, Flags};
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
        let source = read_to_string(format!("../../examples/{name}{variant}")).unwrap();
        for (flags_name, flags) in [("all", Flags::all()), ("none", Flags::none())] {
            let ast = analyze_rg_inner(source.as_str(), &flags, None::<fn(_)>).unwrap();
            let ist = prepare_ist(ast).unwrap().0;
            group.bench_function(
                BenchmarkId::new(variant.to_string(), flags_name),
                |bencher| bencher.iter(|| ist.run(&mut rng, 1, &|_| {})),
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
    scenario(criterion, "ticTacToe", &[".kif.rg", ".rbg.rg", ".rg"]);
}

criterion_group!(benches, runtime);
criterion_main!(benches);
