#![allow(unused_imports)]
use std::io::ErrorKind;
use std::thread::{self};
use std::time::{Duration, Instant};
use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::integration::{common::*, db_irrigation::*, db_weather::*, water_service::*};

use ctrl_lib::app_context::start_up::*;
use ctrl_lib::arrayvec::ArrayVec;
use ctrl_lib::config::app_config::*;
use ctrl_lib::data_structs::msgs::{alert::*, ext_message::*, int_message::*, weather::*};
use ctrl_lib::data_structs::sensor::{daily_value::*, snsor::*, stat_metric::*};
use ctrl_lib::db::db_sql_lite::*;
use ctrl_lib::lib_serde::{data_from_str, data_to_str};
use ctrl_lib::services::msg_broker::{msg_brkr_svc::*, subscriber::*};
use ctrl_lib::services::weather::rain_pred::{data_structs::*, naive_bayes::*};
use ctrl_lib::services::weather::sources::simulation::{mock_weather_data::*, svc::*};
use ctrl_lib::services::weather::sources::tempest::{data_structs::*, rest::*, station::*};
use ctrl_lib::services::weather::{algorithms::*, db_model::*, scale::*, sources::*, trend::*, weather_history::*, weather_inner::*, weather_svc::*};
use ctrl_lib::services::web_svc::*;
use ctrl_lib::{app_time::ctrl_time::*, config::wthr_cfg::*, utils::*};
use ctrl_lib::{log_debug, log_error, log_warn, logger::*};
use ctrl_prelude::globals::*;

#[test]
fn get_weather_success() {
    let _start_ts = CtrlTime::from_utc_parts(2022, 1, 14, 19, 30, 1);
    let db = Persist::new();

    let msg_broker = Arc::new(MsgBrkr::new());
    let wthr_cfg = arc_rw(WthrCfg::new(db.clone(), _start_ts));
    let _handle_evt_mng = msg_broker.start();
    let w = WeatherInner::new(_start_ts, db.clone(), msg_broker.clone(), wthr_cfg);

    let wc = w.wthr_cfg.read();

    let _ = w.site.get_weather(_start_ts, wc.token_tempest.clone(), wc.device_id_tempest.clone(), wc.geo.elev as f32);
    // println!("{}", data_to_str(&res.unwrap()).unwrap());

    msg_broker.terminate();
    assert!(true);
}

const UDP_ERRORS: [ErrorKind; 2] = [ErrorKind::WouldBlock, ErrorKind::TimedOut];
// Só pode ser executado com a estação por perto...a alternativa é lançar uma thread que simule o udp.....
// para já executa sempre, e faz timeout ao fim de 15 segundos
#[test]
// #[ignore]
fn test_tempest() {
    use std::net::UdpSocket;
    let mut buf = vec![0_u8; 1024];
    let socket = UdpSocket::bind("0.0.0.0:50223").expect("couldn't bind to address");

    socket.set_read_timeout(Some(Duration::new(1, 0))).expect("set_read_timeout call failed");
    _ = socket.set_broadcast(true);
    let mut iter = 0;
    // let mut minutes = 0;
    loop {
        match socket.recv_from(&mut buf) {
            Ok(_req) => {
                () // println!("recv: {:?} {:?}", req, std::str::from_utf8(&buf[..req.0]));
            }
            Err(e) => {
                let kind = e.kind();
                if UDP_ERRORS.contains(&kind) {
                    //ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock {
                    // no linux pela doc, pode ser diferente
                    iter += 1;
                } else {
                    println!("{}", e.to_string());
                    println!("{}", e.kind());
                }
            }
        };
        // if iter % 60 == 0 {
        //     minutes += 1;
        //     println!("minutos: {}", minutes);
        // }
        if iter > 15 {
            break;
        }
    } //end loop
}

const OBS_ST_OBS: ObsStObs = ObsStObs {
    epoch: 1588948614,
    wind_lull_min3: 0.18,
    wind_avg: 0.22,
    wind_gust_max3: 0.27,
    wind_direction: 144,
    wind_sample_interval: 6,
    station_pressure: 1017.57,
    air_temperature: 22.37,
    relative_humidity: 50.26,
    illuminance: 328,
    uv: 0.03,
    solar_radiation: 3,
    rain_minute: 0.00000,
    precipitation_type: 0,
    lightning_strike_dist: 0,
    lightning_strike_count: 0,
    battery: 2.410,
    report_interval: 1,
};

