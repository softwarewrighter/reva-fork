//! The output buffer, and the clock the page sets.

use reva_web_state::state::*;

#[unsafe(no_mangle)]
pub extern "C" fn reva_output_ptr() -> *const u8 {
    with(|w| w.host.out.as_ptr(), std::ptr::null())
}

#[unsafe(no_mangle)]
pub extern "C" fn reva_output_len() -> u32 {
    with(|w| w.host.out.len() as u32, 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn reva_output_clear() {
    with(|w| w.host.out.clear(), ())
}

/// The wall clock, for `time&date`, `ms@` and file times: the page sets
/// it before each [`reva_run`] (the module has no imports, so it cannot
/// read one).  Until then the clock reads as the Unix epoch.
#[unsafe(no_mangle)]
pub extern "C" fn reva_set_time(secs: f64, micros: u32) {
    let secs = secs.max(0.0) as u64;
    with(
        |w| {
            w.host.clock = Some((secs, micros));
            w.host.fs.set_time(secs);
        },
        (),
    )
}
