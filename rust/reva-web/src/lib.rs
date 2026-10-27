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
//!    Or start from a saved image: copy its bytes to
//!    `reva_fs_buffer(n)` and call `reva_boot_image(n)`; it returns -2 if
//!    the bytes are not an image this core loads (the reason is the
//!    `reva_fs_len()` bytes at `reva_fs_data()`).
//! 2. To send input: `p = reva_input_buffer(n)`, copy `n` bytes to `p`,
//!    then `reva_feed(n)`.  `reva_feed_start(n)` instead marks the bytes as
//!    *start input* (a page's start files): fed now, and again after every
//!    `restart`, as the CLI re-runs its file arguments.
//! 3. `reva_run(budget)` → 0 yielded (call again soon), 1 needs input,
//!    2 halted (`reva_exit_code()`).  Keep budgets small enough that the
//!    page stays responsive; JavaScript yields to the event loop between
//!    calls.  `restart` is handled inside `reva_run`: the VM is made again
//!    the way it was first made (reva.f or the image), keeps its files,
//!    reads the start input again, then the input not yet read, and prints
//!    its banner again.
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
//!
//! Build information (the page's footer): `reva_build_info_len()` bytes
//! at `reva_build_info_ptr()`, `key\tvalue` lines from
//! `reva_vm::version::fields()`: name, version, copyright, license,
//! repository, build_host, build_commit and build_time.  Works before boot.
//!
//! The canvas (turtle graphics; see `reva_vm::canvas` for the op format):
//!
//! * `n = reva_canvas_take()` moves the drawing ops not yet taken (at most
//!   `reva_vm::canvas::MAX_OPS_PER_TAKE` records) into a buffer of `n`
//!   `f32`s at `reva_canvas_ptr()`; call it after each [`reva_run`] slice
//!   until it returns 0.
//! * `reva_canvas_resend()`: the next takes start over with the whole
//!   drawing (a window opened).
//! * `reva_canvas_reset()` returns 1 once after `restart` made a new
//!   machine (its drawing is empty: windows should clear), else 0.

use std::cell::RefCell;

use reva_vm::vfs::MemFs;
use reva_vm::{BufferHost, Machine, Start, Status, embedded};

struct Web {
    vm: Machine,
    host: BufferHost,
    input: Vec<u8>,
    /// Start input ([`reva_feed_start`]), fed again after each `restart`.
    replay: Vec<u8>,
    /// How `vm` was made, for `restart`.
    start: Start,
    /// `restart` made a new machine (see [`reva_canvas_reset`]).
    canvas_reset: bool,
}

thread_local! {
    static WEB: RefCell<Option<Web>> = const { RefCell::new(None) };
    /// Files added before boot (moved into the host by `reva_boot`).
    static PENDING: RefCell<Option<MemFs>> = const { RefCell::new(None) };
    /// The buffer behind `reva_fs_buffer` and `reva_fs_changes`.
    static FSBUF: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    /// The buffer behind `reva_canvas_take`.
    static CANVAS: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
}

const YIELDED: i32 = 0;
const NEED_INPUT: i32 = 1;
const HALTED: i32 = 2;
/// The VM waits for a host call's answer (rust/DESIGN.md 3.11).  This
/// module's host has no services yet, so it does not happen here; the
/// desktop's bridge is saga step local-sqlite-desktop.
const HOST_CALL: i32 = 3;
const NOT_BOOTED: i32 = -1;
const BAD_IMAGE: i32 = -2;

fn code(s: Status) -> i32 {
    match s {
        Status::Yielded => YIELDED,
        Status::NeedInput => NEED_INPUT,
        Status::Halted(_) => HALTED,
        Status::Restart => YIELDED,
        Status::HostCall => HOST_CALL,
    }
}

fn with<R>(f: impl FnOnce(&mut Web) -> R, default: R) -> R {
    WEB.with(|w| w.borrow_mut().as_mut().map(f).unwrap_or(default))
}

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

fn begin(start: Start, mut vm: Machine) -> i32 {
    let mut host = BufferHost::new();
    // The page renders ANSI colour escapes (www/ansi.js), so `pp` colours.
    host.color = true;
    if let Some(fs) = PENDING.with(|p| p.borrow_mut().take()) {
        host.fs = fs;
    }
    let status = startup(&mut vm, &mut host);
    WEB.with(|w| {
        *w.borrow_mut() = Some(Web {
            vm,
            host,
            input: Vec::new(),
            replay: Vec::new(),
            start,
            canvas_reset: false,
        })
    });
    code(status)
}