#[test]
fn st_obs_rest_1() {
    let buf = r#"
    {
        "status": {
            "status_code": 0,
            "status_message": "SUCCESS"
        },
        "device_id": 211801,
        "type": "obs_st",
        "source": "cache",
        "summary": {
            "pressure_trend": "steady",
            "strike_count_1h": 0,
            "strike_count_3h": 0,
            "precip_total_1h": 0.0,
            "feels_like": 28.7,
            "heat_index": 28.7,
            "wind_chill": 28.5
        },
        "obs": [[1658487610,0,0,0,0,3,1007,28.5,47,2211,0.18,18,0,0,0,0,2.61,1,0,null,null,0]]
    }"#;
    let deserialized: TempestRest = data_from_str(&buf).unwrap(); 
    let t = TempestRest::ObsSt(ObsStRest {
        status: Status { status_code: 0, status_message: "SUCCESS".to_owned() },
        device_id: 211801,
        source: "cache".to_owned(),
        summary: Summary {
            pressure_trend: Some("steady".to_owned()),
            strike_count_1h: Some(0),
            strike_count_3h: Some(0),
            precip_total_1h: Some(0.0),
            feels_like: Some(28.7),
            heat_index: Some(28.7),
            wind_chill: Some(28.5),
        },
        obs: vec![ObsStObsRest {
            epoch: 1658487610,                    //1658487610,
            wind_lull_min3: 0.,                   // 0 Km 0,
            wind_avg: 0.,                         // 0 Km 0,
            wind_gust_max3: 0.,                   // 0 Km 0,
            wind_direction: 0,                    // 0 Km 0,
            wind_sample_interval: 3,              // 3,
            station_pressure: 1007.,              //1007,
            air_temperature: 28.5,                // degrees C 28.5,
            relative_humidity: 47.,               // 47,
            illuminance: 2211,                    // //2211,
            uv: 0.18,                             // 10.0 Degrees C 0.18,
            solar_radiation: 18,                  //18,
            rain_minute: 0.,                      // 0,
            precipitation_type: 0,                // 0,
            average_strike_distance: 0,           //0,
            lightning_strike_count: 0,            //0,
            battery: 2.61,                        //2.61,
            report_interval: 1,                   // 1,
            local_day_rain_accumulation: 0.0,     // 0,
            nc_rain_accumulation: None,           //null,
            local_day_nc_rain_accumulation: None, //null,
            precipitation_analysis_type: 0,       //0 = none, 1 = Rain Check with user display on, 2 = Rain Check with user display off
        }],
    });
    assert!(t == deserialized);
}

#[test]
fn test_prep_new_day_running_for_more_than_one_day() {
    let _start_ts = CtrlTime::from_utc_parts(2022, 1, 14, 19, 30, 1); //simula que a maquina está em funcionamento desde o inicio 2022
    let db = Persist::new();
    // temos que introduzir dados
    let date_ref_original = CtrlTime::from_utc_parts(2022, 08, 01, 0, 0, 0);
    let date_ref = date_ref_original;
    //limpamos dados de teste do dia em causa
    _ = db.del_sensor_data(date_ref);
    _ = db.del_daily_data(date_ref);
    // criamos os dados para o dia
    create_one_day_of_data(date_ref, &db);

    let msg_broker = Arc::new(MsgBrkr::new());
    let wthr_cfg = arc_rw(WthrCfg::new(db.clone(), _start_ts));
    let _handle_evt_mng = msg_broker.start();
    let mut w = WeatherInner::new(_start_ts, db.clone(), msg_broker.clone(), wthr_cfg);
    // e agora simulamos o evento de novo dia
    _ = w.prep_new_day(date_ref_original.add_days(1));
    assert!(w.data.data.len() > 1);
}

#[test]
fn test_prep_new_day_running_for_less_than_one_day() {
    let _start_ts = CtrlTime::from_utc_parts(2022, 7, 31, 19, 30, 1); //simula que a maquina está em funcionamento desde o inicio 2022
    let db = Persist::new();
    // temos que introduzir dados
    let date_ref_original = CtrlTime::from_utc_parts(2022, 08, 01, 0, 0, 0);
    let date_ref = date_ref_original;
    //limpamos dados de teste do dia em causa
    _ = db.del_sensor_data(date_ref);
    _ = db.del_daily_data(date_ref);
    // criamos os dados para o dia
    create_one_day_of_data(date_ref, &db);

    let msg_broker = Arc::new(MsgBrkr::new());

    let wthr_cfg = arc_rw(WthrCfg::new(db.clone(), _start_ts));
    let _handle_evt_mng = msg_broker.start();
    let mut w = WeatherInner::new(_start_ts, db.clone(), msg_broker.clone(), wthr_cfg);
    // e agora simulamos o evento de novo dia

    _ = w.prep_new_day(date_ref_original);
    assert!(w.data.data.len() == 1);
}

#[test]
fn test_insert_in_vector() {
    let mut vec: Vector<MAX_FEATURES> = Vector::new();

    vec.push(SensorValue::new_et(CtrlTime::sys_time(), 0.1));

    assert!(vec.data.len() == 1);
}

#[test]
fn test_remove_vec() {
    let mut vec: Vector<MAX_FEATURES> = Vector::new();

    vec.push(SensorValue::new_et(CtrlTime::sys_time(), 0.1));

    assert!(vec.data.len() == 1);

    vec.remove(Metric::EvapoTranspiration);
    assert!(vec.data.len() == 0);
}

#[test]
fn test_remove_vec_with_more_than_one_elem() {
    let mut vec: Vector<MAX_FEATURES> = Vector::new();

    vec.push(SensorValue::new_et(CtrlTime::sys_time(), 0.1));
    vec.push(SensorValue::new_rain_class(CtrlTime::sys_time(), 0.1));
    vec.push(SensorValue::new_rain_class_forecast(CtrlTime::sys_time(), 0.1));

    assert!(vec.data.len() == 3);

    let res = vec.remove(Metric::EvapoTranspiration);
    assert!(res.is_some());
    assert!(vec.data.len() == 2);
}

