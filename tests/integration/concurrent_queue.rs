use std::{sync::Arc, thread, time::Duration};

use ctrl_lib::data_structs::concurrent_queue::*;
use ctrl_prelude::globals::{GIGA_F, SHUTTING_DOWN};

// use crate::data_structs::concurrent_queue::MtDeque;
use std::thread::JoinHandle;

#[test]
fn test_queue() {
    let wq: Arc<MtDeque<u32>> = Arc::new(MtDeque::new());
    // let rq = wq.clone();
    let mut wt: Vec<JoinHandle<()>> = Vec::new();
    for i in 0..10 {
        let t = wq.clone();
        wt.push(thread::spawn(move || {
            println!("sending: {}", i);
            t.clone().send(i);
        }));
    }
    for _ in 0..10 {
        let t = wq.clone();
        wt.push(thread::spawn(move || {
            let i = t.recv().unwrap();
            println!("receiving: {}", i);
        }));
    }

    for j in wt {
        j.join().unwrap();
    }
}

#[test]
fn test_queue_2() {
    let wq: Arc<MtDeque<u32>> = Arc::new(MtDeque::new());

    let mut wt: Vec<JoinHandle<()>> = Vec::new();
    let wait = (0.2 * GIGA_F) as u32;

    let t = wq.clone();

    wt.push(thread::spawn(move || {
        for i in 0..50 {
            thread::sleep(Duration::new(0,wait));
            // thread::sleep(Duration::new(1, 0));
            println!("sending: {}", i);
            if !t.clone().send(i){
                break;
            }
        }
        println!("saiu do send...");

    }));

    let t = wq.clone();
    wt.push(thread::spawn(move || {
        while !unsafe { SHUTTING_DOWN } {
            let i = t.recv();
            if let Some(j) = i {
                println!("receiving: {}", j);
            } else {
                
                break;
            }
        }
        println!("saiu do receive");
    }));

    thread::sleep(Duration::new(5, 0));
    unsafe { SHUTTING_DOWN = true };
    wq.terminate();

    for j in wt {
        j.join().unwrap();
    }
}
