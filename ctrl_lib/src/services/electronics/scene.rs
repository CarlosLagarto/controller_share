use serde::{Deserialize, Serialize};

use crate::db::SqlRow;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Scene{
    pub id: u16,
    pub desc: String,
    pub devices: Vec<u16>,
}

impl From<&SqlRow<'_>> for Scene {
    #[inline]
    fn from(sql_row: &SqlRow) -> Scene {
        let sql_row = sql_row;

        Scene {
            id: sql_row.get(0).unwrap(),
            desc: sql_row.get(1).unwrap(),
            devices: Vec::new(),
        }
    }
}