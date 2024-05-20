use criterion::{criterion_group, criterion_main, Criterion};
use map_id::MapId;
use rg::ast::Game;
use rg::parsing::parser::parse_with_errors;
use std::fs::read_to_string;
use std::sync::Arc;

fn load(path: &str) -> Game<Arc<str>> {
    let source = read_to_string(format!("../../examples/{path}")).unwrap();
    let (game, errors) = parse_with_errors(&source);
    assert!(errors.is_empty(), "Parse errors: {errors:?}");
    game.map_id(&mut |id| Arc::from(id.identifier.as_str()))
}

fn scenario(criterion: &mut Criterion, path: &str) {
    let game = load(path);
    let mut group = criterion.benchmark_group(format!("transforms/{path}"));

    macro_rules! bench {
        ($fn:ident) => {
            group.bench_function(stringify!($fn), |bencher| {
                bencher.iter(|| {
                    game.clone().$fn().unwrap();
                });
            });
        };
    }

    bench!(calculate_uniques);
    bench!(inline_assignment);
    bench!(prune_unreachable_nodes);
}

fn transforms(criterion: &mut Criterion) {
    scenario(criterion, "breakthrough.rg");
    scenario(criterion, "ticTacToe.rg");
}

criterion_group!(benches, transforms);
criterion_main!(benches);
