//! Boot (from reva.f or an image), run, stop, the exit code.

use reva_vm::prelude::*;
use reva_vm::{Start, Status};
use reva_web_state::start::{begin, restart};
use reva_web_state::state::*;

/// Create the VM and boot `reva.f`.  Boot progress marks are discarded;
/// the banner, the first prompt and anything else boot printed are kept as
/// the first output.
#[unsafe(no_mangle)]
pub extern "C" fn reva_boot() -> i32 {
    let start = Start::reva();
    let vm = start
        .machine()
        .expect("booting reva.f always makes a machine");
    begin(start, vm)
}

/// Create the VM from the image in the [`reva_fs_buffer`] (`len` bytes),
/// as `reva --image` does.  Returns a status, or -2 with the reason in the
/// buffer if the bytes are not an image this core loads.
#[unsafe(no_mangle)]
pub extern "C" fn reva_boot_image(len: u32) -> i32 {
    let mut bytes = FSBUF.with(|b| std::mem::take(&mut *b.borrow_mut()));
    bytes.truncate(len as usize);
    match Start::image(bytes) {
        Ok((start, vm)) => begin(start, vm),
        Err(e) => {
            FSBUF.with(|b| *b.borrow_mut() = e.to_string().into_bytes());
            BAD_IMAGE
        }
    }
}

/// Run up to `budget` steps.
#[unsafe(no_mangle)]
pub extern "C" fn reva_run(budget: u32) -> i32 {
    with(
        |w| match w.vm.run(&mut w.host, budget as u64) {
            Status::Restart => code(restart(w)),
            s => code(s),
        },
        NOT_BOOTED,
    )
}

/// Stop the running code at the next step of [`reva_run`] (the Stop
/// button): queued input is dropped and the interpreter loop restarts with
/// `*** interrupted`, keeping the dictionary.
#[unsafe(no_mangle)]
pub extern "C" fn reva_interrupt() {
    with(|w| w.vm.interrupt(), ())
}

/// The `bye` status once [`reva_run`] has returned 2.
#[unsafe(no_mangle)]
pub extern "C" fn reva_exit_code() -> i32 {
    with(|w| w.vm.halted().unwrap_or(0), 0)
}
