use std::sync::Arc;
use std::thread;
use std::time::Duration;

use ctrl_prelude::error::build_error;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use crate::app_time::ctrl_time::*;
use crate::controller_sync::new_day_and_db_mnt_sync;
use crate::data_structs::concurrent_queue::*;
use crate::data_structs::msgs::int_message::{IntMessage, MsgData, MsgType};
use crate::services::electronics::{actuator::*, db_model::*, device_lawn::*, devices_and_scenes::*, error::*, model::shelly_structs::*, valve_state::*};
use crate::services::msg_broker::{msg_brkr_svc::*, subscriber::*};
#[cfg(debug_assertions)]
use crate::utils::THREAD_COUNT;
use crate::utils::{conv_int, StringUtils};
use crate::{db::db_sql_lite::*, lib_serde::*, logger::*, string_concat::*};
use crate::{ifmt, log_error, log_info, log_warn};
use ctrl_prelude::{domain_types::*, globals::*, string_resources::*};

pub type DevicesMapList = FxHashMap<String, usize>;

pub struct DevicesSvc {
    pub inner: Arc<RwLock<InnerDevices>>,
    pub evt_in: SMtDeque,
    pub msg_brkr: SMsgBrkr,
}

pub type SDevicesSvc = Arc<DevicesSvc>;

impl DevicesSvc {
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new(db: &Persist, msg_brkr: SMsgBrkr) -> Self {
        let inner = Arc::new(RwLock::new(InnerDevices::new(db, msg_brkr.clone())));
        DevicesSvc {
            inner,
            evt_in: Arc::new(MtDeque::new()),
            msg_brkr,
        }
    }

    #[inline]
    pub fn terminate(&self) {
        self.evt_in.terminate();
    }

    #[inline]
    #[rustfmt::skip]
    pub fn start(&self) -> thread::JoinHandle<()> {
        // let builder = thread::Builder::new().name(DEV_SERVICE_THREAD.to_owned()).stack_size(1 * STACK_SIZE_UNIT);
        let builder = thread::Builder::new().name(DEV_SERVICE_THREAD.to_owned()).stack_size(STACK_SIZE_UNIT);
        let inner_svc = self.inner.clone();
        let evt_in = self.evt_in.clone();
        let msg_brkr = self.msg_brkr.clone();

        builder
            .spawn(move || {
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT += 1; } }
                log_info!(INFO_DEV_THREAD_START);
                msg_brkr.register_in_broker(Subscriber::Dev, evt_in.clone());
                msg_brkr.subscribe(MsgType::Shellies, Subscriber::Dev); // inform client
                msg_brkr.subscribe(MsgType::GetDevicessAndScenes, Subscriber::Dev); // inform client
                msg_brkr.subscribe(MsgType::SetActuatorOrScene, Subscriber::Dev); // inform client
                let mut time: CtrlTime;
                let mut opt_msg: Option<IntMessage>;
                // keep track of scenes
                let mut active_scenes: Vec<(SCENE_ID, thread::JoinHandle<()>)> = Vec::new();
                loop {
                    opt_msg = evt_in.recv();
                    if let Some(msg) = opt_msg {
                        new_day_and_db_mnt_sync();
                        time = CtrlTime::sys_time();
                        match msg.data {
                            MsgData::GetDevicessAndScenes => {
                                msg_brkr.reg_int_msg(MsgData::RspDevicesAndScenes(inner_svc.read().devices_and_scenes.clone()), time);
                            }
                            MsgData::SetActuatorOrScene(mut actuator_data) => {
                                match actuator_data.actuator_type {
                                    ActuatorType::Actuator => {
                                        if inner_svc.write().apply_device_cmd(actuator_data.clone()) {
                                            actuator_data.status = Some(r#"{"result":"true"}"#.to_owned());
                                        } else {
                                            actuator_data.status = Some(r#"{"result":"false"}"#.to_owned());
                                        }
                                    }
                                    ActuatorType::Scene => {
                                        // TODO, provavelmente tenho que lançar uma thread que vai fazer coisas em função do cenário
                                        let data = actuator_data.clone();
                                        let inner = inner_svc.clone();
                                        let id = data.id;
                                        let builder = thread::Builder::new()
                                            .name(string_concat!(DEV_SERVICE_THREAD, ":SCENE:", ifmt!(id)).to_owned())
                                            //.stack_size(1 * STACK_SIZE_UNIT);
                                            .stack_size(STACK_SIZE_UNIT);
                                        let jh = builder.spawn(move || {
                                            #[cfg(debug_assertions)]
                                            { unsafe { THREAD_COUNT += 1; } }
                                            Self::apply_scene_cmd(data.clone(), inner);
                                            #[cfg(debug_assertions)]
                                            { unsafe { THREAD_COUNT -= 1; } }
                                    }).unwrap();
                                        active_scenes.push((id, jh));
                                    }
                                }
                                msg_brkr.reg_int_msg(MsgData::RspActuatorOrScene(actuator_data), time);
                            }
                            MsgData::Shellies(topic, data) => {
                                inner_svc.write().process_shelly_data(topic, data);
                            }
                            _ => (), //nop
                        }
                    } else {
                        // if terminating, do nothing and break
                        break;
                    }
                } //loop end
                  // On exit verify active scenes, stop the process, and wait for all scenes handlers
                while !active_scenes.is_empty() {
                    let t = active_scenes.remove(0);
                    let result = t.1.join();
                    if let Err(e) = result {
                        log_error!(format!("Cena: {} com erro: {:?}", t.0, e));
                    }
                }
                log_info!(INFO_DEV_THREAD_STOP);
                #[cfg(debug_assertions)]
                { unsafe { THREAD_COUNT -= 1; } }
            })
            .unwrap()
    }

