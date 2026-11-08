//! Time helpers: the pure maths lives in `dodai_core::clock_math`; this adds the machine's UTC
//! offset (chrono).

pub use dodai_core::clock_math::*;

/// The machine's current UTC offset in minutes.
pub fn local_utc_offset_minutes() -> i32 {
    chrono::Local::now().offset().local_minus_utc() / 60
}
