use std::ffi::c_double;
use std::sync::Arc;

pub use parking_lot::Mutex as DBMutex;
use parking_lot::MutexGuard;
// use no_deadlocks::MutexGuard;
// pub use no_deadlocks::Mutex as DBMutex;

use crate::config::{db_cfg::*, *};
use crate::db::db_error::*;
use crate::{log_error, logger::*};
use ctrl_prelude::error::build_error;
use rusqlite::{config::*, functions::*, *};

pub type SqlResult<'stmt> = Rows<'stmt>;

///Dimension = 8
#[derive(Clone)]
pub struct Persist(Arc<DBMutex<InnerPersistance>>);

impl Persist {
    #[allow(clippy::new_without_default)]
    #[inline]
    #[rustfmt::skip]
    pub fn new() -> Self { Self(Arc::new(DBMutex::new(InnerPersistance::new()))) }

    #[inline]
    #[rustfmt::skip]
    pub fn get_conn(&self) -> MutexGuard<InnerPersistance> { self.0.lock() }
}

pub struct LightPersist{
    pub config: DBConfig,
    pub conn: Connection,
}

impl LightPersist {
    #[allow(clippy::new_without_default)]
    #[inline]
    #[rustfmt::skip]
    pub fn new() -> Self {
        
        let config = DBConfig::new();

        let flags = OpenFlags::SQLITE_OPEN_NO_MUTEX | OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_PRIVATE_CACHE;
        let conection_result = Connection::open_with_flags(&config.db_name, flags);
        let conn = match conection_result {
            Ok(conn) => {
               _config_conn(&conn);
               conn
            }
            Err(e) => {
                let msg = build_error(&e);
                eprintln!("{}", &msg);
                log_error!(&msg);
                panic!(); // the is no app without a functional db
            }
        };
        LightPersist { config, conn, }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn exec_prep(&self, stmt: &mut CachedStatement) -> SimpleResult { _exec_prep_(stmt) }

    #[inline]
    #[rustfmt::skip]
    pub fn get_conn(&self) -> &Connection { &self.conn }

}

#[inline]
fn _config_conn(conn: &Connection) {
    _ = conn.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY, false);
    _ = conn.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FTS3_TOKENIZER, false);
    _ = conn.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER, false);
    _ = conn.set_db_config(DbConfig::SQLITE_DBCONFIG_TRIGGER_EQP, false);
    _ = conn.pragma_update(Some(DatabaseName::Main), "cache_size", 2i8);
    _ = conn.pragma_update(Some(DatabaseName::Main), "cell_size_check", false);
    _ = conn.pragma_update(Some(DatabaseName::Main), "secure_delete", false);
    _ = conn.pragma_update(Some(DatabaseName::Main), "temp_store", 2i8);
    _ = conn.pragma_update(Some(DatabaseName::Main), "threads", 4i8);
    _ = conn.pragma_update(Some(DatabaseName::Main), "cache_size", 100i8);
    _ = conn.pragma_update(Some(DatabaseName::Main), "journal_mode", "WAL");
    //WAL?TRUNCATE //testar o que é mais rápido nas operações read e write
    _ = conn.pragma_update(Some(DatabaseName::Main), "synchronous", 1i8);

    _ = conn.pragma_update(Some(DatabaseName::Main), "mmap_size", 262144i64);
    _ = conn.pragma_update(Some(DatabaseName::Main), "secure_delete", "off");
}

#[inline]
fn _exec_prep_(stmt: &mut CachedStatement) -> SimpleResult {
    let result = stmt.raw_execute();
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("{}", build_error(&e));
            Err(DBError::SqlError(e))
        }
    }
}

/// Dimension = 176
pub struct InnerPersistance {
    pub config: DBConfig,
    pub conn: Connection,
}

pub trait DB {
    fn get_conn(&self) -> MutexGuard<'_, InnerPersistance>;
    fn exec_prep(&self, stmt: &mut CachedStatement) -> SimpleResult;
}

impl DB for Persist {
    #[inline]
    #[rustfmt::skip]
    fn get_conn(&self) -> MutexGuard<'_, InnerPersistance> { self.get_conn() }

    #[inline]
    #[rustfmt::skip]
    fn exec_prep(&self, stmt: &mut CachedStatement) -> SimpleResult { _exec_prep_(stmt) }
}

#[inline]
fn atan2(ctx: &Context<'_>) -> Result<c_double> {
    assert_eq!(ctx.len(), 2, "atan2 called with unexpected number of arguments");
    let value1 = ctx.get::<c_double>(0)?;
    let value2 = ctx.get::<c_double>(1)?;
    Ok(f64::atan2(value1, value2))
}

#[inline]
fn degrees(ctx: &Context<'_>) -> Result<c_double> {
    assert_eq!(ctx.len(), 1, "degrees called with unexpected number of arguments");
    let value1 = ctx.get::<c_double>(DB_FLOAT)?;
    Ok(f64::to_degrees(value1))
}

#[inline]
fn radians(ctx: &Context<'_>) -> Result<c_double> {
    assert_eq!(ctx.len(), 1, "radians called with unexpected number of arguments");
    let value1 = ctx.get::<c_double>(0)?;
    Ok(f64::to_radians(value1))
}

#[inline]
fn sin(ctx: &Context<'_>) -> Result<c_double> {
    assert_eq!(ctx.len(), 1, "sin called with unexpected number of arguments");
    let value1 = ctx.get::<c_double>(0)?;
    Ok(f64::sin(value1))
}

#[inline]
fn cos(ctx: &Context<'_>) -> Result<c_double> {
    assert_eq!(ctx.len(), 1, "cos called with unexpected number of arguments");
    let value1 = ctx.get::<c_double>(0)?;
    Ok(f64::cos(value1))
}

impl InnerPersistance {
    #[allow(clippy::new_without_default)]
    #[rustfmt::skip]
    #[inline]
    pub fn new() -> InnerPersistance {
        let config = DBConfig::new();

        let flags = OpenFlags::SQLITE_OPEN_NO_MUTEX | OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_PRIVATE_CACHE;
        let conection_result = Connection::open_with_flags(&config.db_name, flags);
        let conn = match conection_result {
            Ok(conn) => {
                _config_conn(&conn);

                _ = conn.create_scalar_function("atan2", 2, FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC, atan2);
                _ = conn.create_scalar_function("degrees", 1, FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC, degrees);
                _ = conn.create_scalar_function("radians", 1, FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC, radians);
                _ = conn.create_scalar_function("sin", 1, FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC, sin);
                _ = conn.create_scalar_function("cos", 1, FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC, cos);
                conn
            }
            Err(e) => {
                let msg = build_error(&e);
                eprintln!("{}", &msg);
                log_error!(&msg);
                panic!(); // there is no application without a functional db
            }
        };
        InnerPersistance { config, conn, }
    }
}


// assumes that we always have a row with only one column
#[inline]
pub fn get_row_val_f32(result_value: Result<Option<&Row>>) -> Option<f32> {
    let result_val: Option<f32> = match result_value {
        Ok(Some(row)) => Some(row.get::<usize, f32>(DB_FLOAT).unwrap()),
        Err(e) => {
            log_error!(build_error(&e));
            None
        }
        _ => unreachable!(), // as per defined sql statment we should have always one line, even is it is 0 or N/A
    };
    result_val
}