    /// this function is called in a thread and it may sleep for a few minutes
    #[inline]
    pub fn apply_scene_cmd(actuator_data: ActuatorData, inner: Arc<RwLock<InnerDevices>>) {
        // DESIGN idea
        // in the calling thread we:
        // keep track of active scenes
        // launch each scene on its own thread

        // and in here, the launched thread we:
        // each scene have its own custom logic
        // we just launch the scene.  status update will be followed by mqtt messages
        // scene start / stop is controlled by the inner devices
        // on
        match actuator_data.id {
            0 => {
                // Open gate automatically during final approach
                let dev_id = &inner.read().devices_and_scenes.scenes[0].devices[0];
                let desc = &inner.read().devices_and_scenes.scenes[0].desc;
                let data = ActuatorData {
                    id: *dev_id,
                    actuator_type: ActuatorType::Actuator,
                    cmd: actuator_data.cmd,
                    status: None,
                };
                if !inner.read().apply_device_cmd(data) {
                    warn!("Tema com o cenário {}", desc);
                }
            }
            id @ 1..=5 => {
                // this works because we know what data is in the table and we control the ids.
                // and backend is manually synced with the client.
                // This is not intended to work for arbitrary ids/scenes.  For now I see that as an unnecessary complication
                let desc = &inner.read().devices_and_scenes.scenes[id as usize].desc.clone();
                let devs = &inner.read().devices_and_scenes.scenes[id as usize].devices.clone();
                if !apply_scene_instruction_to_devices(devs, actuator_data, inner) {
                    warn!("Tema com o cenário {}", desc);
                }
            }
            // 1 => All shutters
            // 2 => Suite shutters
            // 3 => Guest shutters
            // 4 => Living room shutters
            // 5 => 1st floor shutters
            _ => log_warn!("O cliente enviou um id de cenário desconhecido."), //nop
        };
    }
}

