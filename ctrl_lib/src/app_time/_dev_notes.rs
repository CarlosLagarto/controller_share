/// CtrlTime, replace the crate Chrono , having the primitives that I need
///
/// The objetive is to work with Unix TimeStamp, with nanosecs resolution, in UTC time.  (exceptions are documented)
/// I ignored the leap seconds.  If tht becomes a problem...I will see what to do
///
/// Only converted to a human readable format when necessary.
/// (show something in UI, config files)
///
/// Handles time from 1/Jan/1970 (unix epoch start) to 31/Dez/2077 (after that I don't care - this is a personal program that will have for sure a shorter timespan :-))
///
#[allow(dead_code)]
const DOC_DUMMY: u8 = 0;
