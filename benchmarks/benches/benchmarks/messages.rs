use criterion::Criterion;
use ctrl_lib::{
    app_time::ctrl_time::CtrlTime,
    data_structs::msgs::{
        alert::{Alert, AlertType},
        int_message::{IntMessage, MsgData},
        log_error::LogError,
    },
};

fn bench_int_message_data_creation_alert() {
    let _msg_data = MsgData::Alert(Alert { header: None, value: 0., type_: AlertType::WIND });
}

fn bench_int_message_data_creation_log_error() {
    let _msg_data = MsgData::ClientError(LogError { header: None, error: "teste".to_owned() });
}

fn bench_int_message_creation(msg_data: MsgData) {
    let _int_msg = IntMessage::build(msg_data, CtrlTime::sys_time());
}

pub fn bench_int_message(d: &mut Criterion) {
    let mut c = d.benchmark_group("bench_int_message");

    let msg_data_alert = MsgData::Alert(Alert {header: None,  value: 0., type_: AlertType::WIND });
    let msg_data_client_error = MsgData::ClientError(LogError {header: None,  error: "teste".to_owned() });

    c.bench_function("bench_int_message_data_creation_alert", |b| b.iter(|| (bench_int_message_data_creation_alert())));
    c.bench_function("bench_int_message_data_creation_log_error", |b| b.iter(|| (bench_int_message_data_creation_log_error())));
    c.bench_function("bench_int_message_creation_data_alert", |b| b.iter(|| (bench_int_message_creation(msg_data_alert.clone()))));
    c.bench_function("bench_int_message_creation_data_cliente_error", |b| b.iter(|| (bench_int_message_creation(msg_data_client_error.clone()))));

    c.finish();
}
