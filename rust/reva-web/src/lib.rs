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
//!
//! Files (an in-memory [`MemFs`] over the embedded `lib/`; see
//! `reva_vm::vfs`):
//!
//! * `p = reva_fs_buffer(n)`, copy a path and then the file's bytes to
//!   `p`, and `reva_fs_put(path_len)` adds the file.  This works before
//!   `reva_boot` too, so a page can restore saved files and bundle its
//!   own before `reva.f` runs.
//! * `n = reva_fs_changes()` collects every file written or removed since
//!   the last call into the buffer at `reva_fs_data()`: records of
//!   `kind:u8` (1 written, 0 removed), `path_len:u32le`, path,
//!   `data_len:u32le`, data.  `www/reva.js` keeps them in `localStorage`.

use std::cell::RefCell;

use reva_vm::vfs::MemFs;
use reva_vm::{embedded, BufferHost, Machine, Status};

struct Web {
    vm: Machine,
    host: BufferHost,
    input: Vec<u8>,
}

thread_local! {
    static WEB: RefCell<Option<Web>> = const { RefCell::new(None) };
    /// Files added before boot (moved into the host by `reva_boot`).
    static PENDING: RefCell<Option<MemFs>> = const { RefCell::new(None) };
    /// The buffer behind `reva_fs_buffer` and `reva_fs_changes`.
    static FSBUF: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
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
    if let Some(fs) = PENDING.with(|p| p.borrow_mut().take()) {
        host.fs = fs;
    }
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

/// Stop the running code at the next step of [`reva_run`] (the Stop
/// button): queued input is dropped and the interpreter loop restarts with
/// `*** interrupted`, keeping the dictionary.
#[no_mangle]
pub extern "C" fn reva_interrupt() {
    with(|w| w.vm.interrupt(), ())
}

/// The wall clock, for `time&date`, `ms@` and file times: the page sets
/// it before each [`reva_run`] (the module has no imports, so it cannot
/// read one).  Until then the clock reads as the Unix epoch.
#[no_mangle]
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

/// The `bye` status once [`reva_run`] has returned 2.
#[no_mangle]
pub extern "C" fn reva_exit_code() -> i32 {
    with(|w| w.vm.halted().unwrap_or(0), 0)
}

/// The file system: the booted host's, or the one waiting for boot.
fn with_fs<R>(f: impl FnOnce(&mut MemFs) -> R) -> R {
    let mut f = Some(f);
    let booted = WEB.with(|w| {
        w.borrow_mut()
            .as_mut()
            .map(|w| (f.take().unwrap())(&mut w.host.fs))
    });
    match booted {
        Some(r) => r,
        None => {
            PENDING.with(|p| (f.take().unwrap())(p.borrow_mut().get_or_insert_with(MemFs::new)))
        }
    }
}

/// A buffer of `len` bytes for [`reva_fs_put`].
#[no_mangle]
pub extern "C" fn reva_fs_buffer(len: u32) -> *mut u8 {
    FSBUF.with(|b| {
        let mut b = b.borrow_mut();
        b.resize(len as usize, 0);
        b.as_mut_ptr()
    })
}

/// The buffer's address (after [`reva_fs_changes`]).
#[no_mangle]
pub extern "C" fn reva_fs_data() -> *const u8 {
    FSBUF.with(|b| b.borrow().as_ptr())
}

/// Add the file in the buffer: a path of `path_len` bytes, then the
/// contents.  Returns 0, or -1 if `path_len` is too long.
#[no_mangle]
pub extern "C" fn reva_fs_put(path_len: u32) -> i32 {
    let buf = FSBUF.with(|b| std::mem::take(&mut *b.borrow_mut()));
    let n = path_len as usize;
    if n > buf.len() {
        return -1;
    }
    let path = String::from_utf8_lossy(&buf[..n]).into_owned();
    with_fs(|fs| fs.put(&path, &buf[n..]));
    0
}

/// Collect the files changed since the last call (see the module docs);
/// returns the number of bytes at [`reva_fs_data`].
#[no_mangle]
pub extern "C" fn reva_fs_changes() -> u32 {
    let changes = with_fs(MemFs::take_changes);
    FSBUF.with(|b| {
        let mut b = b.borrow_mut();
        b.clear();
        for (path, data) in changes {
            b.push(data.is_some() as u8);
            b.extend_from_slice(&(path.len() as u32).to_le_bytes());
            b.extend_from_slice(path.as_bytes());
            let data = data.unwrap_or_default();
            b.extend_from_slice(&(data.len() as u32).to_le_bytes());
            b.extend_from_slice(&data);
        }
        b.len() as u32
    })
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
        // files: one added from outside, one written by Forth
        put("/home/a.f", b": greet .\" hi\" ; ");
        assert_eq!(reva_fs_changes(), 0);
        feed("include /home/a.f greet\n\" /home/b\" creat value h \" xyz\" h write h close\n");
        while reva_run(1000) == YIELDED {}
        assert_eq!(output(), "hi\nok> \nok> ");
        let n = reva_fs_changes() as usize;
        let rec = unsafe { std::slice::from_raw_parts(reva_fs_data(), n) };
        let mut want = vec![1u8, 7, 0, 0, 0];
        want.extend_from_slice(b"/home/b");
        want.extend_from_slice(&[3, 0, 0, 0]);
        want.extend_from_slice(b"xyz");
        assert_eq!(rec, &want[..]);
        // a runaway loop, stopped
        feed(": spin repeat again ; spin\n");
        assert_eq!(reva_run(10_000), YIELDED);
        reva_interrupt();
        while reva_run(1000) == YIELDED {}
        assert!(output().contains("*** interrupted (in spin"));
        // the clock the page sets
        reva_set_time(1_792_886_400.0, 0);
        feed("time&date . . .\n");
        while reva_run(1000) == YIELDED {}
        assert_eq!(output(), "2026 10 25 \nok> ");
        feed("3 ~sys.(bye)\n");
        while reva_run(1000) == YIELDED {}
        assert_eq!(reva_exit_code(), 3);
    }

    fn put(path: &str, data: &[u8]) {
        let p = reva_fs_buffer((path.len() + data.len()) as u32);
        unsafe {
            std::ptr::copy_nonoverlapping(path.as_ptr(), p, path.len());
            std::ptr::copy_nonoverlapping(data.as_ptr(), p.add(path.len()), data.len());
        }
        assert_eq!(reva_fs_put(path.len() as u32), 0);
    }
}
