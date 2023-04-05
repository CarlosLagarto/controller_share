"use strict";

const CLIENT_ID = "CLIENT_" + moment().unix();  
const WAIT_MQTT_TIMEOUT_INTERVAL = 10;
const QoS = 1;
const MAX_CACHE_SIZE = 100;

const MQTT_ERROR = {
    1: "Connection refused - incorrect protocol version",
    2: "Connection refused - invalid client identifier",
    3: "Connection refused - web_server unavailable",
    4: "Connection refused - bad username or password",
    5: "Connection refused - not authorised"
};


const WATER_MACHINE_MODE = {
    MANUAL   : "manual",
    STANDARD : "standard",
    WIZARD   : "wizard"
};

//# never, x-retries, date
const SCHEDULE_STOP = {
    NEVER   : "never",
    RETRIES : "retries",
    DATE    : "date"
};

//# never/hourly/daily/weekday/specific week days/weekly/monthly/every
const SCHEDULE_REPEAT = {
    NEVER               : "never",
    HOURLY              : "hourly",
    DAILY               : "DAILY",
    SPECIFIC_WEEKDAY    : "specific_weekday",
    WEEKLY              : "weekly",
    EVERY               : "every"
};

const PT_UNITS = {
    "seconds" : "segundo",  //A.  + ____z unidades  + B.
    "minutes" : "minuto",
    "hours"   : "hora",
    "days"    : "dia",
    "weeks"   : "semana"
};

//# minutes, hours, days, week
const SCHEDULE_REPEAT_UNIT = {
    SECONDS : "seconds",
    MINUTES : "minutes",
    HOURS   : "hours",
    DAYS    : "days",
    WEEKS   : "weeks"
};

//# Su|Mo|Tu|We|Th|Fr|St
const WEEK_DAYS = {
    SUNDAY      : "Sunday",
    MONDAY      : "Monday",
    TUESDAY     : "Tuesday",
    WEDNESDAY   : "Wednesday",
    THURSDAY    : "Thursday",
    FRIDAY      : "Friday",
    SATURDAY    : "Saturday"
};

const DAYS_ABREV = {
    "Sunday"    : "Dom.",
    "Monday"    : "Seg.",
    "Tuesday"   : "Ter.",
    "Wednesday" : "Qua.",
    "Thursday"  : "Qui.",
    "Friday"    : "Sex.",
    "Saturday"  : "Sab."
};

const repeat_index = {
    "never"             : 0,
    "hourly"            : 1,
    "DAILY"             : 2,
    "specific_weekday"  : 3,
    "weekly"            : 4,
    "every"             : 5
};

const repeat_index_desc = {
    "never"             : "Nunca",
    "hourly"            : "Hora a hora",
    "DAILY"             : "Diariamente",
    "specific_weekday"  : "Dias específicos",
    "weekly"            : "Semanalmente",
    "every"             : "A cada..."
};

const repeat_from_index = {
    0: "never" ,
    1: "hourly",
    2: "DAILY",
    3: "specific_weekday",
    4: "weekly",
    5: "every"
};

const repeat_unit_index = {
    "seconds" : 0,
    "minutes" : 1,
    "hours"   : 2,
    "days"    : 3,
    "weeks"   : 4
};

const repeat_unit_from_index = {
    0: "seconds",
    1: "minutes",
    2: "hours" ,
    3: "days",
    4: "weeks"
};

const days_index = {
    "Sunday"    : 0,
    "Monday"    : 1,
    "Tuesday"   : 2,
    "Wednesday" : 3,
    "Thursday"  : 4,
    "Friday"    : 5,
    "Saturday"  : 6
};

const stop_index = {
    "never"     :0,
    "retries"   :1,
    "date"      :2
};

const stop_index_description = {
    "never"     :"Nunca",
    "retries"   :"Ao fim de ..",
    "date"      :"Até à data de .."
};

const stop_from_index = {
    0: "never",
    1: "retries",
    2: "date"
};

const WATERING_STATUS = {
    WAITING         : "waiting",
    RUNNING         : "running",
    NOT_EXECUTED    : "not_executed",
    TERMINATED      : "terminated"
};

const MODAL_MODE = {
    NEW : 0,
    EDIT: 1,
    VIEW: 2
};

//REVIEW: - algures no tempo vamos também ter aqui os sensores.
const BD_LAST_SYNC      = "BD_LAST_SYNC";
const BD_SENSORS_TBL    = "BD_SENSORS_TBL";
const BD_CYCLES_TBL     = "BD_CYCLES_TBL";
const BD_SECTORS_TBL    = "BD_SECTORS_TBL";

const BD_UUID_TBL = "BD_UUID";
const BD_MQTT_TBL = "BD_MQTT";

const BD_CLIENT_LOG     = "BD_CLIENT_LOG";
const BD_SERVER_LOG     = "BD_SERVER_LOG";
const BD_CLIENT_ALERT   = "BD_CLIENT_ALERT";
const BD_CLIENT_ERROR   = "BD_CLIENT_ERROR";

const BD_CONFIG_ROW = "BD_CONFIG_ROW";

