use std::fmt::{self};
use std::time::{Duration, SystemTime};

use serde::{self, Deserialize, Serialize};

use crate::app_time::{date_time::*, parse_error::*};
use crate::{log_error, logger::error};

use ctrl_prelude::{domain_types::*, globals::*};

/// this is the time for the backend machine (watering and whatever is needed)
pub static mut STARTUP_TIME: u64 = 0;

/// Dimension = 8
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct CtrlTime(pub u64);

const MONTH_SHORT_EN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const MONTH_DAYS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

pub fn ordinal_month(month: &str) -> u8 {
    match month {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _=> 0,
    }
}
// array used in year_day_number
const LUMP_SUM_YEAR_DAYS: [u16; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

pub(crate) const TBL_DIFFS: [[u8; 7]; 7] = [
    [7, 1, 2, 3, 4, 5, 6],
    [6, 7, 1, 2, 3, 4, 5],
    [5, 6, 7, 1, 2, 3, 4],
    [4, 5, 6, 7, 1, 2, 3],
    [3, 4, 5, 6, 7, 1, 2],
    [2, 3, 4, 5, 6, 7, 1],
    [1, 2, 3, 4, 5, 6, 7],
];

impl fmt::Display for CtrlTime {
    #[rustfmt::skip]
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

impl PartialOrd for CtrlTime {
    #[inline]
    #[rustfmt::skip]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { self.0.partial_cmp(&other.0) }
}

impl PartialEq for CtrlTime {
    #[inline]
    #[rustfmt::skip]
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl Eq for CtrlTime {}

impl Ord for CtrlTime {
    #[inline]
    #[rustfmt::skip]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }

    #[inline]
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        std::cmp::max_by(self, other, Ord::cmp)
    }

    #[inline]
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        std::cmp::min_by(self, other, Ord::cmp)
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> CtrlTime
    where
        Self: Sized,
    {
        CtrlTime(self.0.clamp(min.0, max.0))
    }
}

impl ::core::ops::Add<i64> for CtrlTime {
    type Output = CtrlTime;
    /// Add nr of nanos to time
    #[inline]
    #[rustfmt::skip]
    fn add(self, rhs: i64) -> CtrlTime { CtrlTime(self.0.add(rhs as u64)) }
}

impl ::core::ops::Add<u64> for CtrlTime {
    type Output = CtrlTime;
    #[inline]
    #[rustfmt::skip]
    fn add(self, rhs: u64) -> CtrlTime { CtrlTime(self.0.add(rhs)) }
}

impl ::core::ops::Sub<u64> for CtrlTime {
    type Output = CtrlTime;
    #[inline]
    #[rustfmt::skip]
    fn sub(self, rhs: u64) -> CtrlTime { CtrlTime(self.0 - rhs) }
}

impl CtrlTime {
    pub const MAX: u64 = 3_408_134_400_000_000_000;
    pub const WEEK_DAYS_U8: u8 = 7;
    pub const WEEK_DAYS_U64: u64 = 7;
    pub const WEEK_DAYS_I8: i8 = 7;
    pub const WEEK_DAYS_U16: u16 = 7;
    pub const WEEK_DAYS_F64: f64 = 7.0;
    pub const NR_NANOS_IN_A_WEEK: u64 = 604_800_000_000_000;
    pub const NR_NANOS_IN_A_DAY: u64 = 86_400_000_000_000;
    pub const NR_NANOS_IN_A_DAY_F64: f64 = 86_400_000_000_000.;
    // pub const NR_NANOS_IN_A_HALF_DAY: u64 = 43_200_000_000_000;
    pub const NR_NANOS_IN_A_HOUR: u64 = 3_600_000_000_000;
    pub const NR_NANOS_IN_A_MINUTE: u64 = 60_000_000_000; // pub const NR_NANOS_IN_TWO_DAYS: u64 = 172_800_000_000_000;
    pub const MIN_YEAR: u16 = 1970;
    pub const MAX_YEAR: u16 = 2077;
    pub const SECS_IN_A_DAY: u64 = 86400;

    /// "%Y-%m-%dT%H:%M:%S%";
    #[inline]
    pub fn try_parse_str_iso_rfc3339_to_ctrl_time(d: &str) -> ParseDateResult<CtrlTime> {
        let v: Vec<&str> = d.split(&['-', 'T', ':'][..]).collect();

        let mut date = DateTimeE::new_ymdhmsn(0, 0, 0, 0, 0, 0, 0);
        if v.len() != 6 {
            return Err(ParseError::UnknownDate(String::from(d)));
        }
        let res = v[0].parse::<u16>();
        if let Ok(year) = res {
            if (CtrlTime::MIN_YEAR..=CtrlTime::MAX_YEAR).contains(&year) {
                date.year = year;
            } else {
                return Err(ParseError::InvalidYear);
            }
        }
        let res = v[1].parse::<u8>();
        if let Ok(month) = res {
            if (1..=12).contains(&month) {
                date.month = month;
            } else {
                return Err(ParseError::InvalidMonth);
            }
        }
        let res = v[2].parse::<u8>();
        if let Ok(day) = res {
            let mut days = MONTH_DAYS[date.month as usize];
            if date.month == 2 {
                days += is_leap(date.year) as u8;
            }
            if (0..=days).contains(&day) {
                date.day = day;
            } else {
                return Err(ParseError::InvalidDay(date.month, day));
            }
        }
        let res = v[3].parse::<u8>();
        if let Ok(hour) = res {
            if (0..24).contains(&hour) {
                date.hour = hour;
            } else {
                return Err(ParseError::InvalidHour);
            }
        }
        let res = v[4].parse::<u8>();
        if let Ok(min) = res {
            if (0..60).contains(&min) {
                date.min = min;
            } else {
                return Err(ParseError::InvalidMinutes);
            }
        }
        let res = v[5].parse::<u8>();
        if let Ok(sec) = res {
            if (0..60).contains(&sec) {
                date.sec = sec;
            } else {
                return Err(ParseError::InvalidSeconds);
            }
        }
        Ok(CtrlTime::from_utc_date_time_e(&date))
    }

    #[inline]
    pub fn as_utc_date_time_e(&self) -> DateTimeE {
        // above spec, panic
        // bellow spec, not tested
        if self.0 <= Self::MAX {
            // hours, min, secs & nanos
            let mut t = self.0 / GIGA_U;
            let nanos = (self.0 % GIGA_U) as u32;
            let sec = (t % 60) as u8;
            t /= 60;
            let min = (t % 60) as u8;
            t /= 60;
            let hour = (t % 24) as u8;
            t /= 24;

            // to date
            let a = (4 * t + 102032) / 146097 + 15;
            let b = t + 2442113 + a - (a / 4);
            let mut year = (20 * b - 2442) / 7305;
            let d = b - 365 * year - (year / 4);
            let mut month = d * 1000 / 30601;
            let day = d - month * 30 - month * 601 / 1000;

            // January and February are months 13 and 14 of previous year
            if month <= 13 {
                year -= 4716;
                month -= 1;
            } else {
                year -= 4715;
                month -= 13;
            }
            DateTimeE::new_ymdhmsn(year as u16, month as u8, day as u8, hour, min, sec, nanos)
        } else {
            let msg = format!("Só se aceita datas inferiores a {}", CtrlTime(Self::MAX - 1).as_rfc3339_str_e());
            log_error!(&msg);
            eprintln!("{}", &msg);
            panic!()
        }
    }

    #[inline]
    pub const fn from_utc_date_time_e(date_time: &DateTimeE) -> CtrlTime {
        assert!(date_time.year >= 1970);
        let mut y: u64 = date_time.year as u64; // year
        let mut m = date_time.month as u64; // month
        let d = date_time.day as u64; // day

        // January and February are months 13 and 14 of previous year
        if m <= 2 {
            m += 12;
            y -= 1;
        }

        let mut t = (365 * y) + (y / 4) - (y / 100) + (y / 400); // convert years to days
        t += (30 * m) + (3 * (m + 1) / 5) + d; // convert months to days
        t -= 719561; // unix epoch start at 1/Jan/1970
        t *= 86400; // convert days to seconds
        t += (3600 * date_time.hour as u64) + (60 * date_time.min as u64) + date_time.sec as u64; // add hours, minutes and secs
        t *= GIGA_U; // // convert seconds to nanos

        CtrlTime(t + date_time.nanos as u64)
    }

    #[inline]
    pub const fn from_utc_parts(year: u16, month: u8, day: u8, hour: u8, min: u8, sec: u8) -> CtrlTime {
        CtrlTime::from_utc_date_time_e(&DateTimeE::new_ymdhms(year, month, day, hour, min, sec))
    }

    #[inline]
    #[rustfmt::skip]
    pub const fn ux_ts(&self) -> UTC_UNIX_TIME { self.0 / GIGA_U } //only care for the seconds

    #[inline]
    #[rustfmt::skip]
    pub const fn from_ux_ts(ux_ts: UTC_UNIX_TIME) -> CtrlTime { CtrlTime(ux_ts * GIGA_U)} //only care for the seconds

    #[inline]
    #[rustfmt::skip]
    pub fn sys_time() -> CtrlTime { CtrlTime(CtrlTime::sys_time_nanos()) }

    #[inline]
    pub fn sys_time_nanos() -> APP_TIME {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() as APP_TIME
    }

    #[inline]
    pub fn sys_time_duration() -> Duration {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap()
    }

    #[inline]
    #[rustfmt::skip]
    pub fn ts(&self) -> UTC_UNIX_TIME_HR { self.0 as f64 * NANO_F }

    /// STD_IN_DATE_S: &str = "%Y/%m/%d %H:%M:%S";
    #[inline]
    pub fn as_date_web_str_e(&self) -> String {
        let mut output = String::new();
        let d = self.as_utc_date_time_e();
        fmt::write(&mut output, format_args!("{:04} {} {:02} {:02}:{:02}", d.year, MONTH_SHORT_EN[(d.month - 1) as usize], d.day, d.hour, d.min)).unwrap();
        output
    }

    /// STD_IN_DATE_ISO_RFC3339: &str = "%Y-%m-%dT%H:%M:%S%.fZ";
    #[inline]
    pub fn as_rfc3339_str_e(&self) -> String {
        let mut output = String::new();
        let d = self.as_utc_date_time_e();
        fmt::write(
            &mut output,
            format_args!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03.0}", d.year, d.month, d.day, d.hour, d.min, d.sec, (d.nanos as f64 * MICR_F)),
        )
        .unwrap();
        output
    }

    /// End Of Day
    #[inline]
    pub fn eod_ux_e(&self) -> CtrlTime {
        let mut dt = self.as_utc_date_time_e();
        dt.hour = 23;
        dt.min = 59;
        dt.sec = 59;
        dt.nanos = 999999;
        CtrlTime::from_utc_date_time_e(&dt)
    }

    /// STD_IN_DATE_CHAR8: &str = "%Y%m%d";
    #[inline]
    pub fn as_date_char8_str_e(&self) -> String {
        let mut output = String::new();
        let d = self.as_utc_date_time_e();
        fmt::write(&mut output, format_args!("{:04}{:02}{:02}", d.year, d.month, d.day)).unwrap();
        output
    }

    /// Start of Day
    #[inline]
    pub fn sod_ux_e(&self) -> CtrlTime {
        let mut dt = self.as_utc_date_time_e();
        dt.hour = 0;
        dt.min = 0;
        dt.sec = 0;
        dt.nanos = 0;
        CtrlTime::from_utc_date_time_e(&dt)
    }

    #[inline]
    pub const fn add_days(&self, nr_days: u64) -> CtrlTime {
        CtrlTime(self.0 + (nr_days * Self::NR_NANOS_IN_A_DAY))
    }

    #[inline]
    pub const fn sub_days(&self, nr_days: u64) -> CtrlTime {
        CtrlTime(self.0 - (nr_days * Self::NR_NANOS_IN_A_DAY))
    }

    #[inline]
    pub fn add_secs_f32(&self, nr_secs: f32) -> CtrlTime {
        CtrlTime(self.0 + (nr_secs as f64 * GIGA_F) as u64)
    }

    #[inline]
    pub const fn add_secs(&self, nr_secs: u64) -> CtrlTime {
        CtrlTime(self.0 + (nr_secs * GIGA_U))
    }

    #[inline]
    pub fn sub_secs_f32(&self, nr_secs: f32) -> CtrlTime {
        CtrlTime(self.0 - (nr_secs as f64 * GIGA_F) as u64)
    }

    #[inline]
    pub fn year_day_number_e(&self) -> u16 {
        let dt = self.as_utc_date_time_e();
        LUMP_SUM_YEAR_DAYS[(dt.month - 1) as usize] + dt.day as u16 + is_leap(dt.year)
    }

    /// Sunday = 0, Monday = 1 ...  ATENÇÃO:  Isto não é ISO, porque o ISO começa com 0 na segunda feira.
    #[inline]
    pub const fn week_day_e(&self) -> u8 {
        (((self.0 / Self::NR_NANOS_IN_A_DAY) as u16 + 4) % Self::WEEK_DAYS_U16) as u8
    }

    #[inline]
    pub const fn nr_of_days_since(&self, tm: CtrlTime) -> u64 {
        assert!(self.0 >= tm.0);
        (self.0 - tm.0) / Self::NR_NANOS_IN_A_DAY
    }
}

/// is_leap return 1, if it is a leap year.  Returns 0 otherwise
#[inline]
pub fn is_leap(year: u16) -> u16 {
    // if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
    //     1
    // } else {
    //     0
    // }
    u16::from(year % 4 == 0 && (year % 100 != 0 || year % 400 == 0))
}

#[inline]
#[rustfmt::skip]
pub fn setup_start_time(start_date: CtrlTime) {
    // only called once on program start, with only the main thread running, so compiler is free to reorder instructions
    // this unsafe is safe because the value only changes here
    unsafe { STARTUP_TIME = start_date.0; }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use crate::{
        app_time::{
            ctrl_time::{ordinal_month, CtrlTime},
            date_time::DateTimeE,
            tm::get_utc_offset,
        },
        lib_serde::{data_from_str, data_to_str},
        utils::elapsed_dyn,
    };
    use serde::{self, Deserialize, Serialize};

    #[test]
    fn ctrl_time_serialize() {
        #[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct X {
            t: CtrlTime,
            a: i32,
        }

        const TEST_REF: X = X {
            t: CtrlTime(1648726045326800000),
            a: 12,
        };
        let a: X = TEST_REF.clone();
        let s = "{\"t\":1648726045326800000,\"a\":12}";
        assert_eq!(s, data_to_str(&a).unwrap());
    }

    #[test]
    fn ctrl_time_deserialize() {
        #[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct X {
            t: CtrlTime,
            a: i32,
        }

        const TEST_REF: X = X {
            t: CtrlTime(1648726045326800000),
            a: 12,
        };
        let s = "{\"t\":1648726045326800000,\"a\":12}";

        let b = TEST_REF.clone();
        let a: X = data_from_str(s).unwrap();
        assert!(a == b);
    }

    fn str_to_dt(s: &str) -> DateTimeE {
        //Mar 26 00:59:59 2023
        let v: Vec<&str> = s.split([' ', ':']).collect();
        // let smonth: &str = &s[0..2];
        println!("string:{}",s);
        println!("split v: {:?}", v);
        let month = ordinal_month(v[0]);
        let day = v[1].parse().unwrap();
        let hour = v[2].parse().unwrap();
        let min = v[3].parse().unwrap();
        let sec = v[4].parse().unwrap();
        let year = v[5].parse().unwrap();

        DateTimeE::new_ymdhms(year, month, day, hour, min, sec)
    }

    #[test]
    fn test_offset() {
        // DESIGN NOTE
        // on program start gets the current utc offset from the system
        // and schedule the next hour change
        // and the main control loop tests every second if is time to run the update offset procedure (scheduled change) to inform the client
        // zdump -v Europe/Lisbon
        let t = SystemTime::now();
        use std::process::Command;

        let t1 = CtrlTime::sys_time();
        let d1 = t1.as_utc_date_time_e();
        let output = Command::new("zdump")
            .arg("-V")
            .arg("-c")
            .arg(format!("{},{}", d1.year, d1.year + 1))
            .arg("Europe/Lisbon")
            .output()
            .expect("failed to execute process");
        let s = match std::str::from_utf8(&output.stdout) {
            Ok(v) => v,
            Err(_) => "",
        };
        let v: Vec<&str> = s.split('\n').collect();
        // println!("vec:\n{:?}", v);

        //Europe/Lisbon  Sun Mar 26 00:59:59 2023
        //Europe/Lisbon  Sun Mar 26 00:59:59 2023 UT = Sun Mar 26 00:59:59 2023 WET isdst=0 gmtoff=0
        println!("vetor output:\n{:?}", v);
        let d1a = str_to_dt(&v[1][19..39]);
        let d1b = str_to_dt(&v[1][49..69]);
        let d2a = str_to_dt(&v[3][19..39]);
        let d2b = str_to_dt(&v[3][49..69]);
        let os1 = d1b.hour - d1a.hour;
        let os2 = d2b.hour - d2a.hour;

        println!("tempo: {}", elapsed_dyn(t.elapsed().unwrap().as_nanos() as u64));
        println!("Output\n{}", s);


        println!("offset 1: {}", os1);
        println!("offset 2: {}", os2);
        // let time = CtrlTime::sys_time();
        // let ts = time.ux_ts();

        let t = SystemTime::now();
        let os = get_utc_offset().unwrap();
        println!("tempo: {}", elapsed_dyn(t.elapsed().unwrap().as_nanos() as u64));
        println!("offset:\nhoras:{}\nmin:{}\nsef:{}", os.hours, os.minutes, os.seconds);
    }
}
