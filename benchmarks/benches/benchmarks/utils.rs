use criterion::{black_box, BenchmarkId, Criterion};
// use ctrl_lib::{ffmt, string_concat::*};
use ctrl_lib::string_concat::*;
// use ctrl_lib::{string_concat, utils::*};
// use ctrl_prelude::string_resources::*;
use lexical_core::*;
use std::fmt;//, time::Duration}

// pub fn bench_elapsed_dyn(d: &mut Criterion) {
//     let mut c = d.benchmark_group("bench_elapsed_dyn");
//     println!("nanos");
//     let t1 = Duration::from_nanos(1).as_nanos() as u64;
//     let t2 = Duration::from_nanos(999).as_nanos() as u64;
//     let t3 = Duration::from_nanos(1000).as_nanos() as u64;
//     let t4 = Duration::from_nanos(1001).as_nanos() as u64;
//     println!("micros");
//     let t6 = Duration::from_micros(999).as_nanos() as u64;
//     let t7 = Duration::from_micros(1000).as_nanos() as u64;
//     let t8 = Duration::from_micros(1001).as_nanos() as u64;
//     println!("milis");
//     let t10 = Duration::from_millis(999).as_nanos() as u64;
//     let t11 = Duration::from_millis(1000).as_nanos() as u64;
//     let t12 = Duration::from_millis(1001).as_nanos() as u64;

//     // let mut j = 1;
//     // for i in [t1, t2, t3, t4, t6, t7, t8, t10, t11, t12].iter() {
//     //     c.bench_with_input(BenchmarkId::new("elapsed_dyn1_1", j), i, |b, _| b.iter(|| black_box(elapsed_dyn1_1(*i))));
//     //     j += 1;
//     // }

//     // let mut j = 1;
//     // for i in [t1, t2, t3, t4, t6, t7, t8, t10, t11, t12].iter() {
//     //     c.bench_with_input(BenchmarkId::new("elapsed_dyn1_2", j), i, |b, _| b.iter(|| black_box(elapsed_dyn1_2(*i))));
//     //     j += 1;
//     // }

//     // let mut j = 1;
//     // for i in [t1, t2, t3, t4, t6, t7, t8, t10, t11, t12].iter() {
//     //     c.bench_with_input(BenchmarkId::new("elapsed_dyn1_2A", j), i, |b, _| b.iter(|| black_box(elapsed_dyn1_2A(*i))));
//     //     j += 1;
//     // }

//     // let mut j = 1;
//     // for i in [t1, t2, t3, t4, t6, t7, t8, t10, t11, t12].iter() {
//     //     c.bench_with_input(BenchmarkId::new("elapsed_dyn1_3", j), i, |b, _| b.iter(|| black_box(elapsed_dyn1_3(*i))));
//     //     j += 1;
//     // }

//     // let mut j = 1;
//     // for i in [t1, t2, t3, t4, t6, t7, t8, t10, t11, t12].iter() {
//     //     c.bench_with_input(BenchmarkId::new("elapsed_dyn1_3A", j), i, |b, _| b.iter(|| black_box(elapsed_dyn1_3A(*i))));
//     //     j += 1;
//     // }

//     // let mut j = 1;
//     // for i in [t1, t2, t3, t4, t6, t7, t8, t10, t11, t12].iter() {
//     //     c.bench_with_input(BenchmarkId::new("elapsed_dyn1_3B", j), i, |b, _| b.iter(|| black_box(elapsed_dyn1_3B(*i))));
//     //     j += 1;
//     // }
    
//     // let mut j = 1;
//     // for i in [t1, t2, t3, t4, t6, t7, t8, t10, t11, t12].iter() {
//     //     c.bench_with_input(BenchmarkId::new("elapsed_dyn1_4", j), i, |b, _| b.iter(|| black_box(elapsed_dyn1_4(*i))));
//     //     j += 1;
//     // }

//     c.finish();
// }

pub fn bench_conv_u(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_conv_u");

    for i in [1, 999, 138023688] {
        println!("{} - {}", conv_itoa(i), conv_lexi(i));
    }

    for i in [1, 999, 138023688].iter() {
        c.bench_with_input(BenchmarkId::new("itoa", i), i, |b, _| b.iter(|| black_box(conv_itoa(*i))));
    }

    for i in [1, 999, 138023688, 145249298731723, 1223786033230781263].iter() {
        c.bench_with_input(BenchmarkId::new("lexi", i), i, |b, _| b.iter(|| black_box(conv_lexi(*i))));
    }
    c.finish();
}