const SYNC_TYPE = {
    FULL      : "F",
    PARTIAL   : "P",
    UNDEFINED : "U"
};

const OP = {
    I:"I",
    D:"D",
    U:"U"
};

const CONNECTION = {
    ONLINE: "ONLINE",
    OFFLINE: "OFFLINE"
};

const LOG_ENTRY_TYPE = {
    ALERT: "ALERT",
    LOG: "CLIENT",
    ERROR: "ERROR",
    SERVER: "SERVER"
};

const DATE_LONG_FORMAT = "YYYY/MM/DD HH:mm";
const TIME_24H_FORMAT = "HH:mm";
const ISO8601_DATE_FORMAT = "YYYY-MM-DDTHH:MM";
const DATE_LOG_FORMAT = "YYYY/MM/DD HH:mm:ss.SSS";

const unit_mm = " mm";
const unit_percentage = " %";
const unit_direction = " º";
const unit_velocity = " km/h";
const unit_celsius = " ºC";

/*
Notas para adicionar mais um campo de configuração quando tem contraparte aqui no interface cliente
- adicionar no CONFIG_ATTRIBUTES
- adicionar no CONFIG_ATTRIBUTES_UPDATABLE se for o caso
- adicionar o campo no html
- adicionar o controlo respetivo no setup.js
    - se for editavel, adicionar ao array setup_dirty_mon
    - adicionar o controlo no disable_screen
    - adicionar o controlo no enable_screen
    - adicionar o setelementValue no render
    - adicionar o controlo no get_screen_data
    - adicionar o model no _assign
    - se for um campo para excluir, rever as chamadas ao assign, no campo para exclusão 
- adicionar o campo no model config no rega_model.js
    - adicionar nos parametros da função ins
    - adicionar no objeto do insFO
    - adicionar no clone , o.set

*/
const CONFIG_ATTRIBUTES = ["mode",
                           "in_error",
                           "in_alert",
                           "date",
                           "max_sector_time",
                           "pump_recycle_time",
                           "schedule_def",
                           "strategy_water_mm_per_week",
                           "strategy_water_days_per_week",
                           "strategy_root_length",
                           "rain_alert_threshold",
                           "wind_alert_threshold",
                           "watering_suspend_timeout",
                           "decrease_alert_level_after",
                           "db_maint_days",
                           "db_maint_counter",
                           "rain_week_acc_days",
                           "rain_week_acc_counter",
                           "live_since",
                           "current_state",
                           "simulation",
                           "weekly_acc_precipitation",
                           "last_change",
                           "op",
                           "stress_control_interval"];

const CONFIG_ATTRIBUTES_UPDATABLE = ["max_sector_time",
                                     "pump_recycle_time",
                                     "strategy_water_mm_per_week",
                                     "strategy_water_days_per_week",
                                     "strategy_root_length",
                                     "rain_alert_threshold",
                                     "wind_alert_threshold",
                                     "watering_suspend_timeout",
                                     "decrease_alert_level_after",
                                     "db_maint_days",
                                     "db_maint_counter",
                                     "rain_week_acc_days",
                                     "rain_week_acc_counter",
                                     "weekly_acc_precipitation",
                                     "last_change",
                                     "stress_control_interval"];

const CYCLE_ATTRIBUTES = ["nome", 
                          "id",
                          "start_ts", 
                          "last_run_ts",
                          "status",
                          "repeat_kind",
                          "repeat_spec_wd",
                          "repeat_every_qty",
                          "repeat_every_unit",
                          "stop_condition",
                          "stop_retries",
                          "stop_date_ts",
                          "retries_count",
                          "start_sunrise_index",
                          "last_change",
                          "id",
                          "op"];

const SECTOR_ATTRIBUTES = ["id",
                           "description",
                           "week_acc",
                           "precipitation",
                           "debit",
                           "last_watered_in",
                           "enabled",
                           "max_duration",
                           "start_utc_ts",
                           "end_utc_ts",
                           "minutes_to_water",
                           "status",
                           "short_description",
                           "last_change",
                           "op",
                           "stress_score", 
                           "stress_perc"];

const WATER_MACHINE_STATUS = {
     "starting"                         : "A arrancar",
     "no_schedule_defined"              : "Sem programa definido",
     "establish_mode"                   : "A avaliar condições operação",
     "manual_wait"                      : "Parada",
     "wizard_wait"                      : "A monitorizar eventos",
     "standard_wait"                    : "A monitorizar eventos",
     "manual_watering_cycle"            : "Exec. ciclo manual",
     "standard_watering_cycle"          : "Exec. ciclo standard",
     "wizard_watering_cycle"            : "Exec. ciclo inteligente",
     "manual_watering_sector"           : "Exec. ciclo manual: setor ativo",
     "standard_watering_sector"         : "Exec. ciclo standard: setor ativo",
     "wizard_watering_sector"           : "Exec. ciclo inteligente: setor ativo",
     "manual_watering_sector_direct"    : "Manual: Setor ativo",
     "suspended_wizard"                 : "Suspenso - alerta metereológico",
     "error"                            : "Máquina em erro",
     "shut_down"                        : "Servidor desligado"
};

const WIZARD_NAME = "Ciclo inteligente";