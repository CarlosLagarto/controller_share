use crate::db::SqlRow;

pub type HistoryList = Vec<HistoryValue>;

/// Dimension = 32
pub struct HistoryValue {
    pub minutets: u64,
    pub diff: i64,
    pub val1: f64,
    pub val2: f64,
}

impl HistoryValue {
    #[inline]
    pub fn from_db_row(mysql_row: &SqlRow) -> HistoryValue {
        HistoryValue {
            minutets: mysql_row.get(0).unwrap(),
            diff: mysql_row.get::<usize, f64>(1).unwrap().round() as i64,
            val1: mysql_row.get(2).unwrap(),
            val2: mysql_row.get(3).unwrap(),
        }
    }
}
