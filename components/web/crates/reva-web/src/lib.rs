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
//!
//! The exports live in reva-web-run and reva-web-io (over reva-web-state's
//! one VM); this crate is the module that re-exports them, so they are
//! linked in and exported.

pub use reva_web_io::canvas::*;
pub use reva_web_io::files::*;
pub use reva_web_io::hostcall::*;
pub use reva_web_run::boot::*;
pub use reva_web_run::info::*;
pub use reva_web_run::input::*;
pub use reva_web_run::output::*;
pub use reva_web_state::state::{BAD_IMAGE, HALTED, HOST_CALL, NEED_INPUT, NOT_BOOTED, YIELDED};
