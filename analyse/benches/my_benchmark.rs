use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use analyse::{min, max, price_diff, n_window_sma};

pub fn criterion_benchmark(c: &mut Criterion) {
    // Generate 100 random values;
    let mut rng = rand::thread_rng();
    let mut data = vec![];
    for _ in 1..=100 {
        data.push(rng.gen::<f64>());
    }

    c.bench_function("n_window_sma 30", |b| b.iter(|| n_window_sma(black_box(30), &data)));
    c.bench_function("min data", |b| b.iter(|| min(black_box(&data))));
    c.bench_function("max data", |b| b.iter(|| max(black_box(&data))));
    c.bench_function("diff data", |b| b.iter(||price_diff(black_box(&data))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
