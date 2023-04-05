use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use criterion::Criterion;
use ctrl_lib::{
    db::{
        db_error::{DBError, SimpleResult},
        db_sql_lite::{Persistance, DB},
    },
    utils::build_abs_file_path,
};
use ctrl_prelude::domain_types::SIM;
use ctrl_prelude::domain_types::WARP;
use num_enum::TryFromPrimitive;
use serde::*;

pub const CONFIG_FILE: &str = "data\\app_config.toml";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DBMaintConfig {
    pub db_maint_counter: u8,        // 1 byte
    pub db_maint_last_run: CtrlTime, // 8 byte        9
    pub db_maint_days: u8,           // 1 byte        10
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CtrlTime(pub u64);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeData {
    pub time_control_interval: u8, //1 byte                1
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StartupData {
    pub start_date_str: String, //24 + 30     26 -> align to 32 + 30 no heap = 62
    pub start_date: CtrlTime,   // 8        32
    // pub simulation: SIM,        //1 byte
    pub warp: WARP,             //1 byte          34
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
/// never, x-retries, date
pub enum ScheduleStop {
    Never = 0,
    Retries = 1,
    Date = 2,
}

impl Default for ScheduleStop {
    fn default() -> Self {
        ScheduleStop::Never
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ScheduleRepeatUnit {
    Seconds = 0,
    Minutes = 1,
    Hours = 2,
    Days = 3,
    Weeks = 4,
}
impl Default for ScheduleRepeatUnit {
    fn default() -> Self {
        ScheduleRepeatUnit::Days
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ScheduleRepeat {
    Never = 0,
    SpecificWeekday = 1,
    Every = 2,
}

impl Default for ScheduleRepeat {
    fn default() -> Self {
        ScheduleRepeat::Never
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Schedule {
    pub start_ts: CtrlTime,                    // 8
    pub repeat_kind: ScheduleRepeat,           // never/specific week days/every                             // 1         9
    pub stop_condition: ScheduleStop,          // "never", x-retries, date                                     // 1         10
    pub stop_retries: u16,                     // 2                                                                                       12
    pub stop_date_ts: CtrlTime,                // 8                                                                                   20
    pub repeat_spec_wd: u8,                    // 1 = 21
    pub repeat_every_qty: u16,                 // 2                                                                                       23
    pub repeat_every_unit: ScheduleRepeatUnit, // "", minutes, hours, days, week, month                            // 1   24
    pub retries_count: u16, // 2                                                                                       26 aligned to 32
}

#[allow(dead_code)]
#[derive(Default, Serialize, Deserialize)]
pub struct InnerConfig {
    current_day: String, // 24                                                          10 Heap
    #[serde(skip)]
    schedule: Schedule, //32                                                    56
    #[serde(skip)]
    settings_file: String, //24+12 = 36                                            80     12 Heap  22
    live_since: String,  // "2020-08-13T05:01:45.690565000Z" //24 + 30 = 54      104    + 30 Heap 52
    #[serde(skip)]
    pub last_save: CtrlTime, // 8                                                    200
    pub last_client_update: CtrlTime, // 8                                                    208
    last_change: CtrlTime, // 8                                                    216
    #[serde(skip)]
    pub restart: bool, // 1
    check_client_interval: u8, // 1
    shutdown: u8,        // 0 #se 1, foi não controlaado                //1
    file_save_interval: u8, // 1                                                    220
    start_up: StartupData, //40                                                    144
    time: TimeData,      //32                                                    176     +10 Heap 62
    pub db_maint: DBMaintConfig, // 16                                                   192
}

#[derive(Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum AppConfigParams {
    //appconfig
    CurrentDay = 0,
    LiveSince,
    LastChange,
    LastClientUpdate,
    CheckClientInterval,
    ShutDown,
    FileSaveInterval,
    //start up
    StartDateStr,
    StartDate,
    Simulation,
    Warp,
    //time data
    TimeControlInterval,
    //db maint
    DbMaintCounter,
    DbMaintLastRun,
    DbMaintDays,
}

//NOTE - isto está desfasado com a realidade.  o bool column desapareceu e o string agora é 2
#[allow(dead_code)]
pub const FLOAT_COLUMN: usize = 0;
pub const INT_COLUMN: usize = 1;
pub const BOOL_COLUMN: usize = 2;
pub const STRING_COLUMN: usize = 3;

#[repr(u8)]
pub enum Module {
    AppConfig = 0,
}
fn config_new() -> InnerConfig {
    let sfile: String = build_abs_file_path(CONFIG_FILE);
    // let metadata = fs::metadata(&sfile).unwrap_or_else(|_| panic!("Problem with the configuration file: {}", sfile));

    let file_result = File::open(&sfile);

    let mut inner_context: InnerConfig = if let Ok(mut file) = file_result {
        let mut buffer = Vec::with_capacity(640);
        file.read_to_end(&mut buffer).unwrap_or_else(|e| panic!("Problem with the configuration file: {}", e));
        toml::from_slice::<InnerConfig>(&buffer).unwrap_or_else(|e| panic!("Problem deserializing file: {}", e))
    } else {
        panic!("Problem with general context file:1n {:?}", file_result);
    };
    inner_context.settings_file = sfile;
    inner_context
}

pub trait ModelConfig<'a>: DB {
    const GET_MODULE_CONFIG: &'a str = "SELECT float,int,bool,string FROM mods_data where module=? order by param;";
    const UPDATE_MODULE_CONFIG: &'a str = "update mods_data set float=?1,int=?2,bool=?3,string=?4 where module=?5 and param=?6;";

    fn get_config(&mut self, cfg_module: u8) -> Result<InnerConfig, DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_MODULE_CONFIG).unwrap();

        let mut _res = stmt.raw_bind_parameter(1, cfg_module);

        let mut cfg = InnerConfig::default();
        let mut rows = stmt.raw_query();
        let mut idx :u8= 0;
        let mut app_config_param : AppConfigParams;// aqui não interessa o valor.  é só paa alocar o espaço para a variavel
        while let Some(row) = rows.next()? {
            app_config_param = unsafe { AppConfigParams::try_from_primitive(idx).unwrap()};
            match app_config_param {
                AppConfigParams::CurrentDay => cfg.current_day = row.get_unwrap(STRING_COLUMN),
                AppConfigParams::LiveSince => cfg.live_since = row.get_unwrap(STRING_COLUMN),
                AppConfigParams::LastChange => cfg.last_change = CtrlTime(row.get_unwrap(INT_COLUMN)),
                AppConfigParams::LastClientUpdate => cfg.last_client_update = CtrlTime(row.get_unwrap(INT_COLUMN)),
                AppConfigParams::CheckClientInterval => cfg.check_client_interval = row.get_unwrap(INT_COLUMN),
                AppConfigParams::ShutDown => cfg.shutdown = row.get_unwrap(BOOL_COLUMN),
                AppConfigParams::FileSaveInterval => cfg.file_save_interval = row.get_unwrap(INT_COLUMN),
                //start up
                AppConfigParams::StartDateStr => cfg.start_up.start_date_str = row.get_unwrap(STRING_COLUMN),
                AppConfigParams::StartDate => cfg.start_up.start_date = CtrlTime(row.get_unwrap(INT_COLUMN)),
                AppConfigParams::Simulation => cfg.start_up.simulation = row.get_unwrap(INT_COLUMN),
                AppConfigParams::Warp => cfg.start_up.warp = row.get_unwrap(INT_COLUMN),
                //time data
                AppConfigParams::TimeControlInterval => cfg.time.time_control_interval = row.get_unwrap(INT_COLUMN),
                //db maint
                AppConfigParams::DbMaintCounter => cfg.db_maint.db_maint_counter = row.get_unwrap(INT_COLUMN),
                AppConfigParams::DbMaintLastRun => cfg.db_maint.db_maint_last_run = CtrlTime(row.get_unwrap(INT_COLUMN)),
                AppConfigParams::DbMaintDays => cfg.db_maint.db_maint_days = row.get_unwrap(INT_COLUMN),
            }
            // println!("passou campo: {}",idx);
            idx += 1;
        }
        Ok(cfg)
    }

    // update mods_data set float=?1,int=?2,bool=?3,string=?4 where module=?5 and param=?6;";
    // //ter atenção aos caller por causa dos defaults op = SYNC_OP.UPD,
    fn save_config(&mut self, cfg: &InnerConfig) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG).unwrap();

        _ = stmt.raw_bind_parameter(5, Module::AppConfig as u8);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::CurrentDay as u8);
        _ = stmt.raw_bind_parameter(1, 0. as f64);
        _ = stmt.raw_bind_parameter(2, 0 as u64);
        _ = stmt.raw_bind_parameter(3, 0 as u8);
        _ = stmt.raw_bind_parameter(4, &cfg.current_day);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::LiveSince as u8);
        _ = stmt.raw_bind_parameter(4, &cfg.live_since);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::LastChange as u8);
        _ = stmt.raw_bind_parameter(2, cfg.last_change.0);
        _ = stmt.raw_bind_parameter(4, "");
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::LastClientUpdate as u8);
        _ = stmt.raw_bind_parameter(2, cfg.last_client_update.0);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::CheckClientInterval as u8);
        _ = stmt.raw_bind_parameter(2, cfg.check_client_interval);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::ShutDown as u8);
        _ = stmt.raw_bind_parameter(2, 0 as u64);
        _ = stmt.raw_bind_parameter(3, cfg.shutdown);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::FileSaveInterval as u8);
        _ = stmt.raw_bind_parameter(2, cfg.file_save_interval);
        _ = stmt.raw_bind_parameter(3, 0 as u64);
        _ = self.exec_prep(&mut stmt);

        //start up
        _ = stmt.raw_bind_parameter(6, AppConfigParams::StartDateStr as u8);
        _ = stmt.raw_bind_parameter(2, 0 as usize);
        _ = stmt.raw_bind_parameter(3, 0 as u64);
        _ = stmt.raw_bind_parameter(4, &cfg.start_up.start_date_str);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::StartDate as u8);
        _ = stmt.raw_bind_parameter(2, cfg.start_up.start_date.0);
        _ = stmt.raw_bind_parameter(4, "");
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::Simulation as u8);
        _ = stmt.raw_bind_parameter(2, cfg.start_up.simulation);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::Warp as u8);
        _ = stmt.raw_bind_parameter(2, cfg.start_up.warp);
        _ = self.exec_prep(&mut stmt);

        //time data
        _ = stmt.raw_bind_parameter(6, AppConfigParams::TimeControlInterval as u8);
        _ = stmt.raw_bind_parameter(2, cfg.time.time_control_interval);
        _ = self.exec_prep(&mut stmt);

        //db maint
        _ = stmt.raw_bind_parameter(6, AppConfigParams::DbMaintCounter as u8);
        _ = stmt.raw_bind_parameter(2, cfg.db_maint.db_maint_counter);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::DbMaintLastRun as u8);
        _ = stmt.raw_bind_parameter(2, cfg.db_maint.db_maint_last_run.0);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(6, AppConfigParams::DbMaintDays as u8);
        _ = stmt.raw_bind_parameter(2, cfg.db_maint.db_maint_days);
        self.exec_prep(&mut stmt)

    }
}

