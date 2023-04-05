/// Date
///
/// - year: u16
/// - month: u8
/// - day: u8
/// - hour: u8
/// - min: u8
/// - sec: u8
/// - nanos: u32
///
/// Dimension 12
#[derive(Debug, PartialEq, Eq)]
pub struct DateTimeE {
    pub year: u16,  
    pub month: u8,  
    pub day: u8,    
    pub hour: u8,  
    pub min: u8,   
    pub sec: u8,   
    pub nanos: u32,
}

impl DateTimeE {
    #[inline]
    pub const fn new_ymdhmsn(year: u16, month: u8, day: u8, hour: u8, min: u8, sec: u8, nanos: u32) -> DateTimeE {
        DateTimeE { year, month, day, hour, min, sec, nanos }
    }
    #[inline]
    pub const fn new_ymdhms(year: u16, month: u8, day: u8, hour: u8, min: u8, sec: u8) -> DateTimeE {
        DateTimeE { year, month, day, hour, min, sec, nanos: 0 }
    }
    #[inline]
    #[cfg(test)]
    pub const fn new_ymdhm(year: u16, month: u8, day: u8, hour: u8, min: u8) -> DateTimeE {
        DateTimeE { year, month, day, hour, min, sec: 0, nanos: 0 }
    }
    #[inline]
    #[cfg(test)]
    pub const fn new_ymdh(year: u16, month: u8, day: u8, hour: u8) -> DateTimeE {
        DateTimeE { year, month, day, hour, min: 0, sec: 0, nanos: 0 }
    }
    #[inline]
    #[cfg(test)]
    pub const fn new_ymd(year: u16, month: u8, day: u8) -> DateTimeE {
        DateTimeE { year, month, day, hour: 0, min: 0, sec: 0, nanos: 0 }
    }
}