#[inline]
fn apply_scene_instruction_to_devices(devs: &Vec<u16>, actuator_data: ActuatorData, inner: Arc<RwLock<InnerDevices>>) -> bool {
    let mut result = false;
    for d in devs {
        let data = ActuatorData {
            id: *d,
            actuator_type: ActuatorType::Actuator,
            cmd: actuator_data.cmd,
            status: None,
        };
        result &= inner.read().apply_device_cmd(data);
        if actuator_data.cmd == ActuatorCommand::Up || actuator_data.cmd == ActuatorCommand::Down {
            // hard coded.  We wait 15 secs before issuing the next command, so we have at most two rollers working simultaneously
            // REVIEW - a candidate to a configuration entry
            // considering that each roller takes ~20secs to full open/close
            // just for safety regarding load on the roller circuit.
            thread::sleep(Duration::new(15, 0));
        } //else{} - else is cmd stop, and that we can do without wait.
    }
    result
}

pub struct InnerDevices {
    pub lawn_devices: DeviceWateringList,
    pub devices_and_scenes: DevicesAndScenesData,
    pub desc_to_device: DevicesMapList, //to map to devices and scenes index
    pub msg_brkr: SMsgBrkr,
}

const REQ_TIMEOUT: u64 = 5;
const DESC_PORTAO_GRANDE: &str = "Portao Grande";

impl InnerDevices {
    #[inline]
    #[rustfmt::skip]
    pub fn new(db: &Persist, msg_brkr: SMsgBrkr) -> InnerDevices {
        let mut lawn_devices: DeviceWateringList = FxHashMap::default();
        let mut desc_to_device: DevicesMapList = FxHashMap::default();
        let result = db.get_devices(&mut lawn_devices);
        if let Err(e) = result { log_error!(e); }

        let res = db.get_devices_and_scenes(&mut desc_to_device);
        let devices_and_scenes: DevicesAndScenesData;
        // DESIGN NOTE
        // Devices and scenes are loaded at startup.
        // Any changes in the BD outside the program implies service restart
        // Devices and scenes ids have to be the same in the backend and frontend - implies programmer discipline
        // Its no use, at this point, to do effort to improve that
        if let Ok(data) = res {
            devices_and_scenes = data;
        } else {
            log_error!("Não foi possivel obter os dados dos devices e dos cenários.");
            devices_and_scenes = DevicesAndScenesData::new();
        }
        InnerDevices { lawn_devices, devices_and_scenes, desc_to_device, msg_brkr, }
    }

    #[inline]
    pub fn turn_on_physical_sec(&self, device_id: DEVICE_ID) -> bool {
        self.apply_lawn_cmd(device_id, ActuatorCommand::On)
    }

    #[inline]
    pub fn turn_off_physical_sec(&self, device_id: DEVICE_ID) -> bool {
        self.apply_lawn_cmd(device_id, ActuatorCommand::Off)
    }

    #[inline]
    #[rustfmt::skip]
    pub fn relay_status(&self, device_id: DEVICE_ID) -> RelayState {
        let o_device = self.lawn_devices.get(&device_id);
        if let Some(device) = o_device {
            let url = string_concat!("http://", device.ip, "/", device.get_status);
            let get_result = minreq::get(url).with_timeout(REQ_TIMEOUT).send();
            match get_result {
                Ok(response) => {
                    let body = response.as_str().unwrap();
                    let conv_result = data_from_str::<Status>(body);
                    match conv_result {
                        Err(e) => {
                            log_error!(e);
                            RelayState::Error
                        }
                        Ok(status) => { if status.relays[0].ison { RelayState::Open } else { RelayState::Closed }
                        }
                    }
                }
                Err(err) => {
                    log_error!(err);
                    println!("{err}");
                    RelayState::Error
                }
            }
        } else {
            log_error!(DeviceError::DeviceNotRegistered(device_id));
            RelayState::Error
        }
    }