#[test]
fn test_remove_vec_no_existing_elem() {
    let mut vec: Vector<MAX_FEATURES> = Vector::new();

    vec.push(SensorValue::new_rain_class(CtrlTime::sys_time(), 0.1));
    vec.push(SensorValue::new_rain_class_forecast(CtrlTime::sys_time(), 0.1));

    assert!(vec.data.len() == 2);

    let res = vec.remove(Metric::EvapoTranspiration);
    assert!(vec.data.len() == 2);
    assert!(res.is_none());
}

#[test]
fn test_likelihood() {
    let mut prediction = NBGaussianPrediction::default();

    let (model, x_rust) = prep_likelihood_data();

    prediction.jll = log_likelihood(&model, &x_rust);

    // println!("assert!((prediction.jll[0] - {}).abs() < 0.00001);", prediction.jll[0]);
    // println!("assert!((prediction.jll[1] - {}).abs() < 0.00001);", prediction.jll[1]);
    // println!("assert!((prediction.jll[2] - {}).abs() < 0.00001);", prediction.jll[2]);
    // println!("assert!((prediction.jll[3] - {}).abs() < 0.00001);", prediction.jll[3]);
    // println!("assert!((prediction.jll[4] - {}).abs() < 0.00001);", prediction.jll[4]);

    assert!((prediction.jll[0] - -32.85334804139299).abs() < 0.00001);
    assert!((prediction.jll[1] - -32.85334804139299).abs() < 0.00001);
    assert!((prediction.jll[2] - -32.85334804139299).abs() < 0.00001);
    assert!((prediction.jll[3] - -32.85334804139299).abs() < 0.00001);
    assert!((prediction.jll[4] - -32.85334804139299).abs() < 0.00001);
}

fn prep_likelihood_data() -> (Model, [f64; MAX_FEATURES]) {
    let mut model = Model::new();
    model.selected_features =
        vec![0, 1, 2, 3, 4, 5, 6, 7, 9, 12, 13, 14, 15, 16, 19, 20, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 39, 42, 43, 44, 47];
    let mut x_rust: DSRow<{ MAX_FEATURES }> = [0.; MAX_FEATURES];
    for idx in &model.selected_features {
        x_rust[*idx] = 1.;
    }
    for class in 0..NR_CLASSES {
        model.class_prior[class] = 0.2;
        for feature in 0..MAX_FEATURES {
            let mut welford = WelfordMeanAndVar::default();
            welford.k = 2;
            welford.mean = 1.;
            welford.s = 1.;
            model.fit_stats[class][feature] = welford;
        }
    }
    (model, x_rust)
}

#[test]
fn test_log_probability() {
    let (model, x_rust) = prep_likelihood_data();

    let prediction = predict_log_probability(&model, &x_rust);

    // println!("assert!((prediction.log_probability[0] - {}).abs() < 0.00001);", prediction.log_probability[0]);
    // println!("assert!((prediction.log_probability[1] - {}).abs() < 0.00001);", prediction.log_probability[1]);
    // println!("assert!((prediction.log_probability[2] - {}).abs() < 0.00001);", prediction.log_probability[2]);
    // println!("assert!((prediction.log_probability[3] - {}).abs() < 0.00001);", prediction.log_probability[3]);
    // println!("assert!((prediction.log_probability[4] - {}).abs() < 0.00001);", prediction.log_probability[4]);

    assert!((prediction.log_probability[0] - -1.6094379124340996).abs() < 0.00001);
    assert!((prediction.log_probability[1] - -1.6094379124340996).abs() < 0.00001);
    assert!((prediction.log_probability[2] - -1.6094379124340996).abs() < 0.00001);
    assert!((prediction.log_probability[3] - -1.6094379124340996).abs() < 0.00001);
    assert!((prediction.log_probability[4] - -1.6094379124340996).abs() < 0.00001);
}

#[test]
fn test_probability() {
    let (model, x_rust) = prep_likelihood_data();

    let prediction = predict_probability(&model, &x_rust);

    // println!("assert!((prediction.probability[0] - {}).abs() < 0.00001);", prediction.probability[0]);
    // println!("assert!((prediction.probability[1] - {}).abs() < 0.00001);", prediction.probability[1]);
    // println!("assert!((prediction.probability[2] - {}).abs() < 0.00001);", prediction.probability[2]);
    // println!("assert!((prediction.probability[3] - {}).abs() < 0.00001);", prediction.probability[3]);
    // println!("assert!((prediction.probability[4] - {}).abs() < 0.00001);", prediction.probability[4]);

    assert!((prediction.probability[0] - 0.20000000000000015).abs() < 0.00001);
    assert!((prediction.probability[1] - 0.20000000000000015).abs() < 0.00001);
    assert!((prediction.probability[2] - 0.20000000000000015).abs() < 0.00001);
    assert!((prediction.probability[3] - 0.20000000000000015).abs() < 0.00001);
    assert!((prediction.probability[4] - 0.20000000000000015).abs() < 0.00001);
}

#[test]
fn test_get_model_non_exist() {
    let db = Persist::new();
    let model = get_model(&db, 99);

    assert!(model.is_none());
}

#[test]
fn test_get_model_exist() {
    let db = Persist::new();
    let model = get_model(&db, 0);

    assert!(model.is_some());
}

#[test]
fn test_log_sum_exp_cmp_python() {
    let vec1 = [-34.93379304, -34.82580541, -57.42218555, -74.60947399, -60.91755919];
    let mut xmax = f64::MIN;
    for f in vec1.iter() {
        xmax = xmax.max(*f);
    }
    let sum_exp = log_sum_exp(&vec1, xmax);
    assert!((sum_exp - -34.18519509).abs() < 0.001, "log sum exp é: {} e devia ser: -34.18519509", sum_exp);
}

