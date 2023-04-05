use string_concat::*;

use crate::globals::MANUAL_DIRECT_SUFIX;

/// Evento Tipo: "type" "description"
#[inline]
pub fn info_broker_gen(msg_type: &str, msg_desc: &str) -> String {
    string_concat!("Evento Tipo: ", msg_type, " ", msg_desc)
}

pub static INFO_BRKR_THREAD_START: &str = "Arrancou a thread do broker de mensagens.";
pub static INFO_BRKR_THREAD_STOP: &str = "Terminou a thread do broker de mensagens.";
pub static INFO_BRKR_TERMINATE_MSG: &str = "Mensagem término do broker de mensagens.";

pub static INFO_DEV_THREAD_STOP: &str = "Terminou a thread dos devices.";
pub static INFO_DEV_THREAD_START: &str = "Arrancou a thread dos devices.";

pub static INFO_WTHR_THREAD_START: &str = "Arrancou a thread da metereologia.";
pub static INFO_WTHR_THREAD_STOP: &str = "Terminou a thread da metereologia.";
// pub static INFO_WTHR_TERMINATE_MSG: &str = "Mensagem término do broker de mensagens.";

#[inline]
/// "Evento tipo: {} sem subscriptor. {}"
#[cfg(debug_assertions)]
pub fn dbg_brkr_no_sbs(msg_type: &str, msg_desc: &str) -> String {
    format!("Evento tipo: {msg_type} sem subscriptor. {msg_desc}")
}
#[inline]
/// O broker tem {} subscritores"
#[cfg(debug_assertions)]
pub fn dbg_brkr_nr_of_subs(nr: usize) -> String {
    format!("O broker tem {nr} subscritores")
}

///"O subscritor {} já estava registado"
#[inline]
#[cfg(debug_assertions)]
pub fn dbg_brkr_dup_subs(subscriber_name: &str) -> String {
    format!("O subscritor {subscriber_name} já estava registado")
}

///"O evento tipo {} tem {} subscritores"
#[inline]
#[cfg(debug_assertions)]
pub fn dbg_brkr_nr_of_msgs_subs(msg_type: &str, nr_of_subscribers: usize) -> String {
    format!("O evento tipo {msg_type} tem {nr_of_subscribers} subscritores")
}

/// "O canal tem {} mensagens para enviar"
#[inline]
#[cfg(debug_assertions)]
pub fn dbg_brkr_nr_of_msg_in(nr_of_msgs: usize) -> String {
    format!("O canal tem {nr_of_msgs} mensagens para enviar")
}

/// Erro a enviar a mensagem no msg broker no subscritor "subscriber" \n<br>
/// error detail
#[inline]
pub fn err_snd_brkr_msg(subscriber: &str, error: &str) -> String {
    string_concat!("Erro a enviar a mensagem no msg broker no subscritor ", subscriber, ": \n", error)
}

pub static BRKR_STR_EXT_MSG: &str = "External message";

pub static INFO_PROGRAM_STARTED: &str = "O programa arrancou.";
pub static INFO_STARTING_MAIN_CHECK: &str = "Controlador a executar";
pub static INFO_SHUTDOWN_COMPLETED: &str = "O Programa tem o ShutDown completo.";

/// "A iniciar o controlador.<br>
/// Parâmetros:           <br>
///     Data de início: {}<br>
///     Warp time     : {}"<br>
#[inline]
pub fn info_starting_controller(date: &str, warp: u8) -> String {
    format!(
        "\nA iniciar o controlador.\n\
                Parâmetros:           \n\
                \x20  Data de início: {date}\n\
                \x20  Warp time     : {warp}"
    )
}

pub static INFO_LAST_WORDS: &str = "finzinho\n\n\n";

/// "Time Tick no main control loop: {}"
#[inline]
pub fn dbg_time_tick(date: &str) -> String {
    string_concat!("Time Tick no main control loop: ", date)
}

/// "Nr de threads ativas: {}"
#[inline]
#[cfg(debug_assertions)]
pub fn dbf_nr_of_active_threads(nr: usize) -> String {
    format!("Nr de threads ativas: {nr}")
}

/// "Fim do join do serviço: {}"
#[inline]
pub fn dbg_join_end(thread_name: &str) -> String {
    string_concat!("Fim do join do serviço: ", thread_name)
}

pub static DBG_CTRL_HNDLR_INSTALL: &str = "Control handler instalado.";

