//! The VM instance, the module's buffers, the status numbers.

use reva_vm::vfs::MemFs;
use reva_vm::{BufferHost, Machine, Start, Status};
use std::cell::RefCell;

/// The one VM instance and what goes with it.
pub struct Web {
    pub vm: Machine,
    pub host: BufferHost,
    pub input: Vec<u8>,
    /// Start input (`reva_feed_start`), fed again after each `restart`.
    pub replay: Vec<u8>,
    /// How `vm` was made, for `restart`.
    pub start: Start,
    /// `restart` made a new machine (see `reva_canvas_reset`).
    pub canvas_reset: bool,
}

thread_local! {
    pub static WEB: RefCell<Option<Web>> = const { RefCell::new(None) };
    /// Files added before boot (moved into the host by `reva_boot`).
    pub static PENDING: RefCell<Option<MemFs>> = const { RefCell::new(None) };
    /// The buffer behind `reva_fs_buffer` and `reva_fs_changes`.
    pub static FSBUF: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    /// The buffer behind `reva_canvas_take`.
    pub static CANVAS: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
}

pub const YIELDED: i32 = 0;
pub const NEED_INPUT: i32 = 1;
pub const HALTED: i32 = 2;
/// The VM waits for a host call's answer, which the page gives (see
/// hostcall.rs; docs/design.md 3.11).
pub const HOST_CALL: i32 = 3;
pub const NOT_BOOTED: i32 = -1;
pub const BAD_IMAGE: i32 = -2;

/// A status as the number `reva_run` returns.
pub fn code(s: Status) -> i32 {
    match s {
        Status::Yielded => YIELDED,
        Status::NeedInput => NEED_INPUT,
        Status::Halted(_) => HALTED,
        Status::Restart => YIELDED,
        Status::HostCall => HOST_CALL,
    }
}

/// Run `f` on the booted VM, or give `default` before boot.
pub fn with<R>(f: impl FnOnce(&mut Web) -> R, default: R) -> R {
    WEB.with(|w| w.borrow_mut().as_mut().map(f).unwrap_or(default))
}
