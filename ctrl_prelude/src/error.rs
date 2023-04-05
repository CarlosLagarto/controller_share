use std::fmt::Write;
use std::error::Error;

#[inline]
pub fn build_error(e: &dyn Error) -> String{
    let mut s : String = String::new();
    _ = write!(s, "error: {e}");
    let mut cause = e.source();
    while let Some(e) = cause {
        _ = write!(s, "caused by: {e}");
        cause = e.source();
    }
    s
}
