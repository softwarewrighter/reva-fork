//! # The machine: vx86 CPU + memory + the Rust core routines
//!
//! ## Execution model
//!
//! The machine executes vx86 instructions one at a time.  When EIP lands in
//! the gateway region it runs the corresponding Rust routine (a [`Gate`])
//! instead of decoding bytes.  A gateway is entered exactly like a native
//! routine — by `call`, `jmp` or `ret` — and finishes with one of:
//!
//! * `Ret` — pop the return address, as the assembly routine's `ret` did;
//! * `Jump(a)` — continue at `a` (a tail jump, e.g. `mclass: jmp ebx`);
//! * `Call(a, k)` — push gateway `k` as the return address and go to `a`.
//!   This is how Rust routines call back into Forth *without* using the Rust
//!   stack: the rest of the routine is the continuation gateway `k`;
//! * `NeedInput` — stay at this gateway and return to the host until input
//!   arrives; re-entering the gateway retries.
//!
//! Because every routine on the outer-interpreter path (interpret, query,
//! eval, include, catch) is written this way, the VM can stop after any
//! instruction and resume later, which is what a browser host requires.
//! A few rarely-hooked callbacks (`do_cr`, `prompt`, `iterate`, user-defined
//! `emit`/`type`) use a nested call instead (reva-core-run's
//! `call_nested`).
//!
//! This module holds the machine's state and its types.  Its methods are
//! in [`crate::stack`], [`crate::cells`] and [`crate::vars`] (the helpers
//! every gateway uses) and, as extension traits, in reva-core-ffi,
//! reva-core-inspect, reva-core-run and reva-core-boot (Rust allows
//! `impl Machine` only in this crate; docs/design.md 3.12, 3.13).

use std::collections::{HashMap, VecDeque};

use crate::fault::Fault;
use crate::gate::Gate;
use crate::heap::Heap;
use crate::host::Host;
use crate::layout::*;
use crate::mem::Memory;
use crate::translate::Translator;
use crate::x86::Cpu;
use reva_translate::cache::CodeCache;

/// Addresses of the static variables and DEFER vectors.
///
/// Each is one cell in the statics region.  Names follow `newcore.asm`.
pub mod st {
    macro_rules! cells {
        ($($name:ident),* $(,)?) => { cells!(@ 0u32; $($name),*); };
        (@ $n:expr; $name:ident $(, $rest:ident)*) => {
            pub const $name: u32 = crate::layout::STATIC_BASE + 4 * ($n);
            cells!(@ $n + 1; $($rest),*);
        };
        (@ $n:expr;) => { pub const COUNT: u32 = $n; };
    }
    cells! {
        // variables
        LAST, FLAST, H, D, DEFAULT_CLASS, PADCHAR, BASE, OS, APPNAME, ARGC, ARGV,
        IS_COMPILING, IOR, SOURCE, TIN, STDOUT, STDIN, HINSTANCE, TP, S0V, D0V,
        H0V, TIBV, RP0V, RP0_SAVED, NOSAVEDICT, HANDLER, START_LIST, EXIT_LIST,
        SSEMI_DO_TAIL, EXTRA_SIZE, EXTRA_DATA, LASTH, LASTD, LASTL, DIDHEADER,
        // DEFER vectors: each cell holds the address the deferred word runs
        PROMPT, APPSTART, KEY, FINDINTERNAL, IS_WORD, TYPE, EMIT, DO_CR,
        EXCEPTION, CTRLC, HEAPGONE, DICTGONE, HEADER_,
        // portable OS layer (not in newcore.asm): `data: errno` points at
        // ERRNO; ENV_BUF/ENV_LEN and TM_BUF are heap buffers for `getenv`
        // and `localtime` results (0 until first use)
        ERRNO, ENV_BUF, ENV_LEN, TM_BUF,
        // where fault recovery re-enters the interpreter loop: reva.f's
        // `exception-reset`, set by `(boot-restart)` (0: the core's REPL)
        REPL_ENTRY,
        // the virtual libc's `readdir` result (a heap buffer, 0 until used)
        DIRENT_BUF,
        // what the virtual libiconv's `_libiconv_version` reads (iconv.rs)
        ICONV_VERSION,
    }
}

