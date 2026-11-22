//! Time helpers: the pure maths lives in `dodai_ui_core::clock_math`; this adds the machine's UTC
//! offset (jiff).

pub use dodai_ui_core::clock_math::*;

/// The machine's current UTC offset in minutes.
pub fn local_utc_offset_minutes() -> i32 {
    jiff::Zoned::now().offset().seconds() / 60
}
