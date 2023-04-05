
use std::time::{Duration, Instant};
use ctrl_lib::ifmt;
use ctrl_lib::utils::conv_int;

// use alloc_counter::AllocCounterSystem;

// #[global_allocator]
// static A: AllocCounterSystem = AllocCounterSystem;

use alloc_counter::count_alloc;
// use mimalloc::MiMalloc;
// #[global_allocator]
// static A: AllocCounter<MiMalloc> = AllocCounter(MiMalloc);

// use alloc_counter::count_alloc;
// // use alloc_counter::AllocCounterSystem;

// // #[global_allocator]
// // static A: AllocCounterSystem = AllocCounterSystem;

use cache_size::*;
// use mimalloc::MiMalloc;
// #[global_allocator]
// static A: AllocCounter<MiMalloc> = AllocCounter(MiMalloc);
use ctrl_lib::{utils::elapsed_dyn, ArrayVec};

pub fn process_array(var: &mut Vec<i32>) -> i32 {
    let mut acc: i32 = 0;
    for i in var {
        acc += *i;
    }
    acc
}

pub fn process_array_local(var: &mut Vec<i32>) -> i32 {
    let mut acc: i32 = 0;
    let local = var;
    for i in local {
        acc += *i;
    }
    acc
}

pub fn process_arrays(var: &mut ArrayVec<i32, 6>) -> i32 {
    let mut acc: i32 = 0;
    for i in var {
        acc += *i;
    }
    acc
}

pub fn process_array_locals(var: &mut ArrayVec<i32, 6>) -> i32 {
    let mut acc: i32 = 0;
    let local = var;
    for i in local {
        acc += *i;
    }
    acc
}

#[test]
fn test_process() {
    let mut t0: Instant;
    let mut var = vec![1, 2, 3, 4, 5, 6];
    let mut vars: ArrayVec<i32, 6> = ArrayVec::new(); //
    _ = vars.try_extend_from_slice(&var);
    let mut total: u64 = 0;
    for _i in 0..100 {
        t0 = Instant::now();
        process_array(&mut var);
        total += t0.elapsed().as_nanos() as u64;
    }
    println!("tempo da chamada ao process_array: {}", elapsed_dyn(total / 100));

    total = 0;
    for _i in 0..100 {
        t0 = Instant::now();
        process_array_local(&mut var);
        total += t0.elapsed().as_nanos() as u64;
    }
    println!("tempo da chamada ao process_array local: {}", elapsed_dyn(total / 100));

    let mut total: u64 = 0;
    for _i in 0..100 {
        t0 = Instant::now();
        process_arrays(&mut vars);
        total += t0.elapsed().as_nanos() as u64;
    }
    println!("tempo da chamada ao process_array: {}", elapsed_dyn(total / 100));

    total = 0;
    for _i in 0..100 {
        t0 = Instant::now();
        process_array_locals(&mut vars);
        total += t0.elapsed().as_nanos() as u64;
    }
    println!("tempo da chamada ao process_array local: {}", elapsed_dyn(total / 100));

    let (counts, _) = count_alloc(|| process_array(&mut var));
    println!(" process array -> allocs: {} reallocs: {} dealocs: {}", counts.0, counts.1, counts.2);

    let (counts, _) = count_alloc(|| process_array_local(&mut var));
    println!(" process array local-> allocs: {} reallocs: {} dealocs: {}", counts.0, counts.1, counts.2);

    let (counts, _) = count_alloc(|| process_arrays(&mut vars));
    println!(" process arrays -> allocs: {} reallocs: {} dealocs: {}", counts.0, counts.1, counts.2);

    let (counts, _) = count_alloc(|| process_array_locals(&mut vars));
    println!(" process array locals-> allocs: {} reallocs: {} dealocs: {}", counts.0, counts.1, counts.2);
}

#[test]
fn test_caches_size() {
    println!("l1 cache size: {} Kb", l1_cache_size().unwrap() / 1024);
    println!("l1 cache line size: {} bytes", l1_cache_line_size().unwrap());
    println!("l2 cache size: {} Kb", l2_cache_size().unwrap() / 1024);
    println!("l2 cache line size: {} bytes", l2_cache_line_size().unwrap());
    println!("l3 cache size: {} Mb", l3_cache_size().unwrap() / 1024 / 1024);
    println!("l3 cache line size: {} bytes", l3_cache_line_size().unwrap());
}

