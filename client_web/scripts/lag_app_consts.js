"use strict";

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
    MANUAL   : "Manual",
    STANDARD : "Standard",
    WIZARD   : "Wizard"
};

//# never, x-retries, date
const SCHEDULE_STOP = {
    NEVER   : "Never",
    RETRIES : "Retries",
    DATE    : "Date"
};

//# never/hourly/daily/weekday/specific week days/weekly/monthly/every
const SCHEDULE_REPEAT = {
    NEVER               : "Never",
    SPECIFIC_WEEKDAY    : "SpecificWeekday",
    EVERY               : "Every"
};

const PT_UNITS = {
    "Seconds" : "segundo",  //A.  + ____z unidades  + B.
    "Minutes" : "minuto",
    "Hours"   : "hora",
    "Days"    : "dia",
    "Weeks"   : "semana"
};

//# minutes, hours, days, week
const SCHEDULE_REPEAT_UNIT = {
    SECONDS : "Seconds",
    MINUTES : "Minutes",
    HOURS   : "Hours",
    DAYS    : "Days",
    WEEKS   : "Weeks"
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

const WEEK_DAYS_ABR = {
    SUNDAY      : "Dom.",
    MONDAY      : "Seg.",
    TUESDAY     : "Ter.",
    WEDNESDAY   : "Qua.",
    THURSDAY    : "Qui.",
    FRIDAY      : "Sex.",
    SATURDAY    : "Sab."
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

const DAYS_ABREV_IDX = {
    7 : "Dom.",
    6 : "Seg.",
    5 : "Ter.",
    4 : "Qua.",
    3 : "Qui.",
    2 : "Sex.",
    2 : "Sab."
};

const repeat_index = {
    "Never"            : 0,
    "SpecificWeekday"  : 1,
    "Every"            : 2
};

const repeat_index_desc = {
    "Never"            : "Nunca",
    "SpecificWeekday"  : "Dias específicos",
    "Every"            : "A cada..."
};

const repeat_from_index = {
    0: "Never" ,
    1: "SpecificWeekday",
    2: "Every"
};

const repeat_unit_index = {
    "Seconds" : 0,
    "Minutes" : 1,
    "Hours"   : 2,
    "Days"    : 3,
    "Weeks"   : 4
};

const repeat_unit_from_index = {
    0: "Seconds",
    1: "Minutes",
    2: "Hours" ,
    3: "Days",
    4: "Weeks"
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
    "Never"     :0,
    "Retries"   :1,
    "Date"      :2
};

const stop_index_description = {
    "Never"     :"Nunca",
    "Retries"   :"Ao fim de ..",
    "Date"      :"Até à data de .."
};

const stop_from_index = {
    0: "Never",
    1: "Retries",
    2: "Date"
};

const WATERING_STATUS = {
    WAITING             : "Waiting",
    RUNNING             : "Running",
    SUSPENDED           : "Suspended",
    NOT_EXECUTED        : "NotExecuted",
    TERMINATED          : "Terminated",
    ERROR               : "Error",
    SUSPENDED_TIMEOUT   : "SuspendedTimeout"
};
const WATERING_STATUS_INDEX = {
    WAITING             : 0,
    RUNNING             : 1,
    SUSPENDED           : 2,
    NOT_EXECUTED        : 3,
    TERMINATED          : 4,  
    ERROR               : 5,
    SUSPENDED_TIMEOUT   : 6,
}
const watering_status_from_index = {
    0   : "Waiting",
    1   : "Running",
    2   : "Suspended",
    3   : "NotExecuted",
    4   : "Terminated",
    5   : "Error",
    6   : "SuspendedTimeout"    
}

const CYCLE_TYPE = {
    WIZARD:  "Wizard",
    COMPENSATION: "Compensation",
    DIRECT: "Direct",
    STANDARD: "Standard",
}
const CYCLE_TYPE_INDEX = {
    WIZARD : 0,
    COMPENSATION : 1,
    DIRECT : 2,
    STANDARD : 3,
}
const cycle_type_from_index = {
    0: "Wizard",
    1: "Compensation",
    2: "Direct",
    3: "Standard",
}

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

const CONNECTION_STATUS = {
    NOTHING: 0,
    MQTT: 1,
    SERVER: 2,
    ALL: 3
}

const LOG_ENTRY_TYPE = {
    ALERT: "ALERT",
    LOG: "CLIENT",
    ERROR: "ERROR",
    SERVER: "SERVER"
};


const ALERT_TYPE = {
    NoAlert: 0,
    WIND: 2,
    RAIN : 4,
    PresenceDetection : 8,
    WindowOrDoorOpen : 16,
}

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
Notes to add more config data that need to be considered in the client UI
- add to CONFIG_ATTRIBUTES
- ad to CONFIG_ATTRIBUTES_UPDATABLE if thats the case
- add to html
- add to the respective control in setup.js
    - if it is editable, add to setup_dirty_mon array 
    - add control to disable_screen
    - add control to enable_screen
    - add to setelementValue in the render
    - add control to get_screen_data
    - add the model to _assign
    - if it is data to exclude, review assign calls, in the field to exclude logic
- add data to model config in rega_model.js
    - add to parameters of function ins
    - add to object of insFO
    - add to clone , o.set

*/
const CONFIG_ATTRIBUTES = ["mode",
                           "in_error",
                           "in_alert",
                           "date",
                           "max_sector_time",
                           "pump_recycle_time",
                           "schedule_def",
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

const CYCLE_ATTRIBUTES = ["start",
                          "end",
                          "cycle_id",
                          "run_id",
                          "status",
                          "run_start",
                          "repeat_kind",
                          "stop_condition",
                          "stop_retries",
                          "stop_date_ts",
                          "repeat_spec_wd",
                          "repeat_every_qty",
                          "repeat_every_unit",
                          "retries_count",
                          "name", 
                          "last_change",                           
                          "last_run",
                          "sim",
                          "op",
                          "cycle_type",
                          "sunrise_flg",
                          "sunset_flg"
                          ];

const SECTOR_ATTRIBUTES = ["desc",
                           "name",
                           "last_watered_in",
                           "last_change",
                           "deficit",
                           "percolation",
                           "debit",
                           "max_duration",
                           "stress_score", 
                           "stress_perc",
                           "id",
                           "enabled",
                           "op",
                           "start",
                           "end",
                           "minutes_to_water",
                           "status",
                        ];

const WATER_MACHINE_STATUS = {
     "Starting"             : "A arrancar",
     "NoScheduleDef"        : "Sem programa definido",
     "EstablishMode"        : "A avaliar condições operação",
     "ManWait"              : "Parada",
     "WzrWait"              : "A monitorizar eventos",
     "StdWait"              : "A monitorizar eventos",
     "ManWtrCycle"          : "Exec. ciclo manual",
     "StdWtrCycle"          : "Exec. ciclo standard",
     "WzrWtrCycle"          : "Exec. ciclo inteligente",
     "ManWtrSector"         : "Exec. ciclo manual: setor ativo",
     "StdWtrSector"         : "Exec. ciclo standard: setor ativo",
     "WzrWtrSector"         : "Exec. ciclo inteligente: setor ativo",
     "ManWtrSectorDirect"   : "Manual: Setor ativo",
     "SuspendedWizard"      : "Suspenso - alerta metereológico",
     "Error"                : "Máquina em erro",
     "Shutdown"             : "Servidor desligado"
};


const CYCLE_NAME = {
    DIRECT: "Direct",
    WIZARD: "Wizard-auto",
    COMPENSATION: "compensation"
};

const PROTOCOL = "https://"
const HOST = "lagarto-lx.privatedns.org";

const APP_CONTROLLER = "controller";

const CMD_ID = "id";
const CMD_SHUTDOWN = "shutdown";
const CMD_IS_ALIVE = "is_alive";
const CMD_HISTORY = "get_water_history";

const GET_ACTUATORS_AND_SCENES = "get_actuators_and_scenes";
const SET_ACTUATOR_OR_SCENE = "set_actuator_or_scene"

const APPLICATION = APP_CONTROLLER;