/// Why `run` returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// The instruction budget ran out; call `run` again to continue.
    Yielded,
    /// Forth code is waiting for input; `feed` and run again.
    NeedInput,
    /// `bye` (or end of input at the prompt).  The value is the exit code.
    Halted(i32),
    /// `restart`: the host makes a new machine the way it made this one
    /// (see `reva_vm::Start`) and carries over the unread input
    /// (`take_input`).  This machine stays stopped.
    Restart,
    /// A virtual-library function waits for the host's answer: read
    /// `pending_host_call`, then `resume` and run
    /// again (see [`crate::hostcall`]).
    HostCall,
}

/// What a gateway routine asks the dispatcher to do next.
pub enum Next {
    Ret,
    Jump(u32),
    Call(u32, Gate),
    NeedInput,
    Halt(i32),
    Restart,
    NestedReturn,
    HostCall,
}

pub enum Flow {
    Continue,
    NeedInput,
    HostCall,
    Halt(i32),
    Restart,
    NestedReturn,
}

/// How a machine runs gateway `g`: reva-words' `dispatch`, installed by
/// reva-vm's `Machine::new` and `Machine::load_image`.
pub type Dispatch = fn(&mut Machine, Gate, &mut dyn Host) -> Result<Next, Fault>;

/// What the virtual libiconv's `_libiconv_version` reads: libiconv 1.16's.
pub const ICONV_VERSION_VALUE: u32 = 0x0110;

/// A directory being read by the virtual libc's `readdir` (reva-words'
/// fileio.rs).
pub struct DirStream {
    pub path: String,
    pub names: Vec<String>,
    pub next: usize,
}

pub struct Machine {
    /// Runs gateways (see [`Dispatch`]).
    pub dispatch: Dispatch,
    pub cpu: Cpu,
    pub mem: Memory,
    pub cache: CodeCache,
    pub heap: Heap,
    pub input: VecDeque<u8>,
    pub input_closed: bool,
    pub halted: Option<i32>,
    /// Set by `restart`; like `halted`, it stops the machine for good.
    pub restart: bool,
    pub nested: u32,
    pub translator: Option<Box<dyn Translator>>,
    /// Instructions + gateway entries executed (for diagnostics/benchmarks).
    pub steps: u64,
    /// Write a line per step to stderr (see `set_trace`).
    pub trace: bool,
    /// Code-space range `[start, end)` most recently filled with alignment
    /// `nop`s; `;;` uses it to avoid reading padding as a `call` (see
    /// `Machine::ssemi`).  Transient: not part of an image.
    pub last_pad: (u32, u32),
    /// Directories open through the virtual libc's `opendir` (see
    /// `fileio.rs`).  Host state, like open files: not part of an image.
    pub dirs: Vec<Option<DirStream>>,
    /// Set by `interrupt`; taken at the next step of `run`.
    pub interrupt: bool,
    /// Native libraries `(lib)` could not serve: the last one asked for,
    /// and those already reported (see `ffi.rs`).  Transient.
    pub missing_lib: Option<String>,
    pub reported_libs: Vec<String>,
    /// The turtle and its drawing (canvas.rs, turtle.rs).  Host state, like
    /// open files: not part of an image, and new after `restart`.
    pub canvas: crate::canvas::Canvas,
    /// A host call waiting for its answer, and the answer once
    /// `resume` gave it (see [`crate::hostcall`]).  Transient.
    pub pending_call: Option<crate::hostcall::HostCall>,
    pub host_reply: Option<crate::hostcall::HostAnswer>,
    /// reva-sqlite's reply buffers by (statement, column), and the last
    /// one written (`rsql_ptr`).  Host state: not part of an image.
    pub sql_bufs: HashMap<(i64, i64), (u32, u32)>,
    pub sql_ptr: u32,
    /// The virtual libiconv's open descriptors: (from, to), `None` once
    /// closed; descriptor n is index n-1.  Host state: not part of an image.
    pub iconv_descs: Vec<Option<(crate::charset::Enc, crate::charset::Enc)>>,
}

// The statics block must fit its region (checked at compile time).
const _: () = assert!(STATIC_BASE + 4 * st::COUNT <= STATIC_END);
