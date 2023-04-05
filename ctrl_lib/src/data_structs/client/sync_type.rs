#[repr(u8)]
pub enum SyncType {
    FULL,
    PARTIAL,
    UNDEFINED,
}

impl SyncType {
    #[inline]
    pub fn to_str(&self) -> String {
        match self {
            SyncType::FULL => "F".to_owned(),
            SyncType::PARTIAL => "P".to_owned(),
            SyncType::UNDEFINED => "U".to_owned(),
        }
    }
}
