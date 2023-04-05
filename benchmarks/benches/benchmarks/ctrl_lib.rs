use criterion::Criterion;

fn multiply_base(val: u64) -> u64 {
    val * 1_000_000_000
}

#[allow(dead_code)]
fn multiply_bit_i(val: u64) -> u64 {
    let mut factor: u64 = 1_000_000_000;
    let mut res = 0;
    let mut b = val;
    let mut count = 0;
    let mut count1 = 0;

    while factor != 0 {
        if (factor & 1) != 0 {
            count += 1;
            println!("count:{}", count);
            res += b;
        }
        count1 += 1;
        println!("count1:{}", count1);
        b <<= 1;
        factor >>= 1;
    }
    res
}

#[allow(dead_code)]
fn multiply_bit_i_f(val: u64) -> u64 {
    let mut factor: u64 = 73_741_824;
    let mut res = 0;
    let mut b = val;
    let mut count = 0;
    let mut count1 = 0;

    while factor != 0 {
        if (factor & 1) != 0 {
            count += 1;
            println!("count:{}", count);
            res += b;
        }
        count1 += 1;
        println!("count1:{}", count1);
        b <<= 1;
        factor >>= 1;
    }
    res
}

fn multiply_bit(val: u64) -> u64 {
    let mut factor: u64 = 1_000_000_000;
    let mut res = 0;
    let mut b = val;

    while factor != 0 {
        if (factor & 1) != 0 {
            res += b;
        }
        b <<= 1;
        factor >>= 1;
    }
    res
}

fn multiply_bit_unroll(val: u64) -> u64 {
    let mut res = 0;
    let mut b = val;

    b <<= 9; //1-9
    res += b; //1
    b <<= 2; //10-11
    res += b; //2
    b <<= 3; //12-13
    res += b; //3
    b <<= 1; //15
    res += b; //4
    b <<= 2; //16-17
    res += b; //5
    b <<= 2; //18-19
    res += b; //6
    b <<= 1; //20
    res += b; //7
    b <<= 3; //21-23
    res += b; //8
    b <<= 1; //24
    res += b; //9
    b <<= 1; //25
    res += b; //10
    b <<= 2; //26-27
    res += b; //11
    b <<= 1; //28
    res += b; //12
    b <<= 1; //29
    res += b; //13
              // b <<= 1;
              // res += b;

    res
}

fn multiply_bit_unroll_opt(val: u64) -> u64 {
    let mut res = 0;
    let mut b = val;

    // multplica pela potencia de 2 mais proxima - 2^30 = 1_073_741_824

    let res1 = b << 30;
    //vai calcular o valor de 1_073_741_824 - 1_000_000_000 = 73_741_824
    //b <<= 9;//1-9
    b <<= 9; //1-9
    res += b; //1
    b <<= 1; //10
    res += b; //2
    b <<= 2; //11-12
    res += b; //3
    b <<= 1; //13
    res += b; //4
    b <<= 3; //14-16
    res += b; //5
    b <<= 2; //17-18
    res += b; //6
    b <<= 3; //19-21
    res += b; //7
    b <<= 1; //22
    res += b; //8
    b <<= 4; //23-26
    res += b; //9
              // b <<= 1;//25
              // res += b;//10
              // b <<= 2;//26-27
              // res += b;//11
              // b <<= 1;//28
              // res += b;//12
              // b <<= 1;//29
              // res += b;//13
              // b <<= 1;
              // res += b;

    res1 - res
}

fn multiply_bit_unroll_opt_a(val: u64) -> u64 {
    let b = val;

    // multplica pela potencia de 2 mais proxima - 2^30 = 1_073_741_824
    let res_a = b << 29;
    let res_b = b << 28;
    let res_c = b << 27;
    let res_d = b << 25;
    let res_e = b << 24;
    let res_f = b << 23;
    let res_g = b << 20;
    let res_h = b << 19;
    let res_i = b << 17;
    let res_j = b << 15;
    let res_k = b << 14;
    let res_l = b << 11;
    let res_m = b << 9;

    res_a + res_b + res_c + res_d + res_e + res_f + res_g + res_h + res_i + res_j + res_k + res_l + res_m
}

pub fn bench_multiply(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_multiplu_standard");

    // println!("std: {}", multiply_base(1_690_234_123));
    // println!("bit: {}", multiply_bit(1_690_234_123));
    // println!("bit_unroll: {}", multiply_bit_unroll(1_690_234_123));
    // println!("bit_unroll_opt: {}", multiply_bit_unroll_opt(1_690_234_123));
    // println!("bit_unroll_opt_a: {}", multiply_bit_unroll_opt_a(1_690_234_123));

    // c.bench_time_svc("bench_time_svc", |b| b.iter(|| (bench_time_svc(time_svc, start_up))));
    c.bench_function("multiply_base", |b| b.iter(|| (multiply_base(1_690_234_123))));
    c.bench_function("multiply_bit", |b| b.iter(|| (multiply_bit(1_690_234_123))));
    c.bench_function("multiply_bit_unroll", |b| b.iter(|| (multiply_bit_unroll(1_690_234_123))));
    c.bench_function("multiply_bit_unroll_opt", |b| b.iter(|| (multiply_bit_unroll_opt(1_690_234_123))));
    c.bench_function("multiply_bit_unroll_opt_a", |b| b.iter(|| (multiply_bit_unroll_opt_a(1_690_234_123))));

    c.finish();
}