#[test]
fn test_max() {
    let vec = [1., 2., 3., 4., 10., 6., 7., 8., 9.];
    let idx = max_index(&vec);
    assert_eq!(idx, 4);
}

#[test]
fn test_welford() {
    let vec = [1., 2., 3., 4., 5., 6., 7., 8., 9.];

    let mut w = WelfordMeanAndVar::default();

    for i in vec {
        w.next(i);
    }
    assert!((w.mean - 5.).abs() < 0.00001);
    assert!((w.var() - 6.666666666666667).abs() < 0.00001);
}

#[test]
fn scale_new() {
    let a = Scale::new(0., 100.);

    // println!("{}", a.get(1000.));
    assert!((a.get(2.) - 200.).abs() <= 0.);
    // println!("{}", a.middle());
    assert!((a.span() - 50.).abs() <= 0.);
    assert!((a.middle() - 25.).abs() <= 0.);
}

#[test]
fn test_simulation() {
    unsafe { MOCK_SIM = Some(MockSimulation::new()) };

    let simulation = Simulation {};

    let weather = simulation.get_weather(CtrlTime::sys_time());

    let s_w = data_to_str(&weather).unwrap();
    assert!(s_w != "");
    // println!("{}", s_w);
}

#[test]
fn test_weather_station_get_weather() {
    let wst = WeatherStation {};

    let time = CtrlTime::sys_time();
    let altitude = 5.;
    let weather = wst.get_weather(time, &OBS_ST_OBS, altitude);

    let oraculo = Weather {
        current_time_ts: time.ux_ts(),
        utcnow_dt: time.as_date_web_str_e(),
        rain_period: OBS_ST_OBS.rain_minute,
        wind_bearing: OBS_ST_OBS.wind_direction as f32,
        wind_intensity: OBS_ST_OBS.wind_avg,
        temperature: OBS_ST_OBS.air_temperature,
        humidity: OBS_ST_OBS.relative_humidity,
        pressure: station_pressure_to_sea_pressure(OBS_ST_OBS.station_pressure, OBS_ST_OBS.air_temperature, altitude),
        ..Default::default()
    };

    let s_w = data_to_str(&weather).unwrap();
    assert!(s_w == data_to_str(&oraculo).unwrap());
    // println!("{}", s_w);
}

#[test]
fn evt_precip() {
    let buf = r#"
        {
            "serial_number": "SK-00008453",
            "type":"evt_precip",
	        "hub_sn": "HB-00000001",
	        "evt":[1493322445]
        }"#;
    let deserialized: Tempest = data_from_str(&buf).unwrap();
    let t = Tempest::EvtPrecip(EvtPrecip {
        hub_sn: String::from("HB-00000001"),
        evt: EvtPrecipEvt { epoch: 1493322445 },
        serial_number: String::from("SK-00008453"),
    });
    assert!(t == deserialized);
}

#[test]
fn evt_strike() {
    let buf = r#"
        {
            "serial_number": "AR-00004049",
            "type":"evt_strike",
            "hub_sn": "HB-00000001",
            "evt":[1493322445,27,3848]
        }"#;
    let deserialized: Tempest = data_from_str(&buf).unwrap();
    let t = Tempest::EvtStrike(EvtStrike {
        hub_sn: String::from("HB-00000001"),
        evt: EvtStrikeEvt { epoch: 1493322445, distance: 27, energy: 3848 },
        serial_number: String::from("AR-00004049"),
    });
    assert!(t == deserialized);
}

#[test]
fn rapid_wind() {
    let buf = r#"
        {
            "serial_number": "SK-00008453",
            "type": "rapid_wind",
            "hub_sn": "HB-00000001",
            "ob":[1493322445,2.3,128]
        }"#;
    let deserialized: Tempest = data_from_str(&buf).unwrap();
    let t = Tempest::RapidWind(RapidWind {
        hub_sn: String::from("HB-00000001"),
        ob: RapidWindOb { epoch: 1493322445, wind_speed: 2.3, wind_direction: 128 },
        serial_number: String::from("SK-00008453"),
    });
    assert!(t == deserialized);
}

#[test]
fn obs_air() {
    let buf = r#"
        {
            "serial_number": "AR-00004049",
            "type":"obs_air",
            "hub_sn": "HB-00000001",
            "obs":[
                [1493164835,835.0,10.0,45,0,0,3.46,1]
            ],
            "firmware_revision": 17
        }"#;
    let deserialized: Tempest = data_from_str(&buf).unwrap();
    let t = Tempest::ObsAir(ObsAir {
        hub_sn: String::from("HB-00000001"),
        obs: vec![ObsAirObs {
            epoch: 1493164835,
            station_pressure: 835.0,
            air_temperature: 10.0,
            relative_humidity: 45,
            lightning_strike_count: 0,
            lightning_strike_avg_distance: 0,
            battery: 3.46,
            report_interval: 1,
        }],
        serial_number: String::from("AR-00004049"),
        firmware_revision: 17,
    });
    assert!(t == deserialized);
}