#[test]
fn test_dyn_elapsed_1() {
    let t1 = Instant::now();
    let mut t2: Instant = t1 + Duration::from_nanos(1);

    println!("nanos");
    //nanos
    test_elapsed(t2, t1);

    t2 = t1 + Duration::from_nanos(999);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_nanos(1000);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_nanos(1001);
    test_elapsed(t2, t1);

    //micros
    println!("micros");
    t2 = t1 + Duration::from_micros(1);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_micros(999);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_micros(1000);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_micros(1001);
    test_elapsed(t2, t1);

    //milis
    println!("milis");
    t2 = t1 + Duration::from_millis(1);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_millis(999);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_millis(1000);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_millis(1001);
    test_elapsed(t2, t1);

    //secs
    println!("secs");

    t2 = t1 + Duration::from_secs(1);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(59);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(60);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(61);
    test_elapsed(t2, t1);

    //min
    println!("mins");

    t2 = t1 + Duration::from_secs(61);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(60 * 2 - 1);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(60 * 3 + 1);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(3600 - 1);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(3600);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(3600 + 1);
    test_elapsed(t2, t1);

    //horas
    println!("horas");

    t2 = t1 + Duration::from_secs(3600 * 3);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(3600 * 23);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(3600 * 24);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(3600 * 24 + 1);
    test_elapsed(t2, t1);

    //dias
    println!("dias");

    t2 = t1 + Duration::from_secs(3600 * 24 - 1);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(3600 * 24);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(3600 * 24 + 1);
    test_elapsed(t2, t1);
    t2 = t1 + Duration::from_secs(86400 * 27);
    test_elapsed(t2, t1);
}

fn test_elapsed(t2: Instant, t1: Instant) {
    let dur = (t2 - t1).as_nanos() as u64;
    let tref = Instant::now();
    let result = elapsed_dyn(dur);
    let tnow = tref.elapsed().as_nanos() as u64;
    println!("{}", tnow);
    println!("timing do elapsed{}", elapsed_dyn(tnow));
    println!("{}", result);
}

#[test]
fn test_alloc() {
    let t1 = Instant::now();
    let t2: Instant = t1 + Duration::from_secs(86400 * 27);
    let dur = (t2 - t1).as_nanos() as u64;
    let (counts, res) = count_alloc(|| elapsed_dyn(dur));
    println!("{}", res);
    println!("allocs: {} reallocs: {} dealocs: {}", counts.0, counts.1, counts.2);
}

#[test]
fn call_macro_ifmt() {
    println!("{}", ifmt!(1u16));
    println!("{}", ifmt!(2u64));
}

// #[derive(Default)]
// pub struct RunningStat {
//     pub m_n: u64,
//     pub m_old_m: f64,
//     pub m_new_m: f64,
//     pub m_old_s: f64,
//     pub m_new_s: f64,
// }

// impl RunningStat {
//     pub fn clear(&mut self) {
//         self.m_n = 0;
//     }

//     pub fn push(&mut self, x: f64) {
//         self.m_n += 1;
//         // See Knuth TAOCP vol 2, 3rd edition, page 232
//         if self.m_n == 1 {
//             self.m_new_m = x;
//             self.m_old_m = self.m_new_m;
//             self.m_old_s = 0.;
//         } else {
//             self.m_new_m = self.m_old_m + (x - self.m_old_m) / (self.m_n as f64);
//             self.m_new_s = self.m_old_s + (x - self.m_old_m) * (x - self.m_new_m);
//             // set up for next iteration
//             self.m_old_m = self.m_new_m;
//             self.m_old_s = self.m_new_s;
//         }
//     }

//     pub fn num_data_values(&self) -> u64 {
//         self.m_n
//     }

//     pub fn mean(&self) -> f64 {
//         if self.m_n > 0 {
//             self.m_new_m
//         } else {
//             0.
//         }
//     }
//     pub fn variance(&self) -> f64 {
//         if self.m_n > 1 {
//             self.m_new_s / (self.m_n as f64 - 1.)
//         } else {
//             0.
//         }
//     }

//     pub fn standard_deviation(&self) -> f64 {
//         f64::sqrt(self.variance())
//     }

// }
