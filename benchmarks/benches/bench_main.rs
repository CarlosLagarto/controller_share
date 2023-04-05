use criterion::{criterion_main, criterion_group};

mod benchmarks;

criterion_group!(
    benches,
    benchmarks::strings::bench_strings,
    benchmarks::schedule::bench_schedule,
    benchmarks::schedule::bench_schedule_specific_wd_group,
    benchmarks::ctrl_time::bench_date_time,
    benchmarks::ctrl_time::bench_suntimes,
    benchmarks::messages::bench_int_message,
    // benchmarks::irrigation_db::bench_update_watered_sector_sql,
    // benchmarks::irrigation_db::bench_update_sectors_batch,
    // benchmarks::irrigation_db::bench_update_sector,
    // benchmarks::irrigation_db::bench_update_scheduled_cycle,
    // benchmarks::irrigation_db::bench_recover_watered_sectors,
    // benchmarks::irrigation_db::bench_water_cycle,
    // benchmarks::irrigation_db::bench_watered_sector_batch,
    benchmarks::ctrl_lib::bench_multiply,
    benchmarks::format::bench_format_strings,
    benchmarks::log_cmp::bench_logss,
    benchmarks::format::bench_format_ints,
    benchmarks::format::bench_format_mix,
    // benchmarks::utils::bench_elapsed_dyn,
    benchmarks::utils::bench_conv_u,
    benchmarks::utils::bench_conv_f,
    benchmarks::utils::bench_display_small_msg,
    // benchmarks::config_vs_db::bench_config_vs_db,
    // benchmarks::ctrl_time::bench_data_structs_size,
    // bench_time_svc_group,
    benchmarks::tests::test_stuff,
    benchmarks::broker::broker_process,
    benchmarks::et::bench_et,
    benchmarks::math::bench_rads_cmp,
);

criterion_main!(benches);
