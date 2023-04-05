// DESIGN
// A ideia é ter dois mecanismos
// 1. que lê diretamente da estação
// 2. que lê de uma fonte alternativa caso qualquer coisa falhe na estação enquanto não se resolve o tema.  Isto implica ler do site do tempest.
// 3. e qd a estação está em manutenção, ler do site, para mantermos dados
// 4. O serviço weather não trabalha com o tempo simulado.
//
// Notas
// O Darksky foi comprado pela Apple, e vai deixar de ser publico a partir de 31/Mar/2023.
// o WeatherUnderground comprado ~2018 pela IBM

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{io::ErrorKind, net::UdpSocket};
use std::{sync::Arc, thread};

use crate::app_time::ctrl_time::*;
use crate::config::wthr_cfg::*;
use crate::controller_sync::{new_day_and_db_mnt_sync, NEW_DAY_SIG};
use crate::data_structs::msgs::{alert::*, int_message::*, weather::*};
use crate::data_structs::sensor::{daily_value::*, snsor::*};
use crate::db::db_sql_lite::*;
use crate::services::msg_broker::svc::*;
use crate::services::weather::algorithms::dew_point;
use crate::services::weather::sources::tempest::data_structs::*;
use crate::services::weather::{db_model::*, weather_error::*, weather_inner::*};
#[cfg(debug_assertions)]
use crate::utils::THREAD_COUNT;
use crate::{lib_serde::data_from_str, utils::arc_rw};
use crate::{log_error, log_warn, logger::*, log_info};

use ctrl_prelude::error::build_error;
use ctrl_prelude::{globals::*, string_resources::*};

pub const MAX_SOURCE_SWITCHES:u16 = 5;
pub const MAX_SOURCE_ISSUES:u16 = 10;
/// Dimensão: Stack = 56 ; Heap = 744
pub struct WthrSvc {
    inner: WeatherInnerShared,
    wthr_stop: Arc<AtomicBool>,
    pub db: Persist,
}

impl WthrSvc {
    #[inline]
    pub fn new(msg_broker: SMsgBrkr, time: CtrlTime, db: Persist, live_since: CtrlTime) -> WthrSvc {
        WthrSvc {
            inner: arc_rw(WeatherInner::new(time, db.clone(), live_since, msg_broker)),
            wthr_stop: Arc::new(AtomicBool::new(false)),
            db,
        }
    }