impl<'a> ModelConfig<'a> for Persistance {}

fn read_config_from_file() {
    let _cfg = config_new();
}

fn save_config_to_file(cfg: &InnerConfig) {
    let buffer = toml::to_string_pretty(&cfg).expect("erro a serializar o contexto");
    let mut f = OpenOptions::new().write(true).create(true).truncate(true).open(&cfg.settings_file).expect("Erro a abrir o ficheiro contexto");
    f.write_all(buffer.as_bytes()).expect("erro a gravar o ficheiro")
}

fn read_config_from_db(db: &mut Persistance) {
    let _cfg = db.get_config(Module::AppConfig as u8);
}

fn save_config_to_db(db: &mut Persistance, cfg: &InnerConfig) {
    let _res = db.save_config(cfg);
}

pub fn bench_config_vs_db(d: &mut Criterion) {
    let cfg = config_new();

    let mut db =Persistance::new();

    let mut c = d.benchmark_group("bench_config_vs_db");
    c.bench_function("read_config_from_file", |b| b.iter(|| (read_config_from_file())));
    c.bench_function("save_config_to_file", |b| b.iter(|| (save_config_to_file(&cfg))));
    c.bench_function("read_config_from_db", |b| b.iter(|| (read_config_from_db(&mut db))));
    c.bench_function("save_config_to_db", |b| b.iter(|| (save_config_to_db(&mut db, &cfg))));

    c.finish();
}