#[test]
fn obs_sky() {
    let buf = r#"
        {
            "serial_number": "SK-00008453",
            "type":"obs_sky",
            "hub_sn": "HB-00000001",
            "obs":[
                [1493321340,9000,10,0.0,2.6,4.6,7.4,187,3.12,1,130,null,0,3]
            ],
            "firmware_revision": 29
        }"#;
    let deserialized: Tempest = data_from_str(&buf).unwrap();
    let t = Tempest::ObsSky(ObsSky {
        hub_sn: String::from("HB-00000001"),
        obs: vec![ObsSkyObs {
            epoch: 1493321340,
            illuminance: 9000,
            uv: 10,
            rain_minute: 0.0,
            wind_lull_min3: 2.6,
            wind_avg: 4.6,
            wind_gust_max3: 7.4,
            wind_direction: 187,
            battery: 3.12,
            report_interval: 1,
            solar_radiation: 130,
            rain_day: None,
            precipitation_type: 0,
            wind_sample_interval: 3,
        }],
        serial_number: String::from("SK-00008453"),
        firmware_revision: 29,
    });
    assert!(t == deserialized);
}
#[test]
fn obs_st() {
    let buf = r#"
        {
            "serial_number": "AR-00000512",
            "type":"obs_st",
            "hub_sn": "HB-00013030",
            "obs":[
                [1588948614,0.18,0.22,0.27,144,6,1017.57,22.37,50.26,328,0.03,3,0.00000,0,0,0,2.410,1]
            ],
            "firmware_revision": 129
        }"#;
    let deserialized: Tempest = data_from_str(&buf).unwrap();
    let t = Tempest::ObsSt(ObsSt {
        hub_sn: String::from("HB-00013030"),
        obs: vec![ObsStObs {
            epoch: 1588948614,
            wind_lull_min3: 0.18,
            wind_avg: 0.22,
            wind_gust_max3: 0.27,
            wind_direction: 144,
            wind_sample_interval: 6,
            station_pressure: 1017.57,
            air_temperature: 22.37,
            relative_humidity: 50.26,
            illuminance: 328,
            uv: 0.03,
            solar_radiation: 3,
            rain_minute: 0.00000,
            precipitation_type: 0,
            lightning_strike_dist: 0,
            lightning_strike_count: 0,
            battery: 2.410,
            report_interval: 1,
        }],
        serial_number: String::from("AR-00000512"),
        firmware_revision: 129,
    });
    assert!(t == deserialized);
}
#[test]
fn device_status() {
    let buf = r#"
        {
            "serial_number": "AR-00004049",
            "type": "device_status",
            "hub_sn": "HB-00000001",
            "timestamp": 1510855923,
            "uptime": 2189,
            "voltage": 3.50,
            "firmware_revision": 17,
            "rssi": -17,
            "hub_rssi": -87,
            "sensor_status": 0,
            "debug": 0
        }"#;
    let deserialized: Tempest = data_from_str(&buf).unwrap();
    let t = Tempest::DeviceStatus(DeviceStatus {
        serial_number: String::from("AR-00004049"),
        hub_sn: String::from("HB-00000001"),
        timestamp: 1510855923,
        uptime: 2189,
        voltage: 3.50,
        firmware_revision: 17,
        rssi: -17,
        hub_rssi: -87,
        sensor_status: 0,
        debug: 0,
    });
    assert!(t == deserialized);
}

#[test]
fn hub_status() {
    let buf = r#"
        {
            "serial_number":"HB-00000001",
            "type":"hub_status",
            "firmware_revision":"35",
            "uptime":1670133,
            "rssi":-62,
            "timestamp":1495724691,
            "reset_flags": "BOR,PIN,POR",
            "seq": 48,
            "fs": [1, 0, 15675411, 524288],
            "radio_stats": [2, 1, 0, 3, 2839],
            "mqtt_stats": [1, 0]
        }"#;
    let deserialized: Tempest = data_from_str(&buf).unwrap();
    let t = Tempest::HubStatus(HubStatus {
        serial_number: String::from("HB-00000001"),
        firmware_revision: String::from("35"),
        uptime: 1670133,
        rssi: -62,
        timestamp: 1495724691,
        reset_flags: String::from("BOR,PIN,POR"),
        seq: 48,
        // fs: vec![1, 0, 15675411, 524288],
        radio_stats: RadioStats { version: 2, reboots: 1, i2c_errors: 0, radio_status: 3, network_id: 2839 },
        mqtt_stats: vec![1, 0],
    });
    assert!(t == deserialized);
}

#[test]
fn test_weather_hist_build_no_data() {
    let db = Persist::new();
    let time = CtrlTime::sys_time();
    let o_data = WeatherHstry::build(time.ux_ts(), &db, "".to_owned());
    assert!(o_data.is_some());
    let history_msg = WeatherHstry::new_out(ExtMsgOut::WeatherHistory(Box::new(o_data.unwrap())), time);
    let s_msg = data_to_str(&history_msg).unwrap();
    // println!("{}", s_msg);
    assert!(s_msg != "");
    if let ExtMsgOut::WeatherHistory(data) = history_msg {
        assert!(data.temp_and_hp.len() == 0);
    }
}