    /// [Input/Output] <br><br>
    /// Call the watering relay <br>
    #[inline]
    fn apply_lawn_cmd(&self, device_id: DEVICE_ID, cmd: ActuatorCommand) -> bool {
        let mut result = false;
        let url: String;
        let o_device = self.lawn_devices.get(&device_id);
        if let Some(device) = o_device {
            url = device.get_cmd(cmd);

            let get_result = minreq::get(&url).with_timeout(REQ_TIMEOUT).send();
            match get_result {
                Ok(response) => {
                    let body = response.as_str().unwrap();
                    let conv_result = data_from_str::<RelayStatus>(body);
                    match conv_result {
                        Ok(relay_status) => {
                            if relay_status.is_cmd_response_ok(cmd) {
                                let accao = if relay_status.ison { "abrir" } else { "fechar" };
                                result = true;
                                debug!("A {} a valvula: setor {}", accao, &ifmt!(device_id));
                            } else {
                                let msg = format!("O relay não mudou para o estado '{}'", cmd);
                                log_error!(DeviceError::CommandError(url, msg));
                            }
                        }
                        Err(e) => log_error!(e),
                    }
                }
                Err(err) => {
                    println!("{err}");
                    log_error!(err);
                }
            };
        }
        result
    }

    #[inline]
    pub fn apply_device_cmd(&self, actuator_data: ActuatorData) -> bool {
        let id = actuator_data.id;
        let mut result = false;

        let device = &self.devices_and_scenes.devices[(id - unsafe { NR_OF_WATER_DEVICES }) as usize];

        let url = device.get_cmd(actuator_data.cmd);
        let get_result = minreq::get(&url).with_timeout(REQ_TIMEOUT).send();
        match get_result {
            Ok(response) => {
                let body = response.as_str().unwrap();
                match device.device_type {
                    DeviceType::Relay => {
                        // general cases
                        // REVIEW this have to be tested, for the specific external gate case,
                        // have to study how to "pulse" for on and off triggers automatically.  
                        // First idea is to configure that directly in the shelly, 
                        // but if that doesn't work, have to do something here (else part)
                        if device.desc != DESC_PORTAO_GRANDE {
                            let conv_result = data_from_str::<RelayStatus>(body);
                            match conv_result {
                                Ok(relay_status) => {
                                    if relay_status.is_cmd_response_ok(actuator_data.cmd) {
                                        let accao = if relay_status.ison { "abrir" } else { "fechar" };
                                        result = true;
                                        debug!("A {} o relay do {}", accao, device.desc);
                                    } else {
                                        let msg = format!("O device {} não reagiu ao comando '{}'", device.desc, actuator_data.cmd);
                                        log_error!(DeviceError::CommandError(url, msg));
                                    }
                                }
                                Err(e) => log_error!(e),
                            }
                        } else {
                            // gate specific case
                            // if first idea does not work, issue two different commands on and then off, timed in some way
                            //
                            // also, understand how to capture (well mqtt) status of the full open/closed magnetic swithces
                            // to know exactly the state, havae situation awareness, and update the UI
                            let conv_result = data_from_str::<RelayStatus>(body);
                            match conv_result {
                                Ok(relay_status) => {
                                    if relay_status.is_cmd_response_ok(actuator_data.cmd) {
                                        let accao = if relay_status.ison { "abrir" } else { "fechar" };
                                        result = true;
                                        debug!("Portão grande a {}", accao);
                                        // and here maube some more stuff to do
                                        // TODO i4 controldevice for full situation awareness
                                    } else {
                                        let msg = format!("O device não reagiu ao comando '{}'", actuator_data.cmd);
                                        log_error!(DeviceError::CommandError(url, msg));
                                    }
                                }
                                Err(e) => log_error!(e),
                            }
                        }
                    }
                    DeviceType::Roller => {
                        // NOTE - roller devices have to be calibrated, and maximum operating time wel defined to minimize motor effort risks
                        // although I think that the motor has built in protection and turn off automatically, I am not sure
                        let conv_result = data_from_str::<Roller2PMStatus>(body);
                        match conv_result {
                            Ok(roller_status) => {
                                if roller_status.is_cmd_response_ok(actuator_data.cmd) {
                                    let accao = match actuator_data.cmd {
                                        ActuatorCommand::Up => "abrir",
                                        ActuatorCommand::Stop => "parar",
                                        ActuatorCommand::Down => "fechar",
                                        _ => "Algo estranho que ainda não percebi.",
                                    };
                                    result = true;
                                    debug!("A {} {}", accao, device.desc);
                                } else {
                                    let msg = format!("O device não reagiu ao comando '{}'", actuator_data.cmd);
                                    log_error!(DeviceError::CommandError(url, msg));
                                }
                            }
                            Err(e) => log_error!(e),
                        }
                    }
                    DeviceType::TriggerSwitch => (),
                }
            }
            Err(err) => {
                println!("{err}");
                log_error!(err);
            }
        };
        result
    }