pub static DESC_SHUTDOWN_CMD_FRM_CLIENT: &str = "Shutdown a partir do cliente.";
pub static DESC_SHUTDOWN_CMD_FRM_CONSOLE: &str = "Shutdown a partir da consola.";
pub static DESC_SHUTDOWN_CMD_FROM_WEB: &str = "ShutDown a partir da web.";

pub static WARN_LAST_SHUTDOWN_UNCONTROLLED: &str = "O ultimo shutdown não foi controlado!";
pub static WARN_TRYING_TO_RECOVER: &str = "A tentar recuperar bd...";

/// "Tipo de mensagem não subscrita - msg type: {}"
#[inline]
pub fn warn_unsubs_msg_type(msg_type: &str) -> String {
    string_concat!("Tipo de mensagem não subscrita - msg type: ", msg_type)
}
pub static ERR_CTRL_HNDLR_INSTALL: &str = "Não se conseguiu instalar o control handler.";

pub static INFO_PHISICAL_ADAPTER_INI: &str = "Inicialização do sistema fisico - init watering system.";

/// "Error building file path for file: {}"
#[inline]
pub fn err_wrong_file_path(file: &str) -> String {
    string_concat!("Error building file path for file: ", file)
}

/// "Erro na gravação dos parametros na bd.\n{}"
#[inline]
pub fn err_saving_app_cfg(error: &str) -> String {
    string_concat!("Erro na gravação dos parametros na bd.\n", error)
}

/// "Problema com o ficheiro de configuração:\n {}"
#[inline]
pub fn err_read_cfg_file(error: &str) -> String {
    string_concat!("Problema com o ficheiro de configuração:\n", error)
}

/// "Problema com a deserialização do ficheiro:\n{}"
#[inline]
pub fn err_deser_cfg_file(error: &str) -> String {
    string_concat!("Problema com a deserialização do ficheiro:\n", error)
}

pub static INFO_WTR_STATE_INFO_1P: &str = "A máquina de rega está no estado: ";

/// "Programa recebeu o comando para terminar.  Máquina de rega está no estado: {}"
#[inline]
pub fn info_prgrm_starting_shutdown(state: &str) -> String {
    string_concat!("Programa recebeu o comando para terminar.  Máquina de rega está no estado: ", state)
}
pub static INFO_STARTING_STATE_MACHINE: &str = "Arranque da máquina de rega";

///"Máquina rega recebeu o comando: {}"
#[inline]
pub fn info_wtr_eng_cmd_rcvd(cmd: &str) -> String {
    string_concat!("Máquina rega recebeu o comando: ", cmd)
}
#[inline]
pub fn dbg_wtr_eng_curr_state(state: &str) -> String {
    string_concat!("Estamos no 'run' do estado: ", state)
}

pub static WARN_NEW_DAY_PROCESS_DELAY: &str = "A thread de novo dia levou mais que 180 segundos a executar.";
pub static WARN_BD_MAINT_PROCESS_DELAY: &str = "A thread de mnt da BD levou mais que 180 segundos a executar.";

pub static DESC_DB_MNT_START_LARGE_MNT: &str = "Arranque manutenção periódica da BD.";
pub static DESC_DB_MNT_START_DAILY_MNT: &str = "Arranque manutenção diária da BD.";
pub static DESC_DB_MNT_END_LARGE_MNT: &str = "Fim da manutenção periódica da BD.";
pub static DESC_DB_MNT_END_DAILY_MNT: &str = "Fim da manutenção diária da BD.";

pub fn err_db_mnt_script(err: &str) -> String {
    string_concat!("Erro no script de manutenção no dia ", err)
}

pub static DESC_ERROR_TO_CLIENT_DESC: &str = "Erro para o cliente.";
pub static DESC_RESTART_CMD_FROM_CLIENT: &str = "Restart a partir do cliente";

pub static DESC_MQTT_TERMINATE_MSG: &str = "Paragem do MQTT.";
pub static DESC_MQTT_HB_MSG_SRVR_CLNT: &str = "Heart Beat Servidor -> Cliente.";
pub static DESC_MQTT_EXT_MSG_RCVD: &str = "Mensagem externa recebida.";

/// "Alerta Metereologia: {}"
#[inline]
pub fn info_whtr_alert_rcvd(alert_type: &str) -> String {
    string_concat!("Alerta Metereologia: ", alert_type)
}
pub static DESC_INFO_WEATHER: &str = "Informação metereológica disponivel.";

