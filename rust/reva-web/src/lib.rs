//! WebAssembly host for the portable Reva core.
//!
//! Deliberately free of `wasm-bindgen`: the interface is a few C-ABI
//! functions over one VM instance, so the module has **no imports** and any
//! JavaScript (browser or Node) can drive it with plain `WebAssembly`
//! calls.  See `www/reva.js` for the JavaScript side.
//!
//! Protocol (mirrors [`reva_vm::Machine`]):
//!
//! 1. `reva_boot()` — create the VM and run `reva.f`; returns a status.
//! 2. To send input: `p = reva_input_buffer(n)`, copy `n` bytes to `p`,
//!    then `reva_feed(n)`.
//! 3. `reva_run(budget)` → 0 yielded (call again soon), 1 needs input,
//!    2 halted (`reva_exit_code()`).  Keep budgets small enough that the
//!    page stays responsive; JavaScript yields to the event loop between
//!    calls.
//! 4. Output accumulates; read `reva_output_len()` bytes at
//!    `reva_output_ptr()` and then `reva_output_clear()`.

use std::cell::RefCell;

use reva_vm::{embedded, BufferHost, Machine, Status};

struct Web {
    vm: Machine,
    host: BufferHost,
    input: Vec<u8>,
}

thread_local! {
    static WEB: RefCell<Option<Web>> = const { RefCell::new(None) };
}

const YIELDED: i32 = 0;
const NEED_INPUT: i32 = 1;
const HALTED: i32 = 2;
const NOT_BOOTED: i32 = -1;

fn code(s: Status) -> i32 {
    match s {
        Status::Yielded => YIELDED,
        Status::NeedInput => NEED_INPUT,
        Status::Halted(_) => HALTED,
    }
}

fn with<R>(f: impl FnOnce(&mut Web) -> R, default: R) -> R {
    WEB.with(|w| w.borrow_mut().as_mut().map(f).unwrap_or(default))
}

/// Create the VM and boot `reva.f`.  Boot progress marks are discarded;
/// the banner, the first prompt and anything else boot printed are kept as
/// the first output.
#[no_mangle]
pub extern "C" fn reva_boot() -> i32 {
    let mut vm = Machine::new(&["reva", embedded::BOOT_PATH]);
    let mut host = BufferHost::new();
    let status = vm.run_until_blocked(&mut host);
    // Drop reva.f's progress marks; keep the banner and first prompt.
    let out = String::from_utf8_lossy(&host.out).into_owned();
    let (_, rest) = embedded::split_boot_output(&out);
    host.out = rest.as_bytes().to_vec();
    WEB.with(|w| {
        *w.borrow_mut() = Some(Web {
            vm,
            host,
            input: Vec::new(),
        })
    });
    code(status)
}

/// A buffer of `len` bytes for the caller to fill before [`reva_feed`].
#[no_mangle]
pub extern "C" fn reva_input_buffer(len: u32) -> *mut u8 {
    with(
        |w| {
            w.input.resize(len as usize, 0);
            w.input.as_mut_ptr()
        },
        std::ptr::null_mut(),
    )
}

/// Queue the first `len` bytes of the input buffer as keyboard input.
#[no_mangle]
pub extern "C" fn reva_feed(len: u32) {
    with(
        |w| {
            let n = (len as usize).min(w.input.len());
            w.vm.feed(&w.input[..n]);
        },
        (),
    )
}

/// Signal end of input (the interpreter exits once it drains).
#[no_mangle]
pub extern "C" fn reva_close_input() {
    with(|w| w.vm.close_input(), ())
}

/// Run up to `budget` steps.
#[no_mangle]
pub extern "C" fn reva_run(budget: u32) -> i32 {
    with(|w| code(w.vm.run(&mut w.host, budget as u64)), NOT_BOOTED)
}

#[no_mangle]
pub extern "C" fn reva_output_ptr() -> *const u8 {
    with(|w| w.host.out.as_ptr(), std::ptr::null())
}

#[no_mangle]
pub extern "C" fn reva_output_len() -> u32 {
    with(|w| w.host.out.len() as u32, 0)
}

#[no_mangle]
pub extern "C" fn reva_output_clear() {
    with(|w| w.host.out.clear(), ())
}

/// The `bye` status once [`reva_run`] has returned 2.
#[no_mangle]
pub extern "C" fn reva_exit_code() -> i32 {
    with(|w| w.vm.halted().unwrap_or(0), 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(s: &str) {
        let p = reva_input_buffer(s.len() as u32);
        unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), p, s.len()) };
        reva_feed(s.len() as u32);
    }

    fn output() -> String {
        let (p, n) = (reva_output_ptr(), reva_output_len());
        let s = unsafe { std::slice::from_raw_parts(p, n as usize) };
        let s = String::from_utf8_lossy(s).into_owned();
        reva_output_clear();
        s
    }

    #[test]
    fn the_c_abi_drives_a_session() {
        assert_eq!(reva_run(10), NOT_BOOTED);
        assert_eq!(reva_boot(), NEED_INPUT);
        assert_eq!(output(), "Reva 2011.2 Linux\n\nok> ");
        feed("2 3 + . : sq dup * ; 7 sq .\n");
        let mut s = YIELDED;
        while s == YIELDED {
            s = reva_run(1000);
        }
        assert_eq!(s, NEED_INPUT);
        assert_eq!(output(), "5 49 \nok> ");
        feed("3 ~sys.(bye)\n");
        while reva_run(1000) == YIELDED {}
        assert_eq!(reva_exit_code(), 3);
    }
}