    #[inline]
    pub fn process_shelly_data(&self, topic: String, data: String) {
        //TODO process devices data - still to decide on what to do 
        // First idea (2023/Mar/05)
        //  - save information from power consumption each minute for future analysis
        //  - save temperature info for future usage (and eventually have a smarter heat pump)
        //  - in case of active sceneswith dependencies, send the msg to he scenary manager so he decides stuff
        //          - for now I only have groups of shutters opening/closing in some sequence
        //          - external gate, after position control i4 and magnetic switches, for situation awareness
        //          - external gate trigger scenery is controlled by the client...still to decide how to handle more than one client on final approach
        //              maybe not n issue, as the cmd if "open", and worst thing scenario, it will be two open cmds...
        //              ...but as the gate closes after x seconds, the timming functions will have to control the initial gate position
        //              when the cmd was issue, as the time to fully open may differ...and that may impact how much time it stays open on final approach 
        //              (and it may happen that the gate is closing or cosed precisely near touch down time)
        //              so we may need to do something more, logic wise, to have more than one client with this feature
        let o_pos = topic.find('-');
        if let Some(dash_pos) = o_pos {
            let id_str = &topic.substring(dash_pos, 12);
            let o_idx = self.desc_to_device.get(*id_str);
            if let Some(idx) = o_idx {
                let device = &self.devices_and_scenes.devices[*idx];
                match device.device_type {
                    DeviceType::Relay => {
                        match data.as_str() {
                            "off" => (), //nop - here for future usage
                            "on" => (),  //nop - here for future usage
                            _ => (),     // unknown msg, nop
                        }
                    }
                    DeviceType::Roller => {
                        let topic_type = &topic.substring(dash_pos + 13, 6);
                        match *topic_type {
                            "events" => {
                                let result = data_from_str::<ShellyPlus2pmRPCEvent>(&data);
                                match result {
                                    Ok(_event) => {}
                                    Err(e) => {
                                        log_error!(build_error(&e));
                                    }
                                }
                            }
                            "status" => {
                                let result = data_from_str::<ShellyPlus2pmStatus>(&data);
                                match result {
                                    Ok(_status) => {}
                                    Err(e) => {
                                        log_error!(build_error(&e));
                                    }
                                }
                            }
                            _ => {} // ignored for now.  It will change if/when we add other device types
                        }
                    }
                    DeviceType::TriggerSwitch => {}
                }
            }
        }
    }
}

pub mod tests {
    #![allow(unused_imports)]
    use crate::{
        db::db_sql_lite::Persist,
        services::electronics::{actuator::ActuatorCommand, devices_svc::DevicesSvc},
    };

    // #[test]
    // fn test_turn_on_ok() {
    //     let db = Persist::new();
    //     let devs = DevicesSvc::new(&db);
    //     assert!(devs.apply_lawn_cmd(0, ActuatorCommand::On));
    // }

    // #[test]
    // fn test_turn_off_ok() {
    //     let db = Persist::new();
    //     let devs = DevicesSvc::new(&db);
    //     assert!(devs.apply_lawn_cmd(0, ActuatorCommand::Off));
    // }

    // #[test]
    // fn test_valve_status_ok() {
    //     let db = Persist::new();
    //     let devs = DevicesSvc::new(&db);
    //     println!("{:?}", devs.relay_status(0));
    // }

    // #[test]
    // fn test_valve_status_nok() {
    //     let db = Persist::new();
    //     let devs = DevicesSvc::new(&db);
    //     println!("{:?}", devs.relay_status(1));
    // }
}
