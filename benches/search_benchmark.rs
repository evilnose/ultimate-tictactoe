extern crate uttt;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uttt::moves::*;
use uttt::engine;

fn criterion_benchmark(c: &mut Criterion) {
    let mut pos = Position::new();
    init_moves();
    engine::init_engine();
    panic!("Fix best_move first. Implement generic termination function");
    //c.bench_function("search 6", |b| b.iter(|| engine::best_move(black_box(6), black_box(&mut pos))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
