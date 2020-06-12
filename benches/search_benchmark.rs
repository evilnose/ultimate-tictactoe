extern crate uttt;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uttt::moves::*;
use uttt::engine;

fn criterion_benchmark(c: &mut Criterion) {
    let mut pos = Position::new();
    init_moves();
    engine::init_engine();
    c.bench_function("search 5", |b| b.iter(|| engine::best_move(black_box(5), black_box(&mut pos))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