#[test]
fn test_weather_hist_build_with_data() {
    let db = Persist::new();
    let time = CtrlTime::from_utc_parts(2022, 1, 14, 0, 0, 0);

    //limpamos dados de teste do dia em causa
    _ = db.del_sensor_data(time);

    // criamos os dados para o dia
    create_one_day_of_data(time, &db);

    let o_data = WeatherHstry::build(time.eod_ux_e().ux_ts(), &db, "".to_owned());
    assert!(o_data.is_some());
    let history_msg = WeatherHstry::new_out(ExtMsgOut::WeatherHistory(Box::new(o_data.unwrap())), time);
    let s_msg = data_to_str(&history_msg).unwrap();
    // println!("{}", s_msg);
    assert!(s_msg != "");
    if let ExtMsgOut::WeatherHistory(data) = history_msg {
        assert!(data.temp_and_hp.len() > 0);
    }
}

#[test]
fn test_db_select_agregated_values_no_data() {
    let db = Persist::new();
    let time = CtrlTime::from_utc_parts(2022, 1, 14, 0, 0, 0);

    //limpamos dados de teste
    _ = db.del_all_sensor_daily_data();

    let result = db.get_daily_metric(0, time.ux_ts());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_db_select_agregated_values_with_data() {
    let db = Persist::new();
    let time = CtrlTime::from_utc_parts(2022, 1, 14, 0, 0, 0).sod_ux_e();

    //limpamos dados de teste
    _ = db.del_all_sensor_daily_data();

    let mut daily_metrics_buf = ArrayVec::<SensorValue, 1>::new();
    daily_metrics_buf.push(SensorValue::new_et(time, 3.));
    _ = db.ins_daily_data_batch_aux(&daily_metrics_buf);
    assert_eq!(1642118400, time.sod_ux_e().ux_ts(), "o ux_ts é {} e devia ser {}", 1642118400, time.sod_ux_e().ux_ts());
    let result = db.get_daily_metric(Metric::EvapoTranspiration as u8, time.ux_ts());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_db_get_daily_measure_no_data() {
    let db = Persist::new();
    let time = CtrlTime::from_utc_parts(2022, 1, 14, 0, 0, 0);
    //limpamos dados de teste do dia em causa
    _ = db.del_sensor_data(time);
    let result = db.get_sensor_data(Sensor::TempOutside as u8, time.ux_ts());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_db_get_daily_measure_with_data() {
    let db = Persist::new();
    let time = CtrlTime::from_utc_parts(2022, 1, 14, 0, 0, 0);
    //limpamos dados de teste do dia em causa
    _ = db.del_sensor_data(time);
    // criamos os dados para o dia
    create_one_day_of_data(time, &db);
    let result = db.get_sensor_data(Sensor::TempOutside as u8, time.ux_ts());
    assert!(result.unwrap().is_some());
}
#[test]
fn test_snd_alert() {
    let msg_broker: SMsgBrkr = Arc::new(MsgBrkr::new());
    let _handle_evt_mng = msg_broker.start();
    let time = CtrlTime::from_utc_parts(2022, 1, 14, 0, 0, 0);
    snd_alert(AlertType::RAIN, 0., time, &msg_broker);
}

#[test]
fn test_get_rest() {
    let (time, station_altitude, inner, _db) = create_weather_process_objects();

    let o_weather = inner.get_weather_from_site(time, station_altitude);
    assert!(o_weather.is_some());
}

fn create_weather_process_objects() -> (CtrlTime, f32, WeatherInner, Persist) {
    let time = CtrlTime::from_utc_parts(2022, 1, 14, 0, 0, 0);
    let station_altitude = 5.;
    let db = Persist::new();
    let msg_broker = Arc::new(MsgBrkr::new());
    let live_since = time.sub_days(2);
    let wthr_cfg = arc_rw(WthrCfg::new(db.clone(), live_since));
    let inner = WeatherInner::new(time, db.clone(), msg_broker, wthr_cfg);
    (time, station_altitude, inner, db)
}

#[test]
fn test_process_weather_one_call_on_time_station_no_data() {
    let (_time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    let time = inner.sched_weather.start;
    inner.process_manager(time);
    // porque o weather source é "station", e não estamos a recolher efetivamente o tempo pela station
    assert!(inner.o_weather.is_none());
}

#[test]
fn test_process_weather_one_call_on_time_rest_data() {
    let (_time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::WebREST);

    let time = inner.sched_weather.start;
    inner.process_manager(time);
    // porque o weather source é "rest"
    assert!(inner.o_weather.is_some());
}

#[test]
fn test_process_weather_one_call_on_time_sim_data() {
    let (_time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Simulation);
    let time = inner.sched_weather.start;
    inner.process_manager(time);
    // porque o weather source é "rest"
    assert!(inner.o_weather.is_some());
}

#[test]
fn test_process_weather_one_call_on_time_station_data() {
    let (time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    inner.o_weather = get_weather_data(1013., time);
    let time = inner.sched_weather.start;
    inner.process_manager(time);
    assert!(inner.o_weather.is_some());
}

#[test]
fn test_process_weather_one_call_before_time() {
    let (_time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    let time = inner.sched_weather.start.sub_secs_f32(30.);
    inner.process_manager(time);
    assert!(inner.o_weather.is_none());
}

#[test]
fn test_process_weather_one_call_after_time() {
    let (time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    inner.o_weather = get_weather_data(1013., time);
    let time = inner.sched_weather.start.add_secs(30);
    inner.process_manager(time);
    // deve correr, porque se não se atualizou o schedule, assume-se que está atrasado e corre na mesma.
    // ou seja, só o before time, que pressupoe que se atualizou o schedule, é que não corre.
    assert!(inner.o_weather.is_some());
}

#[test]
fn test_process_weather_with_alerts() {
    let (time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    inner.o_weather = get_weather_data(1013., time);
    inner.o_weather.as_mut().unwrap().rain_period = 0.3;
    inner.o_weather.as_mut().unwrap().wind_intensity = 21.;
    let time = inner.sched_weather.start.add_secs(30);
    inner.process_manager(time);
    // deve correr, porque se não se atualizou o schedule, assume-se que está atrasado e corre na mesma.
    // ou seja, só o before time, que pressupoe que se atualizou o schedule, é que não corre.
    assert!(inner.o_weather.is_some());
    assert!(inner.wthr_cfg.read().alrt_thresholds.is_rain_alert(0.3));
    assert!(inner.wthr_cfg.read().alrt_thresholds.is_wind_alert(21.));
    assert!(inner.wthr_cfg.read().alrt_thresholds.is_weather_alert(0.3, 21.));
}

#[test]
fn test_process_weather_with_failure_one_switch() {
    let (_time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    let mut prev_iter_switches;
    let mut prev_iter_issues;
    // simulamos falha na station não passando dados no weather
    for _i in 0..MAX_SOURCE_ISSUES {
        prev_iter_issues = inner.nr_of_gets_with_issues;
        prev_iter_switches = inner.nr_of_sources_switch;
        let time = inner.sched_weather.start.add_secs(30);
        inner.process_manager(time);
        assert!(inner.o_weather.is_none());
        assert!(inner.nr_of_gets_with_issues == prev_iter_issues + 1);

        if inner.nr_of_gets_with_issues >= MAX_SOURCE_ISSUES {
            assert!(inner.weather_source == WeatherSource::WebREST);

            if inner.nr_of_sources_switch < MAX_SOURCE_SWITCHES {
                assert!(inner.nr_of_sources_switch == prev_iter_switches + 1);
            } else {
                // teste do reset do counter
                assert!(inner.nr_of_sources_switch == 0);
            }
        }
        //repoe-se a source para resimular o erro
        inner.weather_source = WeatherSource::Station;
    }
}

#[test]
fn test_process_weather_with_failure_five_switches() {
    let (_time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    let mut prev_iter_switches = 0;
    let mut prev_iter_issues;
    // simulamos falha na station não passando dados no weather
    for _j in 0..MAX_SOURCE_SWITCHES {
        for _i in 0..MAX_SOURCE_ISSUES {
            prev_iter_issues = inner.nr_of_gets_with_issues;
            prev_iter_switches = inner.nr_of_sources_switch;
            let time = inner.sched_weather.start.add_secs(30);
            inner.process_manager(time);
            assert!(inner.o_weather.is_none());
            assert!(inner.nr_of_gets_with_issues == prev_iter_issues + 1);

            if inner.nr_of_gets_with_issues == MAX_SOURCE_ISSUES {
                //mudei de >=
                assert!(inner.weather_source == WeatherSource::WebREST);
            }
            //repoe-se a source para resimular o erro
            inner.weather_source = WeatherSource::Station;
        }
        if inner.nr_of_sources_switch == MAX_SOURCE_SWITCHES {
            // teste do reset do counter
            assert!(inner.nr_of_sources_switch == 0);
        } else {
            assert!(inner.nr_of_sources_switch == prev_iter_switches + 1);
        }
    }
}

#[test]
fn test_process_weather_with_failure_one_switch_stabilizing_after_one_hour() {
    let (_time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    let mut prev_iter_switches;
    let mut prev_iter_issues;
    let mut time: CtrlTime = CtrlTime(0);
    // simulamos falha na station não passando dados no weather
    for _i in 0..MAX_SOURCE_ISSUES {
        prev_iter_issues = inner.nr_of_gets_with_issues;
        prev_iter_switches = inner.nr_of_sources_switch;
        time = inner.sched_weather.start.add_secs(30);
        inner.process_manager(time);
        assert!(inner.o_weather.is_none());
        assert!(inner.nr_of_gets_with_issues == prev_iter_issues + 1);

        if inner.nr_of_gets_with_issues >= MAX_SOURCE_ISSUES {
            assert!(inner.weather_source == WeatherSource::WebREST);

            if inner.nr_of_sources_switch < MAX_SOURCE_SWITCHES {
                assert!(inner.nr_of_sources_switch == prev_iter_switches + 1);
            } else {
                // teste do reset do counter
                assert!(inner.nr_of_sources_switch == 0);
            }
        }
        //repoe-se a source para resimular o erro
        inner.weather_source = WeatherSource::Station;
    }
    // restabelece-se as condições e avança-se o relógio
    // process_data.weather_source = WeatherSource::Station;
    time = time.add_secs(3601);
    inner.o_weather = get_weather_data(1013., time);
    inner.process_manager(time);
    assert!(inner.nr_of_gets_with_issues == 0);
    assert!(inner.weather_source == WeatherSource::WebREST); //na mudança, assumimos que volta a alterar para o valor supostamente em vigor antes da falha.
}

#[test]
fn test_process_weather_one_call_with_new_day() {
    let (time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    inner.o_weather = get_weather_data(1013., time);
    let time = inner.sched_new_day.start;
    inner.sched_weather.start = time;
    inner.process_manager(time);
    assert!(inner.o_weather.is_some());
    assert!(inner.sched_new_day.start > time);
}

#[rustfmt::skip]
#[test]
fn test_process_weather_one_call_with_new_day_with_sim() {
    let (time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    inner.o_weather = get_weather_data(1013., time);
    let time = inner.sched_new_day.start;
    inner.sched_weather.start = time;
    inner.process_manager(time);
    assert!(inner.o_weather.is_some());
    assert!(inner.sched_new_day.start > time);
}

#[rustfmt::skip]
#[test]
fn test_process_weather_press_change() {
    let (time, mut _station_altitude, mut inner, _db) = create_weather_process_objects();

    let (_weather_source, _station_altitude) = prepare_data(&mut inner, WeatherSource::Station);

    inner.o_weather = get_weather_data(1013., time);
    let time = inner.sched_new_day.start;
    inner.sched_weather.start = time;
    inner.process_manager(time);
    assert!(inner.o_weather.is_some());
    assert!(inner.sched_new_day.start > time);
    //simulamos avançar 1 minuto
    let time = time.add_secs(60);
    inner.process_manager(time);
    assert!(inner.o_weather.is_some());
    //e nesta altura a variação ainda é zero.
    //simulamos avançar +1 minuto
    let time = time.add_secs(60);
    inner.process_manager(time);
    assert!(inner.o_weather.is_some());
 
    inner.sched_weather.start = time; 

    inner.process_manager(time);
    // assert!(process_data.inner.read().last_pressure_time == process_data.time.ux_ts());
}

fn prepare_data(inner: &mut WeatherInner, weather_source: WeatherSource) -> (WeatherSource, f32) {
    let station_altitude: f32;
    {
        inner.weather_source = weather_source;
        station_altitude = inner.wthr_cfg.read().geo.elev as f32;
    }
    (weather_source, station_altitude)
}

fn get_weather_data(pressure: f32, time: CtrlTime) -> Option<Weather> {
    Some(Weather {
        header: None,
        rain_period: 0.,
        rain_today: 0.,
        rain_probability: 0.,
        rain_class_forecast: 0,
        rain_week_acc: 0.,
        wind_bearing: 0.,
        wind_intensity: 1.,
        temperature: 18.,
        humidity: 50.,
        pressure,
        pressure_velocity: 0.,
        current_time_ts: time.ux_ts(),
        utcnow_dt: time.as_date_web_str_e(),
        solar_rad: 200.,
        et: 0.,
    })
}

#[test]
#[rustfmt::skip]
fn validate_weather_service_start_stop_via_web() {
    setup_start_time(CtrlTime::sys_time());
    unsafe {TESTING = true};
    let msg_broker = Arc::new(MsgBrkr::new());
    let handle_evt_mng = msg_broker.start();
    msg_broker.subscribe(MsgType::ShutDown, Subscriber::Test);
    thread::sleep(Duration::from_secs(1));
    println!("Event broker is running");
    
    let db = Persist::new();
    let app_cfg = AppCfg::new(db.clone());
    let live_since  = CtrlTime::from_utc_parts(2022, 1, 14, 0, 0, 0);
    let wthr_svc = WthrSvc::new(live_since, db.clone());

    let wthr_handle = wthr_svc.start(app_cfg.time.time_control_interval as u64, db, msg_broker.clone());

    // aqui fazemos cenas - TODO - não está terminado

    let t0 = Instant::now();
    wthr_svc.terminate();
    let _res = wthr_handle.join();
    println!("Elapsed at terminate weather service: {:?}", elapsed_dyn(t0.elapsed().as_nanos() as u64));

    thread::sleep(Duration::from_secs(1));
    println!("Event broker is still running");

    //----------------------------------
    unsafe {SHUTTING_DOWN = true};
    let _res = msg_broker.terminate();
    let _res = handle_evt_mng.join();
    println!("MsgBroker stoped");
}

#[test]
fn last_24_hrs_wind() {
    let db = Persist::new();

    if let Some(data) = WeatherHstry::build(CtrlTime::sys_time().ux_ts(), &db, "".to_owned()) {
        println!("{:?}", data);
    }
}

#[test]
#[allow(non_snake_case)]
fn test_trend() {
    let mut trend_data = TrendA::new();

    let mut NR_DATA_POINTS = 1;
    let mut press = 1000.0;
    let mut trend = 0.0;
    for _i in 0..NR_DATA_POINTS {
        trend = trend_data.trend_analysis(press);
    }
    println!("trend devia ser 0 e é {}", trend);

    let mut trend_data = TrendA::new();
    NR_DATA_POINTS = 10;
    for _i in 0..NR_DATA_POINTS {
        press += 1.;
        trend = trend_data.trend_analysis(press);
    }
    println!("trend devia ser >= 1 e é {}", trend);

    let mut trend_data = TrendA::new();
    NR_DATA_POINTS = 30;
    press = 1000.0;
    for _i in 0..NR_DATA_POINTS {
        press += 2.1;
        trend = trend_data.trend_analysis(press);
    }
    for _i in 0..NR_DATA_POINTS {
        press -= 1.1;
        trend = trend_data.trend_analysis(press);
    }
    println!("trend devia ser >= 2 e é {}", trend);

    let mut trend_data = TrendA::new();
    NR_DATA_POINTS = 60;
    for _i in 0..NR_DATA_POINTS {
        press -= 1.;
        trend = trend_data.trend_analysis(press);
    }
    println!("trend devia ser <0 e é {}", trend);
}
