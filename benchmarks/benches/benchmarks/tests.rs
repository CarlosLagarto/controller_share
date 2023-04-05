use criterion::{black_box,  Criterion};//BenchmarkId,

pub fn process_array(var: &mut Vec<i32>) {
    let mut _acc = 0;
    for i in var {
        _acc += *i;
    }
}

pub fn process_array_local(var: &mut Vec<i32>) {
    let mut _acc = 0;
    let local = var;
    for i in local {
        _acc += *i;
    }
}

pub fn test_stuff(d: &mut Criterion) {
    let mut c = d.benchmark_group("test_stuff");

    let mut var = vec![1, 2, 3, 4, 5, 6];

    c.bench_function("process_array", |b| b.iter(|| black_box(process_array(&mut var))));
    c.bench_function("process_array_local", |b| b.iter(|| black_box(process_array_local(&mut var))));
    c.finish();
}