    // lança a thread de meterologia que funciona de forma autónoma
    // Se houver estação, fica á escuta
    // senão utiliza os planos de sources alternativos
    // o udp listener serve também de timer, de forma a controlar a passagem do tempo, e tratar do new day
    //tested
    #[inline]
    #[rustfmt::skip]
    pub fn start(&self, interval: u64) -> thread::JoinHandle<()> {
        let builder = thread::Builder::new().name(WTHR_SERVICE_THREAD.to_owned()).stack_size(STACK_SIZE_UNIT);

        let wthr_stop = self.wthr_stop.clone();

        let inner = self.inner.clone();
        let address: String;
        let weather_source: WeatherSource;
        let station_altitude: f32;
        {
            let wc = self.inner.read();
            address = wc.wthr_cfg.read().address_tempest.clone();
            weather_source = wc.wthr_cfg.read().weather_source;
            station_altitude = wc.wthr_cfg.read().geo.elev as f32;
        }
        let db = self.db.clone();
        let interval = interval * GIGA_U;

        builder
            .spawn(move || {
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT += 1; } }
                let mut receiver_deadline_duration_nanos: u64;
                // utilizamos também o socket para controlo dos timings, mesmo que a source não seja a estação tempest
                let socket = UdpSocket::bind(address).expect("Problema a ligar ao port da estação tempest.");

                log_info!(INFO_WTHR_THREAD_START);
                let mut process_data : ProcessManagerData = ProcessManagerData { 
                    // process_weather: false, 
                    weather_source, 
                    buf: vec![0_u8; BUF_SIZE], 
                    inner: inner.clone(), 
                    station_altitude, 
                    msg_broker: inner.read().msg_broker.clone(), 
                    nr_of_gets_with_issues: 0, 
                    db, 
                    nr_of_sources_switch: 0,
                    o_weather: None,
                    time: CtrlTime(0),
                    time_last_source_failure: CtrlTime(0),
                };
                let mut is_time_out :bool;
                loop {
                    is_time_out = false;
                    // executa sempre uma vez no arranque
                    receiver_deadline_duration_nanos = CtrlTime::sys_time_duration().subsec_nanos() as u64;
                    //acerta para o segundo certo seguinte tendo em conta o intervalo configurado
                    let deadline_duration = Duration::from_nanos(interval - receiver_deadline_duration_nanos.rem_euclid(interval));
                    _ = socket.set_read_timeout(Some(deadline_duration));
                    let udp_read_result = socket.recv_from(&mut process_data.buf);
                    process_data.time = CtrlTime::sys_time();
                    match udp_read_result {
                        Ok(req) => {
                            let s = std::str::from_utf8(&process_data.buf[..req.0]).unwrap();
                            // println!("recv: {:?} {:?}", req, s);
                            let w_station_data: Tempest = data_from_str(s).unwrap();
                            let wi = process_data.inner.read();
                            // se por ventura não foi processada a ultima info do weather, será esmagada
                            if let Tempest::ObsSt(data) = &w_station_data {
                                process_data.o_weather = Some(wi.station.get_weather(process_data.time, &data.obs[0], process_data.station_altitude));
                            }
                            // ignoramos as demais mensagens. para já.  Só me interessa as ObsSt
                                // else {}
                            // } 
                        }
                        Err(e) => {
                            // no linux pela doc, pode ser diferente
                            if e.kind() != ErrorKind::TimedOut {
                                log_error!(build_error(&e));
                            } else {
                                is_time_out = true;
                            }
                        }
                    };
                    // println!("tempo do proc: {}  tempo do schedule: {}", time.as_rfc3339_str_e(), wi.sched_weather.start.as_rfc3339_str_e());
                    if is_time_out  {
                        process_manager(&mut process_data);
                        process_data.o_weather = None;
                    }
                    if wthr_stop.load(Ordering::Relaxed) { break; }
                } // end loop

                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT -= 1; } }
                log_info!(INFO_WTHR_THREAD_STOP);
            })
            .unwrap()
    }

    #[inline]
    #[rustfmt::skip]
    pub fn terminate(&self) { self.wthr_stop.store(true, Ordering::Relaxed); }

    #[inline]
    #[rustfmt::skip]
    pub fn get_context(&self) -> SWthrCfg { self.inner.read().wthr_cfg.clone() }

    // REVIEW á aqui um tema que parece que isto não é chamado de lado nenhum, mas devia ser chamado do cliente quando atualizamos dados
    #[inline]
    pub fn set_context(&mut self, weather_config_copy: &WthrCfg, time: CtrlTime) {
        let wi_lock = self.inner.write();
        let mut config_lock = wi_lock.wthr_cfg.write();

        config_lock.alrt_thresholds = weather_config_copy.alrt_thresholds.clone();
        config_lock.changed = true;
        config_lock.save_if_updated(time);
    }
}

#[inline]
pub fn snd_alert(alert_type: AlertType, value: f32, time: CtrlTime, msg_broker: &SMsgBrkr) {
    let desc = info_whtr_alert_rcvd(&alert_type.to_string());
    msg_broker.reg_int_msg(MsgData::Alert(Alert::new(alert_type, value)), time, &desc);
}

#[inline]
pub fn get_weather_from_site(time: CtrlTime, inner: &WeatherInnerShared, station_altitude: f32) -> Option<Weather> {
    let wi = inner.read();
    let wc = wi.wthr_cfg.read();
    if wi.sched_weather.is_time_to_run(time) {
        match wi.site.get_weather(time, wc.token_tempest.clone(), wc.device_id_tempest.clone(), station_altitude) {
            Ok(w) => Some(w),
            Err(e) => {
                log_error!(build_error(&e));
                None
            }
        }
    } else {
        None
    }
}

pub struct ProcessManagerData {
    pub weather_source: WeatherSource,
    pub o_weather: Option<Weather>,
    pub buf: Vec<u8>,
    pub inner: WeatherInnerShared,
    pub station_altitude: f32,
    pub msg_broker: SMsgBrkr,
    pub nr_of_gets_with_issues: u16,
    pub db: Persist,
    pub nr_of_sources_switch: u16,
    pub time: CtrlTime,
    pub time_last_source_failure: CtrlTime,
}