/// "Ciclo {} criado em: {} Próxima execução: {}"
#[inline]
pub fn desc_cycle(name: &str, last_start: &str, next_start: &str) -> String {
    format!("Ciclo {name} criado em: {last_start} Próxima execução: {next_start}")
}

/// "Ciclo - forçado manualmente setor: {}"
/// <br>
/// A string "forçado manualmente" é utilizada no programa pelo que se se mudar aqui, mudar nas global consts.
#[inline]
pub fn desc_forced_sector_cycle(sector_name: &str) -> String {
    string_concat!("Ciclo - ", MANUAL_DIRECT_SUFIX, ": ", sector_name)
}

pub static WARN_CLI_DUPLICATED_CYCLE: &str = "O cliente está a duplicar um ciclo.  Ignorado.";
pub static WARN_CLI_DEL_NOT_EXISTING_CYCLE: &str = "O cliente quer apagar um ciclo que não existe.";

pub static ERR_GETTING_METEO_HISTORY: &str = "Erro a obter o histórico da metereologia.";
pub static ERR_CLI_CYCLE_INSERT: &str = "Erro na criação de novo ciclo pelo cliente.";
pub static ERR_CLI_CYCLE_DELETE: &str = "Erro a eliminar um ciclo pelo cliente.";
pub static ERR_CLI_UPD_EXISTING_CYCLE: &str = "Erro a atualizar o ciclo enviado pelo cliente";

pub static INFO_MQTT_CONNECTED_1P: &str = "MQTT Connected. Código: ";
pub static INFO_MQTT_DISCONNECTED: &str = "MQTT desconectado no shutdown.";
pub static INFO_MQTT_STARTING: &str = "A arrancar o mqtt";
pub static INFO_MQTT_ENDING: &str = "A terminar o serviço mqtt";

pub static DBG_MQTT_MSG_PUBLISHED: &str = "MQTT - mensagem aceite pelo broker";
pub static DBG_MQTT_RETRYING_CONNECT: &str = "Estamos a tentar religar ao broker.";

pub static WARN_MQTT_DISCONNECTED: &str = "MQTT desconectado - NOK: desconexão inesperada";
/// "Mensagem do cliente: {} com o tópico: {} recebida duas vezes.  Ignorada!"
#[inline]
pub fn warn_mqtt_dup_msg(sender_id: &str, msg_topic: &str) -> String {
    format!("Mensagem do cliente: {sender_id} com o tópico: {msg_topic} recebida duas vezes.  Ignorada!")
}

/// "MQTT - broker recusou ligação com erro código: {}, descrição: {}"
#[inline]
pub fn err_mqtt_refused_connection(error_code: &str, desc: &str) -> String {
    format!("MQTT - broker recusou ligação com erro código: {error_code}, descrição: {desc}")
}
/// "Erro a lançar o cliente interno do MQTT: {}"
#[inline]
pub fn err_mqtt_int_clnt_creation(error: &str) -> String {
    string_concat!("Erro a lançar o cliente interno do MQTT: ", error)
}
/// "Erro no envio da mensagem para o MQTT.\n{}"
#[inline]
pub fn err_snd_mqtt_msg(error: &str) -> String {
    string_concat!("Erro no envio da mensagem para o MQTT.\n", error)
}
/// "Mensagem MQTT desconhecida. Erro: {}\nTópico: {}\nContéudo: {}"
#[inline]
pub fn err_unknown_mqtt_msg(error: &str, topic: &str, msg_content: &str) -> String {
    format!("Mensagem MQTT desconhecida. Erro: {error}\nTópico: {topic}\nContéudo: {msg_content}")
}

/// "A abrir a válvula em modo simulação: sector "
#[inline]
pub fn dbg_wtr_adptr_simul_valve_open(sector_name: &str) -> String {
    string_concat!("A abrir a válvula em modo simulação: sector ", sector_name)
}
/// "A fechar a válvula em modo simulação: sector "
#[inline]
pub fn dbg_wtr_adptr_simul_valve_closed(sector_name: &str) -> String {
    string_concat!("A fechar a válvula em modo simulação: sector ", sector_name)
}
/// "A abrir a válvula: sector {}"
#[inline]
pub fn dbg_wtr_adptr_valve_open(sector_name: &str) -> String {
    string_concat!("A abrir a válvula: sector ", sector_name)
}
/// "A fechar a válvula: sector {}"
#[inline]
pub fn dbg_wtr_adptr_valve_closed(sector_name: &str) -> String {
    string_concat!("A fechar a válvula: sector ", sector_name)
}
/// "A valvula do sector: {} estava fechada na inicialização"
#[inline]
pub fn warn_wtr_adptr_close_valve_on_init(sector_name: &str) -> String {
    format!("A valvula do sector: {sector_name} estava fechada na inicialização")
}
pub static PHADPT_CRITICAL_IO: &str = "Problema no interface/relé";

