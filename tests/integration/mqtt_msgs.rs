use ctrl_lib::data_structs::msgs::{ext_message::ExtMsgIn, topic::Topic};


#[test]
fn client_db_sync(){

    let payload = r#"{"type":"DBSync","header":{"topic":"LAGARTO_CONTROLLER/CTS/DATA/SYNCDB/TEST","client_id":"laptop-ax-01","time":1668438959,"uuid":"3ea22e50-642f-11ed-961f-d7ec119285b3"},"db_sync":{"sync_type":"P","last_sync":1668438959,"cycles":[{"start":1668438660,"end":0,"cycle_id":0,"run_id":0,"status":"Waiting","run_start":0,"repeat_kind":"SpecificWeekday","stop_condition":"Never","stop_retries":0,"stop_date_ts":0,"repeat_spec_wd":85,"repeat_every_qty":1,"repeat_every_unit":"Days","retries_count":0,"name":"DEF1","last_change":1668438959,"last_run":0,"sim":0,"op":"I","cycle_type":3,"sunrise_flg":1,"sunset_flg":0}],"sectors":[]}}"#;

    let msg = ExtMsgIn::new(Topic::from_string("LAGARTO_CONTROLLER/CTS/DATA/SYNCDB/TEST"), &payload);
    println!("{:?}", msg);
    assert!(msg.is_ok());
}

#[test]
fn force_sector(){
    let payload = r#"{"type":"Sector",
                            "header":{"topic":"LAGARTO_CONTROLLER/CTS/STATUS/FORCE_SECTOR/TEST",
                                      "client_id":"laptop-ax-01",
                                      "time":1668802408,
                                      "uuid":"775670c0-677d-11ed-9b41-b75b73afb9eb"},
                            "running_ptr": {"cycle": 0, "sector": 0, "run_sector": 0}}"#;
    let msg = ExtMsgIn::new(Topic::from_string("LAGARTO_CONTROLLER/CTS/STATUS/FORCE_SECTOR/TEST"), &payload);
    println!("{:?}", msg);
    assert!(msg.is_ok());
}

// #[test]
// fn client_db_sync1(){
//     let payload = r#"{"type":"DBSync","header":{"topic":"LAGARTO_CONTROLLER/CTS/DATA/SYNCDB/TEST","client_id":"laptop-ax-01","time":1668717828,"uuid":"89602ab0-66b8-11ed-94b3-f79d968c3bcb"},"db_sync":{"sync_type":"P","last_sync":1668717828,"cycles":[],"sectors":[{"desc":"Zona Sobreiro","name":"Sobreiro","last_watered_in":0,"last_change":1668717828,"deficit":150,"percolation":null,"debit":6.5,"max_duration":20,"stress_score":3,"stress_perc":0,"id":0,"enabled":true,"op":"U","start":1667629080,"end":1667630280,"minutes_to_water":20,"status":"Waiting"}]}}"#;

//     let msg = ExtMsgIn::new(Topic::from_string("LAGARTO_CONTROLLER/CTS/DATA/SYNCDB/TEST"), &payload);
//     println!("{:?}", msg);
//     assert!(msg.is_ok());
// }

#[test]
fn client_db_sync2(){
    let payload = r#"{"type":"DBSync","header":{"topic":"LAGARTO_CONTROLLER/CTS/DATA/SYNCDB/TEST","client_id":"laptop-ax-01_tst","time":1675196512,"uuid":"e62d2460-a1a4-11ed-ae10-5757c2124e39"},"db_sync":{"sync_type":"P","last_sync":1675196512,"cycles":[{"start":1675231200,"end":0,"cycle_id":0,"run_id":0,"status":"Waiting","run_start":0,"repeat_kind":"Every","stop_condition":"Never","stop_retries":0,"stop_date_ts":0,"repeat_spec_wd":0,"repeat_every_qty":1,"repeat_every_unit":"Days","retries_count":0,"name":"Standard 1","last_change":1675196464,"last_run":0,"sim":0,"op":"I","cycle_type":3,"sunrise_flg":1,"sunset_flg":0}],"sectors":[]}}"#;

    let msg = ExtMsgIn::new(Topic::from_string("LAGARTO_CONTROLLER/CTS/DATA/SYNCDB/TEST"), &payload);
    println!("{:?}", msg);
    assert!(msg.is_ok());
}



// #[test]
// fn force_sector_fail(){
//     let payload = r#"{"type":"Sector",
//                             "header":{"topic":"LAGARTO_CONTROLLER/CTS/STATUS/FORCE_SECTOR/TEST",
//                                       "client_id":"laptop-ax-01",
//                                       "time":1668802408,
//                                       "uuid":"775670c0-677d-11ed-9b41-b75b73afb9eb"},
//                                       "id":0}"#;
//     let msg = ExtMsgIn::new(Topic::from_string("LAGARTO_CONTROLLER/CTS/STATUS/FORCE_SECTOR/TEST"), &payload);
//     println!("{:?}", msg);
//     assert!(msg.is_ok());
// }
