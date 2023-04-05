use criterion::{black_box, Criterion};

pub fn bench_to_string() -> String {
    "string test".to_string()
}

pub fn bench_string_from() -> String {
    String::from("string test")
}

pub fn bench_to_owned() -> String {
    "string test".to_owned()
}

pub fn bench_into_string() -> String {
    "string test".into()
}

pub fn bench_new_to_string() -> String {
    "".to_string()
}

pub fn bench_new_string_from() -> String {
    String::from("")
}

pub fn bench_new_to_owned() -> String {
    "".to_owned()
}

pub fn bench_new_into_string() -> String {
    "".into()
}

pub fn bench_new_string() -> String {
    String::new()
}

pub fn bench_strings(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_strings");

    c.bench_function("bench_to_string", |b| b.iter(|| black_box(bench_to_string())));
    c.bench_function("bench_string_from", |b| b.iter(|| black_box(bench_string_from())));
    c.bench_function("bench_to_owned", |b| b.iter(|| black_box(bench_to_owned())));
    c.bench_function("bench_into_string", |b| b.iter(|| black_box(bench_into_string())));
    c.bench_function("bench_new_to_string", |b| b.iter(|| black_box(bench_new_to_string())));
    c.bench_function("bench_new_string_from", |b| b.iter(|| black_box(bench_new_string_from())));
    c.bench_function("bench_new_to_owned", |b| b.iter(|| black_box(bench_new_to_owned())));
    c.bench_function("bench_new_into_string", |b| b.iter(|| black_box(bench_new_into_string())));
    c.bench_function("bench_new_string", |b| b.iter(|| black_box(bench_new_string())));
    c.finish();
}