pub static INFO_WEB_SRVR_STARTING_1P: &str = "A inicializar o Web Server em ";

/// "Tempo recolhido para o dia: {}. Probabilidade de chover é:{:.0}"
#[inline]
pub fn dbg_wthr_gathered(date: &str, rain_probability: f32) -> String {
    format!("Tempo recolhido para o dia: {date}. Probabilidade de chover é:{rain_probability:.0}")
}
/// "Não foi possivel avaliar a chuva: "
#[inline]
pub fn err_getting_rain(error: &str) -> String {
    string_concat!("Não foi possivel avaliar a chuva: ", error)
}

/// "Arranque de ciclos de rega: encontrámos {} ciclos"
#[inline]
pub fn info_cycle_start(nr_of_cycles: u8) -> String {
    format!("Arranque de ciclos de rega: encontrámos {nr_of_cycles} ciclos")
}
/// "Terminou o ciclo de rega! Inicio: {}  Fim: {}"
#[inline]
pub fn info_cycle_end(start: &str, end: &str) -> String {
    format!("Terminou o ciclo de rega! Inicio: {start}  Fim: {end}")
}
/// "Criámos um ciclo no reschedule de {} setores."
#[inline]
pub fn info_new_cycle(nr_of_sectors: usize) -> String {
    format!("Criámos um ciclo no reschedule de {nr_of_sectors} setores.")
}
/// "O setor {} não foi regado."
#[inline]
pub fn info_sector_not_watered(sector_name: &str) -> String {
    format!("O setor {sector_name} não foi regado.")
}
/// "A iniciar o ciclo '{}' em '{}'"
#[inline]
pub fn info_cycle_time(cycle_name: &str, date: &str) -> String {
    format!("A iniciar o ciclo '{cycle_name}' em '{date}'")
}
pub static INFO_CYCLE_ADD: &str = "Adicionado um novo ciclo de rega.";
pub static INFO_CYCLE_DEL: &str = "Removeu-se um ciclo de rega.";
pub static INFO_CYCLE_UPD: &str = "Atualizou-se um ciclo de rega.";
pub static INFO_SECTORS_UPD: &str = "Atualizou-se informação dos setores.";
pub static INFO_CONFIG_UPD: &str = "Atualizou-se informação da configuração da rega.";
/// "Arranque máquina de rega no modo {}"
#[inline]
pub fn info_starting_mode(mode: &str) -> String {
    string_concat!("Arranque máquina de rega no modo ", mode)
}
pub static INFO_WZRD_ADITIONAL_CYCLE: &str = "Ciclo adicional criado no modo wizard.";

/// "Estamos a parar um ciclo que não está ativo: {}"
#[inline]
pub fn warn_stop_inactive_cycle(cycle_name: &str) -> String {
    string_concat!("Estamos a parar um ciclo que não está ativo: ", cycle_name)
}
pub static WARN_WATER_CYCLE_CONFLICT: &str = "Evento de rega detetado durante a execução de outro ciclo.";
pub static WARN_FORCED_SECTOR_ISSUE: &str = "Mensagem de ativação manual do setor com problema.";
pub static WARN_NO_RAIN_INFO: &str = "Não foi possivel avaliar a chuva.";
/// "Não temos metereologia para o dia: {}"
#[inline]
pub fn warn_no_wthr_info(date: &str) -> String {
    string_concat!("Não temos metereologia para o dia: ", date)
}

/// "Erro no arranque do ciclo: {}"
#[inline]
pub fn err_cycle_start(cycle_name: &str) -> String {
    string_concat!("Erro no arranque do ciclo: ", cycle_name)
}

// /// Para obter uma string sem parametros, não é preciso a macro
// #[macro_export]
// macro_rules! msg {
//     ($a:expr,$($b:expr),*) => {
//         res($a,&[$($b),*])
//     };
// }

// #[test]
// fn call_macro_msg() {
//     println!("{}", msg!("teste{}", 1));
//     println!("{}", msg!("teste{}{}", 1, 2));
//     println!("{}", msg!("teste{}{}{}", 1, 2, 3));
// }
