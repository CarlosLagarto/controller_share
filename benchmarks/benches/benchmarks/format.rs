use lexical_core::BUFFER_SIZE;
use std::fmt::Write;
use std::io::BufWriter;

use criterion::{black_box, Criterion};

pub fn bench_format_std_only_string() {
    let _s =
        format!("a very big sql string with some parameters select a, b, c, d where a = {} and b {} and c = {}", "'texto1'", "'texto2'", "'texto3'");
}

pub fn bench_format_std_mix() {
    let _s = format!("a very big sql string with some parameters select a, b, c, d where a = {} and b {} and c = {}", 1u16, 1.316f32, "'texto'");
}

pub fn bench_format_std_only_ints() {
    let _s = format!("a very big sql string with some parameters select a, b, c, d where a = {} and b {} and c = {}", 1u16, 2u64, "'texto'");
}

pub fn bench_format_fmt_array_mix() {
    let mut buffer = [0u8; 300];
    let mut buf_writer = &mut buffer[..];

    let _res = std::io::Write::write_fmt(
        &mut buf_writer,
        format_args!("a very big sql string with some parameters select a, b, c, d where a = {} and b {} and c = {}", 1u16, 1.316f32, "'texto'"),
    );
    // unsafe { String::from(std::str::from_utf8_unchecked(&buffer)) }
}

pub fn bench_format_fmt_array_only_ints() {
    let mut buffer = [0u8; 300];
    let mut buf_writer = &mut buffer[..];

    let _res = std::io::Write::write_fmt(
        &mut buf_writer,
        format_args!("a very big sql string with some parameters select a, b, c, d where a = {} and b {} and c = {}", 1u16, 2u64, "'texto'"),
    );
}

pub fn bench_format_fmt_array_only_strings() {
    let mut buffer = [0u8; 300];
    let mut buf_writer = &mut buffer[..];

    let _res = std::io::Write::write_fmt(
        &mut buf_writer,
        format_args!(
            "a very big sql string with some parameters select a, b, c, d where a = {} and b {} and c = {}",
            "'texto1'", "'texto2'", "'texto3'"
        ),
    );
    // unsafe { String::from(std::str::from_utf8_unchecked(&buffer)) }
}

pub fn bench_format_fmt_array_v2_only_ints() {
    let mut buf = itoa::Buffer::new();
    // let printed = buf.format(128u64);

    let mut buffer = [0u8; 300];
    let mut buf_writer = BufWriter::new(&mut buffer[..]);
    let mut _offset =
        std::io::Write::write(&mut buf_writer, "a very big sql string with some parameters select a, b, c, d where a = ".as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, buf.format(1u16).as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, " and b ".as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, buf.format(2u16).as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, " and c = '".as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, "texto".as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, "'".as_bytes()).unwrap();
}

pub fn bench_format_fmt_array_v3_mix() {
    let mut buffer = [0u8; 300];

    let mut buf_writer = BufWriter::new(&mut buffer[..]);
    let mut _offset =
        std::io::Write::write(&mut buf_writer, "a very big sql string with some parameters select a, b, c, d where a = ".as_bytes()).unwrap();
    let mut buf = [b'0'; BUFFER_SIZE];

    _offset += std::io::Write::write(&mut buf_writer, lexical_core::write(1u16, &mut buf)).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, " and b ".as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, lexical_core::write(1.316f32, &mut buf)).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, " and c = '".as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, "texto".as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, "'".as_bytes()).unwrap();
}

pub fn bench_format_fmt_array_v3_only_ints() {
    // let mut buf = itoa::Buffer::new();
    // let printed = buf.format(128u64);
    let mut buffer = [0u8; 300];

    let mut buf_writer = BufWriter::new(&mut buffer[..]);
    let mut _offset =
        std::io::Write::write(&mut buf_writer, "a very big sql string with some parameters select a, b, c, d where a = ".as_bytes()).unwrap();
    let mut buf = [b'0'; BUFFER_SIZE];

    _offset += std::io::Write::write(&mut buf_writer, lexical_core::write(1u16, &mut buf)).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, " and b ".as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, lexical_core::write(2u64, &mut buf)).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, " and c = '".as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, "texto".as_bytes()).unwrap();
    _offset += std::io::Write::write(&mut buf_writer, "'".as_bytes()).unwrap();
}

pub fn bench_format_fmt_only_ints() {
    let mut buffer = String::with_capacity(300);

    let _res = buffer.write_fmt(format_args!(
        "a very big sql string with some parameters select a, b, c, d where a = {} and b {} and c = {}",
        1u16, 2u64, "'texto'"
    ));
}

pub fn bench_format_fmt_mix() {
    let mut buffer = String::with_capacity(300);

    let _res = buffer.write_fmt(format_args!(
        "a very big sql string with some parameters select a, b, c, d where a = {} and b {} and c = {}",
        1u16, 1.316f32, "'texto'"
    ));
}

pub fn bench_format_fmt_only_strings() {
    let mut buffer = String::with_capacity(300);

    let _res = buffer.write_fmt(format_args!(
        "a very big sql string with some parameters select a, b, c, d where a = {} and b {} and c = {}",
        "'texto1'", "'texto2'", "'texto3'"
    ));
}

pub fn bench_format_strings(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_format_strings");

    c.bench_function("bench_format_fmt_std_strings", |b| b.iter(|| black_box(bench_format_std_only_string())));
    c.bench_function("bench_format_fmt_args_strings_string_buf", |b| b.iter(|| black_box(bench_format_fmt_only_strings())));
    c.bench_function("bench_format_fmt_args_strings_array_buf", |b| b.iter(|| black_box(bench_format_fmt_array_only_strings())));

    c.finish();
}

pub fn bench_format_ints(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_format_ints");

    c.bench_function("bench_format_fmt_std_ints", |b| b.iter(|| black_box(bench_format_std_only_ints())));
    c.bench_function("bench_format_fmt_args_ints_string_buf", |b| b.iter(|| black_box(bench_format_fmt_only_ints())));
    c.bench_function("bench_format_fmt_args_ints_array_buf", |b| b.iter(|| black_box(bench_format_fmt_array_only_ints())));
    c.bench_function("bench_format_array_itoa_ints", |b| b.iter(|| black_box(bench_format_fmt_array_v2_only_ints())));
    c.bench_function("bench_format_array_lexical_ints", |b| b.iter(|| black_box(bench_format_fmt_array_v3_only_ints())));

    c.finish();
}

pub fn bench_format_mix(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_format_mix");

    c.bench_function("bench_format_fmt_std_mix", |b| b.iter(|| black_box(bench_format_std_mix())));
    c.bench_function("bench_format_fmt_args_mix_string_buf", |b| b.iter(|| black_box(bench_format_fmt_mix())));
    c.bench_function("bench_format_fmt_args_mix_array_buf", |b| b.iter(|| black_box(bench_format_fmt_array_mix())));
    c.bench_function("bench_format_array_lexical_mix", |b| b.iter(|| black_box(bench_format_fmt_array_v3_mix())));

    c.finish();
}
