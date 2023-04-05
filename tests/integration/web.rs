use std::{sync::Arc, thread, time};

use ctrl_lib::app_time::ctrl_time::*;
use ctrl_lib::data_structs::concurrent_queue::MtDeque;
use ctrl_lib::data_structs::msgs::int_message::*;
use ctrl_lib::services::msg_broker::subscriber::*;
use ctrl_lib::services::{msg_broker::msg_brkr_svc::*, web_svc::*};

use crate::integration::common::*;

// 2022-3-15 bem, tenho conseguido tirar os locks e tornar tudo dinamico...mas aqui tenho os ports para a web
// o port é unico pelo que uma segunda instancia do webserver sobre o mesmo port parece dar erro.
// 2022-03-17 - já apanhei parte da questão.  Pelo caminho limpei 40 linhas de código ao web server :-)
// O tema prende-se com o bind do tcp listener.  O tcp listener por alguma razão está a fazer o bind com a main tread
// apesar de ser criado numa outra thread.  E apesar de a child thread matar o listener, a main thread (e o sistema operativo)
// mantém um bind ativo (apesar de já não estar) pelo que a 2ª child thread não consegue fazer o bind
//
// a estudar como resolver isto...ou então a fazer testes mais compridos/redesenhar os testes
// para ter as situações de teste desejadas, nas situações em que se tem binds a ports
//
//  Todos os testes associados ao servidor web têm que ser bem coordenadas porque só pode haver uma coisa de cada vez pendurada agarrada ao port http
// porttanto todos os testes associados a ports são mutuamente exclusivos
#[test]
#[rustfmt::skip]
#[ignore]  // isto só pode ser corrido isoladamente por causa do port.  
fn validate_web_service_start_stop_via_web() {
    setup_start_time(CtrlTime::sys_time());
    let web_svc: WebService = WebService::default();

    let msg_broker = Arc::new(MsgBrkr::new());
    let handle_evt_mng = msg_broker.start();
    let subs_queue = Arc::new(MtDeque::<IntMessage>::new());
    msg_broker.register_in_broker(Subscriber::Test, subs_queue);
    let broker_channel = msg_broker.get_msg_brkr_handle();
    msg_broker.subscribe(MsgType::ShutDown, Subscriber::Test);
    let handler_web = web_svc.start(msg_broker.clone());

    thread::sleep(time::Duration::from_secs(1));
    println!("1. event broker is running");

    

    let _res = make_request("http://localhost:5004/tst/controller/shutdown");
    println!("{:?}", _res);
    let _msg = broker_channel.recv();
    println!("6. shutdown received from web");

    web_svc.terminate();
    let mut t0 = time::Instant::now();
    let res = handler_web.join();
    println!("elapsed at validate_web_service_start_stop via_web: {:?}", t0.elapsed());
    if res.is_err() { 
        println!("{:?}", res);
        panic!("X1. web service panic"); }

    //------------------- simula ctrl-c validate_web_service_start_stop via_ctrlc:
    thread::sleep(time::Duration::from_secs(1));
    println!("1. event broker is running");

    let handler_web = web_svc.start(msg_broker.clone());
    let _res = msg_broker.reg_int_msg(MsgData::ShutDown(CtrlTime::sys_time()), CtrlTime::sys_time());

    let _msg = broker_channel.recv();
    println!("6. shutdown received from ctrl-c");

    web_svc.terminate();
    t0 = time::Instant::now();
    let res = handler_web.join();

    println!("elapsed at validate_web_service_start_stop via_ctrlc: {:?}", t0.elapsed());
    if res.is_err() {
        panic!("X1. web service panic");
    }
    //----------------------------------

    let _res = msg_broker.terminate();
    let _res = handle_evt_mng.join();

}