#[inline]
#[rustfmt::skip]
pub fn process_manager(process_data: &mut ProcessManagerData) {
    let is_time: bool;
    let is_new_day: bool;
    {
        let wi = process_data.inner.read();
        is_time = wi.sched_weather.is_time_to_run(process_data.time);
        is_new_day = wi.sched_new_day.is_time_to_run(process_data.time);
    }
    // println!("tempo do proc: {}  tempo do schedule: {}", time.as_rfc3339_str_e(), wi.sched_weather.start.as_rfc3339_str_e());
    if is_time {
        // Vamos buscar a weather data, caso não se esteja a ler da estação
        match process_data.weather_source {
            // Não fazemos nada porque o process data têm sempre a ultima leitura que a estação fez.
            WeatherSource::Station => (),
            // em todas as outras sources vamos avaliar se é tempo de fazer alguma coisa - para aqui virá sempre como timeout
            WeatherSource::WebREST => {
                process_data.o_weather = get_weather_from_site(process_data.time, &process_data.inner, process_data.station_altitude);
            }
            WeatherSource::Simulation => {
                let wi = process_data.inner.read();
                // println!("tempo do proc: {}  tempo do schedule: {}", time.as_rfc3339_str_e(), wi.sched_weather.start.as_rfc3339_str_e());
                process_data.o_weather = Some(wi.simulation.get_weather(process_data.time));
            }
        }
        if is_new_day{
            // tratamos do new day antes de inserir na bd a informação metereológica
            new_day_and_db_mnt_sync();
            {
                let mut wi = process_data.inner.write();
                // wait for new day processing or db maint - at night
                NEW_DAY_SIG.read().reset();
                let res = wi.prep_new_day(process_data.time);
                NEW_DAY_SIG.read().set();
                //avançamos para o dia/evento seguinte.
                _ = wi.sched_new_day.set_next_event().map_err(|e| log_error!(build_error(&e)));
                // Se houver erro damos nota aos clientes
                res.unwrap_or_else(|e| { process_data.msg_broker.snd_error_to_clients(&e.to_string(), ""); });
            }
        }
        if process_data.o_weather.is_some(){
            if process_data.nr_of_gets_with_issues > 0 && process_data.time > process_data.time_last_source_failure.add_secs(3600){
                // ao fim de 1 hora tentamos voltar á source original.  Isto pode implicar andar a mudar de hora a hora se o tema não estiver resolvido
                // mas se o tema for intermitente, desta forma "ultrapassa-se" sem dramas
                // se for intermitente ou definitivo, pelas mensagens de erro fica o alerta de que se têm que analisar a situação
                process_data.nr_of_gets_with_issues = 0;
                _ = switch_source(process_data);
            }
            let weather = process_data.o_weather.as_mut().unwrap();
            let mut wi = process_data.inner.write();
            // faz coisas com o weather
            // calculamos a velocidade horária de variação da pressão e preparamos dados derivados
            if wi.last_pressure == 0. {
                wi.last_pressure = weather.pressure;
                wi.last_pressure_time = process_data.time.ux_ts();
            } else {
                let denominador = process_data.time.ux_ts() - wi.last_pressure_time;
                weather.pressure_velocity = 0.;
                if denominador > 0 {
                    // div by zero prevention
                    weather.pressure_velocity = (weather.pressure - wi.last_pressure) * 3600. / (process_data.time.ux_ts() - wi.last_pressure_time) as f32;
                }// else{
                    // Isto em tese nunca acontece - o sched avança sempre no tempo.  E como o sched avança, se for o mesmo tempo quer dizer que não é apanhado pelo is_time
                    // Só em caso de falha no update da variável e/ou inconsistência na BD
                    // Pelo sim pelo não fica a validação para qq cenário que não esteja a ver agora
                // }
                wi.last_pressure = weather.pressure;
                wi.last_pressure_time = process_data.time.ux_ts();
            }

            wi.sensor_data_buf.clear();
            wi.sensor_data_buf.push(SensorValue::new(Sensor::Rain as u8, process_data.time, weather.rain_period));
            wi.sensor_data_buf.push(SensorValue::new(Sensor::Temp as u8, process_data.time, weather.temperature));
            wi.sensor_data_buf.push(SensorValue::new(Sensor::WindBearing as u8, process_data.time, weather.wind_bearing));
            wi.sensor_data_buf.push(SensorValue::new(Sensor::WindSpeed as u8, process_data.time, weather.wind_intensity));
            wi.sensor_data_buf.push(SensorValue::new(Sensor::Humidity as u8, process_data.time, weather.humidity));
            wi.sensor_data_buf.push(SensorValue::new(Sensor::Pressure as u8, process_data.time, weather.pressure));
            wi.sensor_data_buf.push(SensorValue::new(Sensor::SolarRadiation as u8, process_data.time, weather.solar_rad));
            wi.sensor_data_buf.push(SensorValue::new(Sensor::DewPoint as u8, process_data.time, dew_point(weather.temperature as f64, weather.humidity as f64) as f32));

            // atualiza-se a base de dados

            //  In simulation mode, the program can start at a time where there is already some info in the DB
            //  In production mode that's not supposed to happen - thats an error that it is logged
            clear_sim_mode_prev_data(&mut wi.sensor_data_buf, &process_data.db);

            if process_data.db.ins_sensor_data_batch(&wi.sensor_data_buf).is_err() {
                let msg = WeatherError::CantInsertDailyMeasures.to_string();
                log_error!(msg);
            }
            // enviamos o weather para quem estiver á escuta
            process_data.msg_broker.reg_int_msg(MsgData::Weather(weather.clone()), process_data.time, DESC_INFO_WEATHER);
            // avaliamos se há alertas
            {
                let mut config_lock = wi.wthr_cfg.write();
                if config_lock.alrt_thresholds.is_rain_alert(weather.rain_period) {
                    snd_alert(AlertType::RAIN, weather.rain_period, process_data.time, &process_data.msg_broker);
                }
                if config_lock.alrt_thresholds.is_wind_alert(weather.wind_intensity) {
                    snd_alert(AlertType::WIND, weather.wind_intensity, process_data.time, &process_data.msg_broker);
                }
                config_lock.save_if_updated(process_data.time);
            }
            //avançamos para o evento seguinte.
            _ = wi.sched_weather.set_next_event().map_err(|e| log_error!(build_error(&e)));
        } else {
            // foi tempo de ir buscar a metereologia mas não se obteve dados....
            warn!("Não se conseguiu obter dados da metereologia na fonte: {}", process_data.weather_source);
            process_data.nr_of_gets_with_issues += 1;
            process_data.time_last_source_failure = process_data.time;  // registamos o tempo da ultima falha
            if process_data.nr_of_gets_with_issues >= MAX_SOURCE_ISSUES {
                // já houve 10 falhas na estação (não se controla se consecutivas ou intermitentes) sem conseguir obter dados.
                // muda-se a fonte de station(udp) para rest e vice-versa
                let no_change = switch_source(process_data);
                if !no_change {
                    warn!("Alterou-se temporariamente a fonte dos dados da metereologia para {}", process_data.weather_source);
                    process_data.nr_of_sources_switch += 1;
                }
                if process_data.nr_of_sources_switch >= MAX_SOURCE_SWITCHES {
                    // quer dizer que andamos a alterar as sources sem sucesso á pelo menos 1 hora - tem que se perceber o que se passa
                    let msg: &str = "Não temos informação metereológica á uma hora pelo menos.  Validar o que se passa.";
                    log_warn!(msg);
                    process_data.msg_broker.snd_error_to_clients(msg, "");
                    process_data.nr_of_sources_switch = 0;  //reset ao counter dos switches
                }
            }
        }
    }
}

#[inline]
fn switch_source(process_data: &mut ProcessManagerData)->bool {
    let mut no_change = false;
    let wi = process_data.inner.write();
    let mut wc = wi.wthr_cfg.write();
    match process_data.weather_source {
        WeatherSource::Station => process_data.weather_source = WeatherSource::WebREST,
        WeatherSource::WebREST => process_data.weather_source = WeatherSource::Station,
        _ => no_change = true,  // o que sobra é o simulation, mas aí nunca falha, em tese, a não ser por erro de programação
    }
    wc.weather_source = process_data.weather_source;
    no_change
}
