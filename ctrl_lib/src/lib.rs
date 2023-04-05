pub mod app_context;
pub mod app_time;
pub mod config;
pub mod controller_sync;
pub mod data_structs;
pub mod db;
pub mod lib_serde;
pub mod logger;
pub mod services;
// pub mod thread_pool;
pub mod thread_signal;
pub mod utils;

// A ideia de colocar o serde como uma unica dependencia de uma library/module, e exportá-la a partir daí para reduzir as dependencias nos outros módulos
// deu algum trabalho a perceber como se fzia
// Após alguma pesquisa (várias horas ao longo de alguns dias) descobri uma pista e lá fui olhar para o código do serde (proc_macro)
//
// https://github.com/serde-rs/serde/blob/5a8dcac2ed1407fab3f7fd23f2d56af42dcd448f/serde_derive/src/internals/attr.rs#L556-L561
//
// onde percebi que com o atributo a coisa finalmente compilava #[serde(crate = "self::serde")]
//
pub extern crate serde;
pub extern crate serde_json;
pub use serde::*;
pub use serde_json::*;

pub extern crate string_concat;
pub use string_concat::*;

pub extern crate arrayvec;
pub use arrayvec::*;