/// Run a new machine until it first waits, keeping what it printed except
/// reva.f's progress marks.
fn startup(vm: &mut Machine, host: &mut BufferHost) -> Status {
    let before = host.out.len();
    let status = vm.run_until_blocked(host);
    let out = String::from_utf8_lossy(&host.out[before..]).into_owned();
    let (_, rest) = embedded::split_boot_output(&out);
    host.out.truncate(before);
    host.out.extend_from_slice(rest.as_bytes());
    status
}

/// `restart`: a new machine made as the first one was, with the old one's
/// files and output, the start input again, then the unread input.
fn restart(w: &mut Web) -> Status {
    let (input, closed) = w.vm.take_input();
    // The same start made the first machine (an image was checked then).
    w.vm = w.start.machine().expect("a start that worked once");
    w.canvas_reset = true;
    match startup(&mut w.vm, &mut w.host) {
        Status::NeedInput => {}
        s => return s,
    }
    w.vm.feed(&w.replay);
    w.vm.feed(&input);
    if closed {
        w.vm.close_input();
    }
    Status::Yielded
}

/// A buffer of `len` bytes for the caller to fill before [`reva_feed`].
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
pub extern "C" fn reva_feed(len: u32) {
    with(
        |w| {
            let n = (len as usize).min(w.input.len());
            w.vm.feed(&w.input[..n]);
        },
        (),
    )
}

/// Like [`reva_feed`], and keep the bytes to feed again after `restart`.
#[unsafe(no_mangle)]
pub extern "C" fn reva_feed_start(len: u32) {
    with(
        |w| {
            let n = (len as usize).min(w.input.len());
            w.vm.feed(&w.input[..n]);
            w.replay.extend_from_slice(&w.input[..n]);
        },
        (),
    )
}