pub fn bench_conv_f(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_conv_f");

    // for i in [1.0, 19.0, 22.12456, 100.0, 570.123688, 798.1298731723, 983.1230781263] {
    //     println!("{} - {}", i.to_pretty_string(), conv_fmt(i));
    // }

    for i in [1.0, 100.12456, 13800.123688].iter() {
        c.bench_with_input(BenchmarkId::new("fmt", i), i, |b, _| b.iter(|| black_box(conv_fmt(*i))));
    }

    for i in [1.0, 100.12456, 13800.123688].iter() {
        c.bench_with_input(BenchmarkId::new("fmt_args", i), i, |b, _| b.iter(|| black_box(conv_fmt_args(*i))));
    }
    // for i in [1.0, 100.12456, 13800.123688].iter() {
    //     c.bench_with_input(BenchmarkId::new("lexf", i), i, |b, _| b.iter(|| black_box(i.to_pretty_string())));
    // }

    // for i in [1.0, 100.12456, 13800.123688].iter() {
    //     c.bench_with_input(BenchmarkId::new("ffmt", i), i, |b, _| b.iter(|| black_box(ffmt!(*i))));
    // }
    c.finish();
}

fn call_fmt(p1: f64, p2: f64) {
    format!("Terminou o ciclo de rega! Inicio: {:>.3} Fim: {:>.3}", p1, p2);
} //Terminou o ciclo de rega! Inicio: {}  Fim: {}

fn call_fmt_args(p1: f64, p2: f64) {
    let mut output: String = String::with_capacity(80);
    fmt::write(&mut output, format_args!("Terminou o ciclo de rega! Inicio: {:>.3} Fim: {:>.3}", p1, p2)).unwrap();
} //Terminou o ciclo de rega! Inicio: {}  Fim: {}
// fn call_msg(p1: f64, p2: f64) {
//     msg!(INFO_CYCLE_END_2P, p1.to_pretty_string(), p2.to_pretty_string());
// }

// fn call_res(p1: f64, p2: f64) {
//     res(INFO_CYCLE_END_2P, &[&p1.to_pretty_string(), &p2.to_pretty_string()]);
// }

// fn call_concat_directly(p1: f64, p2: f64) {
//     string_concat!(INFO_CYCLE_END_2P(&p1.to_pretty_string(), p2.to_pretty_string()));
// }

// fn call_fn(p1: f64, p2: f64) {
//     string_concat!("Terminou o ciclo de rega! Inicio: ", &ffmt!(p1), "  Fim: ", &ffmt!(p2));
// }

fn call_fmt_args_only_str() {
    let mut output: String = String::with_capacity(80);
    fmt::write(&mut output, format_args!("Terminou o ciclo de rega! Inicio: {} Fim: {}", "string 112", "string 2222")).unwrap();
} //Terminou o ciclo de rega! Inicio: {}  Fim: {}

// fn call_concat_directly_only_str() {
//     string_concat!(INFO_CYCLE_END_2P, "string 112", "string 2222");
// }

pub fn bench_display_small_msg(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_display_small_msg");

    c.bench_function("call_fmt", |b| b.iter(|| black_box(call_fmt(3.45, 6.88))));
    c.bench_function("call_fmt_args", |b| b.iter(|| black_box(call_fmt_args(3.45, 6.88))));
    // c.bench_function("call_msg", |b| b.iter(|| black_box(call_msg(3.45, 6.88))));
    // c.bench_function("call_res", |b| b.iter(|| black_box(call_res(3.45, 6.88))));
    // c.bench_function("call_concat_directly", |b| b.iter(|| black_box(call_concat_directly(3.45, 6.88))));
    // c.bench_function("call_fn", |b| b.iter(|| black_box(call_fn(3.45, 6.88))));
    c.bench_function("call_fmt_args_only_str", |b| b.iter(|| black_box(call_fmt_args_only_str())));
    // c.bench_function("call_concat_directly_only_str", |b| b.iter(|| black_box(call_concat_directly_only_str())));
    c.finish();
}

fn conv_itoa(val: u64) -> String {
    let mut buffer = itoa::Buffer::new();
    buffer.format(val).to_string()
}

fn conv_lexi(val: u64) -> String {
    let mut buffer = [b'\x00'; BUFFER_SIZE].to_vec();
    unsafe {
        val.to_lexical_unchecked(&mut buffer);
    }
    unsafe { String::from_utf8_unchecked(buffer) }
}

fn conv_fmt(val: f64) -> String {
    format!("{:>.3}", val)
}

fn conv_fmt_args(val: f64) -> String {
    let mut output: String = String::with_capacity(32);
    fmt::write(&mut output, format_args!("{:>.3}", val)).unwrap();
    output
}
