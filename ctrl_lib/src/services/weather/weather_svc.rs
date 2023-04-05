// DESIGN
// We have two sources
// 1. station
// 3. and if station in maintenance, tempest site
// 4. Weather service does not work with simulated weather
//
// Notes
// Darksky bought by  Apple, ceasing open api after 31/Mar/2023.
// WeatherUnderground bought ~2018 by IBM
// so...I had to buy something

use std::io::Error;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{io::ErrorKind, net::UdpSocket};
use std::{sync::Arc, thread};

use crate::{config::wthr_cfg::*, utils::*};
use crate::{db::db_sql_lite::*, app_time::ctrl_time::*};
use crate::services::msg_broker::msg_brkr_svc::*;
use crate::services::weather::sources::tempest::data_structs::*;
use crate::services::weather::weather_inner::*;
#[cfg(debug_assertions)]
use crate::utils::THREAD_COUNT;
use crate::{lib_serde::data_from_str, utils::arc_rw};
use crate::{log_error, logger::*, log_info};

use ctrl_prelude::{globals::*, string_resources::*, error::*};

pub const UDP_ERRORS: [ErrorKind; 3] = [ErrorKind::WouldBlock, ErrorKind::TimedOut, ErrorKind::Interrupted];

/// Dimension: Stack = 24
pub struct WthrSvc {
    wthr_cfg : SWthrCfg,
    wthr_stop: Arc<AtomicBool>,
}

impl WthrSvc {
    #[inline]
    pub fn new(live_since: CtrlTime, db: Persist) -> WthrSvc {
        let wthr_cfg = arc_rw(WthrCfg::new(db, live_since));

        WthrSvc {
            wthr_cfg,
            wthr_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    // weather thread is autonomous
    // With station, listen for udp packets from station
    // else, read from tempest site
    // udp listener also works as a timer, to ckeck for the new day event
    #[inline]
    #[rustfmt::skip]
    pub fn start(&self, interval: u64, db: Persist, msg_broker: SMsgBrkr) -> thread::JoinHandle<()> {
        let builder = thread::Builder::new().name(WTHR_SERVICE_THREAD.to_owned()).stack_size(STACK_SIZE_UNIT * 131);  //machine learning array is big

        let wthr_stop = self.wthr_stop.clone();
        let mut time: CtrlTime = CtrlTime::sys_time();
        let address: String;
        let whtr_cfg = self.wthr_cfg.clone();

        {
            let wc = self.wthr_cfg.read();
            
            if !unsafe { TESTING } { 
                address = wc.address_tempest.clone();
            }else{
                address = wc.address_tempest_test.clone();
            }
        }

        let interval = interval * GIGA_U;

        let res = builder
            .spawn(move || {
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT += 1; } }
                // socket is also used to control timming, even when the source is not the weather station
                let socket = UdpSocket::bind(address).expect("Problema a ligar ao port da estação tempest.");

                log_info!(INFO_WTHR_THREAD_START);

                let mut whtr_inner = WeatherInner::new(time, db.clone(), msg_broker, whtr_cfg);

                let mut is_time_out :bool;
                let mut error: ErrorKind;
                let mut temp_buffer: &str;
                let mut w_station_data: Tempest;
                
                let mut deadline_duration: Duration;
                let mut udp_read_result: Result<(usize, SocketAddr), Error>;
                loop {
                    is_time_out = false;
                    // run one time at start up
                    deadline_duration = get_deadline_duration(interval);
                    _ = socket.set_read_timeout(Some(deadline_duration));
                    udp_read_result = socket.recv_from(&mut whtr_inner.buf);
                    // verify each second (defined interval) if it should terminate, and exit if it is
                    if wthr_stop.load(Ordering::Relaxed) { 
                        drop(socket);
                        break; 
                    }
                    time = CtrlTime::sys_time();
                    match udp_read_result {
                        Ok(req) => {
                            temp_buffer = std::str::from_utf8(&whtr_inner.buf[..req.0]).unwrap();
                            w_station_data = data_from_str(temp_buffer).unwrap();
                            // if last weather msg was not processed it will be lost
                            if let Tempest::ObsSt(data) = &w_station_data {
                                whtr_inner.o_weather = Some(whtr_inner.station.get_weather(time, &data.obs[0], whtr_inner.station_altitude));
                            }
                            // ignore all other msgs for now.  just looking for ObsSt
                                // else {}
                            // } 
                        }
                        Err(e) => {
                            error = e.kind();
                            if UDP_ERRORS.contains(&error){
                                is_time_out = true;
                            }else{
                                log_error!(build_error(&e));
                            }
                        }
                    };
                    if is_time_out  {
                        whtr_inner.process_manager(time);
                        whtr_inner.o_weather = None;
                    }
                    
                } // end loop

                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT -= 1; } }
                log_info!(INFO_WTHR_THREAD_STOP);
                // println!("saiu da thread da metereologia")
            });
            // if let Err(_res)= &res{
            //     println!("{:?}",_res);
            // }
            res.unwrap()
    }

    #[inline]
    #[rustfmt::skip]
    pub fn terminate(&self) { 
        self.wthr_stop.store(true, Ordering::Relaxed); 
    }

    #[inline]
    #[rustfmt::skip]
    pub fn get_context(&self) -> SWthrCfg { self.wthr_cfg.clone() }

    // REVIEW it seems that this function is not called.  It should be called from the clientsvc when updating data...
    #[inline]
    pub fn set_context(&self, weather_config_copy: &WthrCfg, time: CtrlTime) {
        let mut config_lock = self.wthr_cfg.write();

        config_lock.alrt_thresholds = weather_config_copy.alrt_thresholds.clone();
        config_lock.changed = true;
        config_lock.save_if_updated(time);
    }
}
