use criterion::{Criterion, black_box};

const DEGREE: f64 = 0.0174532925199433;


fn bench_manual_rads_conversion() {
    let _a = 100f64 * DEGREE;
}

fn bench_rust_rads() {
    let _a = 100f64.to_radians();
}

pub fn bench_rads_cmp(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_rads_cmp");

    c.bench_function("bench_manual_rads_conversion", |b| b.iter(|| black_box(bench_manual_rads_conversion())));
    c.bench_function("bench_rust_rads", |b| b.iter(|| black_box(bench_rust_rads())));

    c.finish();
}