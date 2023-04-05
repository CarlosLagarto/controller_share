use criterion::{black_box,  Criterion};
use ctrl_lib::{app_time::ctrl_time::CtrlTime, data_structs::sensor::metrics::evapo_transpiracao::*}; //BenchmarkId,

pub fn bench_et(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_et");

    let et_data = EtData{
        time:CtrlTime::from_utc_parts(2022, 7, 6, 15, 0, 0),
        lat: 50.8,
        elev : 100.,
        max_t : 21.5,
        min_t : 12.3,
        avg_hr : 52.,
        max_hr : 84.,
        min_hr : 63.,
         avg_ws:  10.,
         avg_press:  1001.
    };

    c.bench_function("daily_evapo_transpiration_e1", |b| b.iter(|| black_box(daily_evapo_transpiration(et_data.clone()))));
    c.finish();
}