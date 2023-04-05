use std::time::Instant;

use ctrl_lib::{
    config::app_config::*,
    db::{db_error::SimpleResult, db_sql_lite::*},
    utils::elapsed_dyn, app_time::ctrl_time::CtrlTime,
};
// use serial_test::serial;

#[test]

fn test_save() {
    let mut cfg = AppCfg::new(Persist::new());
    cfg.changed = true;
    let _res = cfg.save_if_updated(CtrlTime::sys_time());
}

pub trait DBModelBindTest<'a>: DB {
    const INSERT_MODULE_CONFIG: &'a str = "insert into mods_data (module, param, float, int,string, name, descricao) values(99,?,?,?,?,?,?);";
    const DELETE_MODULE99_CONFIG: &'a str = "delete from mods_data where module= 99;";

    fn test_bind_1_prepare(&mut self) {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::INSERT_MODULE_CONFIG).unwrap();

        let mut _res: SimpleResult;

        let mut _res1 = stmt.raw_bind_parameter(1, 98);
        let mut _res1 = stmt.raw_bind_parameter(2, 0);
        let mut _res1 = stmt.raw_bind_parameter(5, "teste 0");

        let mut _res2 = self.exec_prep(&mut stmt);

        //  stmt = conn.unwrap().prepare_cached(Self::INSERT_MODULE_CONFIG).unwrap();
        let mut _res1 = stmt.raw_bind_parameter(1, 99);
        let mut _res1 = stmt.raw_bind_parameter(2, 1);
        let mut _res1 = stmt.raw_bind_parameter(3, 0);
        let mut _res1 = stmt.raw_bind_parameter(5, "teste 1");

        let mut _res2 = self.exec_prep(&mut stmt);
    }

    fn test_bind_2_prepare(&mut self) {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::INSERT_MODULE_CONFIG).unwrap();

        let mut _res: SimpleResult;

        let mut _res1 = stmt.raw_bind_parameter(1, 98);
        let mut _res1 = stmt.raw_bind_parameter(2, 0);
        let mut _res1 = stmt.raw_bind_parameter(5, "teste 0");

        let mut _res2 = self.exec_prep(&mut stmt);

        stmt = conn.prepare_cached(Self::INSERT_MODULE_CONFIG).unwrap();
        let mut _res1 = stmt.raw_bind_parameter(1, 99);
        let mut _res1 = stmt.raw_bind_parameter(3, 0);
        let mut _res1 = stmt.raw_bind_parameter(5, "teste 1");

        let mut _res2 = self.exec_prep(&mut stmt);
    }

    fn test_delete(&mut self) {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::DELETE_MODULE99_CONFIG).unwrap();

        let mut _res: SimpleResult;
        let mut _res2 = self.exec_prep(&mut stmt);
    }
}

impl<'a> DBModelBindTest<'a> for Persist {}

// bem, concluimos que eexiste uma função para fazer o reset dos bindings, mas não está publica.
// o que quer dizer que temos que manter
#[test]

fn test_binds() {
    let mut db = Persist::new();
    let mut t0: Instant;
    let mut total: u64 = 0;
    for _i in 0..50 {
        t0 = Instant::now();
        db.test_bind_1_prepare();
        total += t0.elapsed().as_nanos() as u64;
        db.test_delete();
    }

    println!("'standard' batch: {}", elapsed_dyn(total/50));
    total = 0;
    for _i in 0..50 {
        t0 = Instant::now();
        db.test_bind_2_prepare();
        total += t0.elapsed().as_nanos() as u64;
        db.test_delete();
    }
    println!("recreate stmt: {}", elapsed_dyn(total/50));
}
