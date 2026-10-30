//! # The address map
//!
//! Reva was written for a flat 32-bit address space and its compiled code
//! embeds absolute and relative 32-bit addresses.  The portable core keeps
//! that model unchanged: the VM owns one linear block of memory, and *every
//! address Forth code sees is an offset into that block*.  This is exactly the
//! `wasm32` memory model, and on 64-bit hosts it costs one base register.
//!
//! The regions below mirror the `DATASECTION`/`BSSSECTION` of
//! `src/newcore.asm`.  Sizes follow the original where they matter to Forth
//! code (TIB, parse buffer, stack sizes) and are rounded for alignment
//! elsewhere.
//!
//! ```text
//!  0x0000_0000  null guard          any access faults (catches 0 @)
//!  0x0000_1000  gateway "ROM"       one 4-byte slot per Rust routine
//!  0x0000_2000  statics             variables and DEFER vectors
//!  0x0000_3000  number pad          (.r) builds digits downward from NPAD
//!  0x0000_3400  TIB                 terminal input buffer, 1 KiB
//!  0x0000_3800  app_file_name       256 bytes
//!  0x0000_3900  fzt_buf             512 bytes (zero-terminated temp)
//!  0x0000_4000  parse_buf           128 KiB (parse/ escape buffer)
//!  0x0002_4000  data stack          grows down from S0, 2 Ki cells
//!  0x0002_7000  return stack        grows down from RP0, 64 KiB
//!  0x0003_7000  argv                argv pointers and strings, 4 KiB
//!  0x0004_0000  dictionary (D0)     1 MiB of headers
//!  0x0014_0000  code (H0)           2 MiB of IA-32-subset code
//!  0x0034_0000  heap                allocate/free/slurp, up to MEM_SIZE
//! ```

/// Accesses below this address fault, so a stray `0 @` is reported rather
/// than silently reading garbage.
pub const NULL_GUARD: u32 = 0x0000_1000;

/// Gateway slots: executing at `GATE_BASE + 4*n` runs Rust routine `n`.
pub const GATE_BASE: u32 = 0x0000_1000;
pub const GATE_END: u32 = 0x0000_2000;
pub const GATE_STRIDE: u32 = 4;

/// Static variables and DEFER vectors (see reva-vm's `machine::st`).
pub const STATIC_BASE: u32 = 0x0000_2000;
pub const STATIC_END: u32 = 0x0000_3000;

/// `(.r)` writes digits (and up to 255 pad characters) downward from here.
pub const NPAD: u32 = 0x0000_3200;

pub const TIB: u32 = 0x0000_3400;
pub const TIB_SIZE: u32 = 1024;
pub const TIB_TOP: u32 = TIB + TIB_SIZE;

pub const APP_FILE_NAME: u32 = 0x0000_3800;
pub const FZT_BUF: u32 = 0x0000_3900;
pub const FZT_BUF_SIZE: u32 = 512;

pub const PARSE_BUF: u32 = 0x0000_4000;
pub const PARSE_BUF_SIZE: u32 = 128 * 1024;

/// Data stack: 2 Ki cells growing down from `S0`, 16 cells of overflow
/// slack below, and ~1000 cells of underflow slack above (Reva had 256),
/// so a few extra drops never touch the return stack.  The interpreter
/// checks `ESI <= S0` between words and reports underflow.
pub const DSTACK_LO: u32 = 0x0002_4000;
pub const S0: u32 = DSTACK_LO + 64 + 2048 * 4;
pub const DSTACK_HI: u32 = 0x0002_7000;

pub const RSTACK_LO: u32 = 0x0002_7000;
pub const RP0: u32 = 0x0003_7000;

pub const ARGV_BASE: u32 = 0x0003_7000;
pub const ARGV_END: u32 = 0x0003_8000;

pub const D0: u32 = 0x0004_0000;
pub const DTOP: u32 = 0x0014_0000;

pub const H0: u32 = 0x0014_0000;
pub const HTOP: u32 = 0x0034_0000;

pub const HEAP_BASE: u32 = 0x0034_0000;

/// Default size of VM memory: 16 MiB, leaving ~12.7 MiB of heap.
pub const MEM_SIZE: u32 = 0x0100_0000;

/// The value of Reva's `os` constant.  Forth code in `reva.f` selects
/// libraries by it; the VM reports Linux (1) until a portable OS layer
/// exists.  See docs/design.md, "OS identity".
pub const OS_ID: u32 = 1;

/// Key code that `query` treats as end of input (Ctrl-D, as in linux.asm).
pub const EOF_KEY: u32 = 4;

// Region invariants, checked at compile time.
const _: () = assert!(FZT_BUF + FZT_BUF_SIZE <= PARSE_BUF);
const _: () = assert!(PARSE_BUF + PARSE_BUF_SIZE <= DSTACK_LO);
const _: () = assert!(NPAD - 256 - 32 >= STATIC_END);