/// Signal end of input (the interpreter exits once it drains).
#[unsafe(no_mangle)]
pub extern "C" fn reva_close_input() {
    with(|w| w.vm.close_input(), ())
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

/// Stop the running code at the next step of [`reva_run`] (the Stop
/// button): queued input is dropped and the interpreter loop restarts with
/// `*** interrupted`, keeping the dictionary.
#[unsafe(no_mangle)]
pub extern "C" fn reva_interrupt() {
    with(|w| w.vm.interrupt(), ())
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

/// The `bye` status once [`reva_run`] has returned 2.
#[unsafe(no_mangle)]
pub extern "C" fn reva_exit_code() -> i32 {
    with(|w| w.vm.halted().unwrap_or(0), 0)
}

/// This build's facts as `key\tvalue` lines (see the module docs).
fn build_info() -> &'static str {
    static INFO: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    INFO.get_or_init(reva_vm::version::fields)
}

/// Where [`build_info`]'s text starts.
#[unsafe(no_mangle)]
pub extern "C" fn reva_build_info_ptr() -> *const u8 {
    build_info().as_ptr()
}

/// The length of [`build_info`]'s text in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn reva_build_info_len() -> u32 {
    build_info().len() as u32
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
#[unsafe(no_mangle)]
pub extern "C" fn reva_fs_buffer(len: u32) -> *mut u8 {
    FSBUF.with(|b| {
        let mut b = b.borrow_mut();
        b.resize(len as usize, 0);
        b.as_mut_ptr()
    })
}

/// The buffer's address (after [`reva_fs_changes`]).
#[unsafe(no_mangle)]
pub extern "C" fn reva_fs_data() -> *const u8 {
    FSBUF.with(|b| b.borrow().as_ptr())
}

/// How many bytes [`reva_fs_data`] holds (after [`reva_boot_image`]
/// failed, the reason).
#[unsafe(no_mangle)]
pub extern "C" fn reva_fs_len() -> u32 {
    FSBUF.with(|b| b.borrow().len() as u32)
}

/// Add the file in the buffer: a path of `path_len` bytes, then the
/// contents.  Returns 0, or -1 if `path_len` is too long.
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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

/// Take the drawing ops not yet taken into the canvas buffer; returns how
/// many `f32`s it holds (0: nothing new).
#[unsafe(no_mangle)]
pub extern "C" fn reva_canvas_take() -> u32 {
    let ops = with(|w| w.vm.canvas().take_ops(), Vec::new());
    let n = ops.len() as u32;
    CANVAS.with(|c| *c.borrow_mut() = ops);
    n
}

/// The canvas buffer's address (after [`reva_canvas_take`]).
#[unsafe(no_mangle)]
pub extern "C" fn reva_canvas_ptr() -> *const f32 {
    CANVAS.with(|c| c.borrow().as_ptr())
}

/// Hand out the whole drawing again from the next [`reva_canvas_take`].
#[unsafe(no_mangle)]
pub extern "C" fn reva_canvas_resend() {
    with(|w| w.vm.canvas().resend(), ())
}

/// 1 once after `restart` replaced the machine (and its drawing), else 0.
#[unsafe(no_mangle)]
pub extern "C" fn reva_canvas_reset() -> i32 {
    with(|w| std::mem::take(&mut w.canvas_reset) as i32, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reva_vm::version::BANNER;

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
        assert_eq!(output(), format!("{BANNER}\n\nok> "));
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

    fn run_out() -> String {
        while reva_run(1000) == YIELDED {}
        output()
    }

    #[test]
    fn images_and_restart() {
        // each test runs on its own thread, so this is a fresh module state
        assert_eq!(reva_boot(), NEED_INPUT);
        output();
        feed(": hi .\" from the image\" ; save /app.img\n");
        assert!(run_out().contains("Saving: /app.img"));
        let n = reva_fs_changes() as usize;
        let rec = unsafe { std::slice::from_raw_parts(reva_fs_data(), n) }.to_vec();
        let image = &rec[5 + 8 + 4..];
        // a fresh start from the image
        let p = reva_fs_buffer(image.len() as u32);
        unsafe { std::ptr::copy_nonoverlapping(image.as_ptr(), p, image.len()) };
        assert_eq!(reva_boot_image(image.len() as u32), NEED_INPUT);
        assert_eq!(output(), format!("{BANNER}\n\nok> "));
        feed(": gone ; hi restart\nhi gone\n");
        // restart: the banner again, the image's words, not later ones;
        // the next line was not read yet, so it reaches the new machine
        let out = run_out();
        assert!(
            out.starts_with(&format!("from the image{BANNER}\n\nok> from the image")),
            "{out:?}"
        );
        assert!(out.contains("unknown word:  gone"), "{out:?}");
        // not an image
        let p = reva_fs_buffer(4);
        unsafe { std::ptr::copy_nonoverlapping(b"nope".as_ptr(), p, 4) };
        assert_eq!(reva_boot_image(4), BAD_IMAGE);
        let why = unsafe { std::slice::from_raw_parts(reva_fs_data(), reva_fs_len() as usize) };
        assert_eq!(why, b"image is truncated");
    }

    #[test]
    fn restart_reads_the_start_input_again() {
        assert_eq!(reva_boot(), NEED_INPUT);
        output();
        put("/app.f", b": hello .\" hello\" ; ");
        let start = "include /app.f\n";
        let p = reva_input_buffer(start.len() as u32);
        unsafe { std::ptr::copy_nonoverlapping(start.as_ptr(), p, start.len()) };
        reva_feed_start(start.len() as u32);
        feed(": gone ; restart\nhello gone\n");
        let out = run_out();
        assert!(out.contains("ok> hello"), "{out:?}");
        assert!(out.contains("unknown word:  gone"), "{out:?}");
    }

    fn canvas() -> Vec<f32> {
        let n = reva_canvas_take() as usize;
        unsafe { std::slice::from_raw_parts(reva_canvas_ptr(), n) }.to_vec()
    }

    #[test]
    fn turtle_ops_reach_the_page() {
        assert_eq!(reva_boot(), NEED_INPUT);
        output();
        assert!(canvas().is_empty());
        feed("needs turtle/core : sq 4 0do 100 forward 90 right loop ; clear sq\n");
        run_out();
        let ops = canvas();
        let lines = ops.chunks(8).filter(|r| r[0] == 2.0).count();
        assert_eq!((ops[0], lines), (1.0, 4));
        assert!(canvas().is_empty());
        reva_canvas_resend();
        assert_eq!(canvas().len(), ops.len());
        assert_eq!(reva_canvas_reset(), 0);
        feed("restart\n");
        run_out();
        assert_eq!(reva_canvas_reset(), 1);
        assert_eq!(reva_canvas_reset(), 0);
        assert!(canvas().is_empty());
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
