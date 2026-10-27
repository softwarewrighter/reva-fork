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
//! `emit`/`type`) use [`Machine::call_nested`] instead; see its notes.

use std::collections::{HashMap, VecDeque};
use std::hash::{BuildHasherDefault, Hasher};

use crate::fault::Fault;
use crate::gates::Gate;
use crate::heap::Heap;
use crate::host::Host;
use crate::layout::*;
use crate::mem::{Memory, PAGE_SIZE};
use crate::translate::Translator;
use crate::x86::{Cpu, EAX, ESI, ESP, Insn, decode};

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
    }
}

/// Why [`Machine::run`] returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// The instruction budget ran out; call `run` again to continue.
    Yielded,
    /// Forth code is waiting for input; [`Machine::feed`] and run again.
    NeedInput,
    /// `bye` (or end of input at the prompt).  The value is the exit code.
    Halted(i32),
    /// `restart`: the host makes a new machine the way it made this one
    /// (see [`crate::Start`]) and carries over the unread input
    /// ([`Machine::take_input`]).  This machine stays stopped.
    Restart,
    /// A virtual-library function waits for the host's answer: read
    /// [`Machine::pending_host_call`], then [`Machine::resume`] and run
    /// again (see [`crate::hostcall`]).
    HostCall,
}

/// What a gateway routine asks the dispatcher to do next.
pub(crate) enum Next {
    Ret,
    Jump(u32),
    Call(u32, Gate),
    NeedInput,
    Halt(i32),
    Restart,
    NestedReturn,
    HostCall,
}

pub(crate) enum Flow {
    Continue,
    NeedInput,
    HostCall,
    Halt(i32),
    Restart,
    NestedReturn,
}

/// Fast hasher for `u32` code addresses.
#[derive(Default)]
pub(crate) struct AddrHasher(u64);

impl Hasher for AddrHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ b as u64).wrapping_mul(0x100_0000_01b3);
        }
    }
    fn write_u32(&mut self, n: u32) {
        self.0 = (n as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

pub(crate) type CodeCache = HashMap<u32, Insn, BuildHasherDefault<AddrHasher>>;

pub struct Machine {
    pub(crate) cpu: Cpu,
    pub(crate) mem: Memory,
    pub(crate) cache: CodeCache,
    pub(crate) heap: Heap,
    pub(crate) input: VecDeque<u8>,
    pub(crate) input_closed: bool,
    pub(crate) halted: Option<i32>,
    /// Set by `restart`; like `halted`, it stops the machine for good.
    pub(crate) restart: bool,
    pub(crate) nested: u32,
    translator: Option<Box<dyn Translator>>,
    /// Instructions + gateway entries executed (for diagnostics/benchmarks).
    pub steps: u64,
    /// Write a line per step to stderr (see [`Machine::set_trace`]).
    trace: bool,
    /// Code-space range `[start, end)` most recently filled with alignment
    /// `nop`s; `;;` uses it to avoid reading padding as a `call` (see
    /// `Machine::ssemi`).  Transient: not part of an image.
    pub(crate) last_pad: (u32, u32),
    /// Directories open through the virtual libc's `opendir` (see
    /// `fileio.rs`).  Host state, like open files: not part of an image.
    pub(crate) dirs: Vec<Option<crate::fileio::DirStream>>,
    /// Set by [`Machine::interrupt`]; taken at the next step of [`Machine::run`].
    interrupt: bool,
    /// Native libraries `(lib)` could not serve: the last one asked for,
    /// and those already reported (see `ffi.rs`).  Transient.
    pub(crate) missing_lib: Option<String>,
    pub(crate) reported_libs: Vec<String>,
    /// The turtle and its drawing (canvas.rs, turtle.rs).  Host state, like
    /// open files: not part of an image, and new after `restart`.
    pub(crate) canvas: crate::canvas::Canvas,
    /// A host call waiting for its answer, and the answer once
    /// [`Machine::resume`] gave it (see [`crate::hostcall`]).  Transient.
    pub(crate) pending_call: Option<crate::hostcall::HostCall>,
    pub(crate) host_reply: Option<crate::hostcall::HostAnswer>,
    /// reva-sqlite's reply buffers by (statement, column), and the last
    /// one written (`rsql_ptr`).  Host state: not part of an image.
    pub(crate) sql_bufs: HashMap<(i64, i64), (u32, u32)>,
    pub(crate) sql_ptr: u32,
}

impl Machine {
    /// A fresh machine.  `args` become `argc`/`(argv)`; as in Reva's core,
    /// `args[1]` (if present) is the file `appstart` includes before the
    /// interactive loop.  [`crate::embedded::BOOT_PATH`] boots `reva.f`.
    pub fn new(args: &[&str]) -> Machine {
        let mut m = Machine::with_memory(MEM_SIZE);
        m.init_statics();
        m.build_dictionary();
        m.set_args(args);
        m.reset_cpu();
        m
    }

    pub(crate) fn with_memory(mem_size: u32) -> Machine {
        let mut mem = Memory::new(mem_size);
        // Gateway ROM reads as `hlt`, so a disassembler shows something sane.
        mem.fill(GATE_BASE, GATE_END - GATE_BASE, 0xf4)
            .expect("gateway region");
        Machine {
            cpu: Cpu::default(),
            mem,
            cache: CodeCache::default(),
            heap: Heap::new(HEAP_BASE, mem_size),
            input: VecDeque::new(),
            input_closed: false,
            halted: None,
            restart: false,
            nested: 0,
            translator: None,
            steps: 0,
            trace: false,
            last_pad: (0, 0),
            dirs: Vec::new(),
            interrupt: false,
            missing_lib: None,
            reported_libs: Vec::new(),
            canvas: crate::canvas::Canvas::new(),
            pending_call: None,
            host_reply: None,
            sql_bufs: HashMap::new(),
            sql_ptr: 0,
        }
    }

    pub(crate) fn reset_cpu(&mut self) {
        self.cpu = Cpu {
            esp_floor: RSTACK_LO,
            ..Cpu::default()
        };
        self.cpu.r[ESI as usize] = S0;
        self.cpu.r[ESP as usize] = RP0;
        self.cpu.eip = Gate::Cold.addr();
    }

    fn init_statics(&mut self) {
        use st::*;
        let g = |g: Gate| g.addr();
        let init: &[(u32, u32)] = &[
            (LAST, FLAST),
            (H, H0),
            (D, D0),
            (DEFAULT_CLASS, g(Gate::FClass)),
            (PADCHAR, 32),
            (BASE, 10),
            (OS, OS_ID),
            (APPNAME, APP_FILE_NAME),
            (STDOUT, 1),
            (TP, TIB),
            (TIN, TIB),
            (S0V, S0),
            (D0V, D0),
            (H0V, H0),
            (TIBV, TIB),
            (RP0V, RP0_SAVED),
            (RP0_SAVED, RP0),
            (SSEMI_DO_TAIL, 0x90),
            (PROMPT, g(Gate::Noop)),
            (APPSTART, g(Gate::Hello)),
            (KEY, g(Gate::OsKey)),
            (FINDINTERNAL, g(Gate::Find)),
            (IS_WORD, g(Gate::WordNotFound)),
            (TYPE, g(Gate::OsType)),
            (EMIT, g(Gate::OsEmit)),
            (DO_CR, g(Gate::Noop)),
            (EXCEPTION, g(Gate::OsBye)),
            (CTRLC, g(Gate::Noop)),
            (HEAPGONE, g(Gate::OsBye)),
            (DICTGONE, g(Gate::OsBye)),
            (HEADER_, g(Gate::HeaderDefault)),
        ];
        for &(a, v) in init {
            self.mem.write_u32(a, v).expect("statics");
        }
        self.mem
            .write_bytes(APP_FILE_NAME, crate::embedded::APP_PATH.as_bytes())
            .expect("appname");
        self.mem
            .write_u8(APP_FILE_NAME + crate::embedded::APP_PATH.len() as u32, 0)
            .expect("appname");
    }

    /// Write `argc`/`(argv)`: a pointer array followed by the strings.
    pub(crate) fn set_args(&mut self, args: &[&str]) {
        let mut strings = ARGV_BASE + 4 * (args.len() as u32 + 1);
        for (i, a) in args.iter().enumerate() {
            let bytes = a.as_bytes();
            if strings + bytes.len() as u32 + 1 > ARGV_END {
                break;
            }
            self.mem
                .write_u32(ARGV_BASE + 4 * i as u32, strings)
                .expect("argv");
            self.mem.write_bytes(strings, bytes).expect("argv");
            self.mem
                .write_u8(strings + bytes.len() as u32, 0)
                .expect("argv");
            strings += bytes.len() as u32 + 1;
        }
        self.mem
            .write_u32(ARGV_BASE + 4 * args.len() as u32, 0)
            .expect("argv");
        self.mem
            .write_u32(st::ARGC, args.len() as u32)
            .expect("argc");
        self.mem.write_u32(st::ARGV, ARGV_BASE).expect("argv");
    }

    /// Create the core dictionary, in the same order as `newcore.asm`'s
    /// `DICT` list (so the most recent core word is `getextra`).
    fn build_dictionary(&mut self) {
        use Gate::*;
        use st::*;
        let (fc, mc, vc, cc, valc, dc) = (
            FClass.addr(),
            MClass.addr(),
            VClass.addr(),
            CClass.addr(),
            ValClass.addr(),
            DClass.addr(),
        );
        let words: &[(&str, u32, u32)] = &[
            ("'", Tick.addr(), fc),
            ("makeexe", MakeExe.addr(), fc),
            ("exception", EXCEPTION, dc),
            ("heapgone", HEAPGONE, dc),
            ("dictgone", DICTGONE, dc),
            ("ctrl-c", CTRLC, dc),
            ("do_cr", DO_CR, dc),
            ("s0", S0V, cc),
            ("d0", D0V, vc),
            ("h0", H0V, vc),
            ("ioerr", IOR, vc),
            ("rp0", RP0V, cc),
            ("padchar", PADCHAR, vc),
            (">in", TIN, vc),
            ("tib", TIBV, vc),
            ("tp", TP, vc),
            ("src", SOURCE, vc),
            ("state", IS_COMPILING, vc),
            ("dict", D, vc),
            ("(here)", H, vc),
            ("last", LAST, cc),
            ("base", BASE, vc),
            ("(argv)", ARGV, valc),
            ("argc", ARGC, valc),
            ("hinst", HINSTANCE, valc),
            ("stdout", STDOUT, valc),
            ("stdin", STDIN, valc),
            ("os", OS, cc),
            ("syscall", Syscall.addr(), fc),
            ("nosavedict", NOSAVEDICT, vc),
            ("cold", Cold.addr(), fc),
            ("reset", Reset.addr(), fc),
            ("prompt", PROMPT, dc),
            ("default_class", DEFAULT_CLASS, vc),
            (">lz", LzPack.addr(), fc),
            ("lz>", LzDepack.addr(), fc),
            ("lzmax", LzMax.addr(), fc),
            ("'variable", VClass.addr(), fc),
            ("'constant", CClass.addr(), fc),
            ("'does", DoesClass.addr(), fc),
            ("'value", ValClass.addr(), fc),
            ("'forth", FClass.addr(), fc),
            ("'notail", FClassNotail.addr(), fc),
            ("'macro", MClass.addr(), fc),
            ("'inline", IClass.addr(), fc),
            ("'macront", MClassNotail.addr(), fc),
            ("'defer", DClass.addr(), fc),
            ("catch", Catch.addr(), fc),
            ("throw", Throw.addr(), fc),
            ("onexit", OnExit.addr(), fc),
            ("onstartup", OnStartup.addr(), fc),
            ("(lib)", LoadLib.addr(), fc),
            ("(-lib)", UnloadLib.addr(), fc),
            ("(call)", CallFn.addr(), fc),
            ("(func)", OsFunc.addr(), fc),
            ("fnvhash", FnvHash.addr(), fc),
            ("(bye)", Bye.addr(), fc),
            ("cmp", Cmp.addr(), fc),
            ("cmpi", Cmpi.addr(), fc),
            ("place", Place.addr(), fc),
            ("+place", PlusPlace.addr(), fc),
            ("move", Move.addr(), fc),
            ("fill", Fill.addr(), fc),
            ("zt", Zt.addr(), fc),
            ("zcount", Zcount.addr(), fc),
            ("open/r", OpenR.addr(), fc),
            ("open/rw", OpenRw.addr(), fc),
            ("creat", Creat.addr(), fc),
            ("close", Close.addr(), fc),
            ("read", Read.addr(), fc),
            ("write", Write.addr(), fc),
            ("fsize", Fsize.addr(), fc),
            ("allocate", Allocate.addr(), fc),
            ("free", Free.addr(), fc),
            ("resize", Resize.addr(), fc),
            ("header", Header.addr(), fc),
            ("(header)", HEADER_, dc),
            ("literal", Literal.addr(), mc),
            (";;", SSemi.addr(), mc),
            (";inline", InlineSemi.addr(), mc),
            ("parsews", Parsews.addr(), fc),
            ("align", Align.addr(), fc),
            ("here,", StoreHere.addr(), fc),
            ("compile", Compile.addr(), fc),
            ("interp", Interpret.addr(), fc),
            ("key", KEY, dc),
            ("appstart", APPSTART, dc),
            ("emit", EMIT, dc),
            ("find-dict", FINDINTERNAL, dc),
            (">single", Single.addr(), fc),
            (">double", Double.addr(), fc),
            ("word?", IS_WORD, dc),
            ("type", TYPE, dc),
            ("digit>", FromDigit.addr(), fc),
            (">digit", ToDigit.addr(), fc),
            (":", Colon.addr(), mc),
            (";", Semi.addr(), mc),
            ("[", LBracket.addr(), mc),
            ("]", RBracket.addr(), mc),
            ("2dup", TwoDup.addr(), fc),
            ("variable", Variable.addr(), fc),
            ("variable,", Variable2.addr(), fc),
            (",", Comma.addr(), fc),
            ("3,", Comma3.addr(), fc),
            ("2,", Comma2.addr(), fc),
            ("1,", Comma1.addr(), fc),
            ("include", Include.addr(), fc),
            ("(include)", Include2.addr(), fc),
            ("eval", Eval.addr(), fc),
            ("parse", Parse.addr(), fc),
            ("parse/", ParseEsc.addr(), fc),
            ("appname", APPNAME, cc),
            ("(save)", Save.addr(), fc),
            ("(.r)", PrintR.addr(), fc),
            ("link", Link.addr(), fc),
            ("-link", Link2.addr(), fc),
            ("iterate", Iterate.addr(), fc),
            ("slurp", Slurp.addr(), fc),
            ("setextra", SetExtra.addr(), fc),
            ("getextra", GetExtra.addr(), fc),
            ("(boot-restart)", BootRestart.addr(), fc),
            ("restart", Restart.addr(), fc),
            ("version", Version.addr(), fc),
        ];
        let mut d = D0;
        let mut prev = 0;
        for &(name, xt, class) in words {
            let entry = d + 4;
            let n = name.len() as u32;
            let w = &mut self.mem;
            w.write_u32(d, class).expect("dict");
            w.write_u32(entry, prev).expect("dict");
            w.write_u32(entry + 4, xt).expect("dict");
            w.write_u8(entry + 8, n as u8).expect("dict");
            w.write_bytes(entry + 9, name.as_bytes()).expect("dict");
            prev = entry;
            d = (entry + 9 + n + 3) & !3;
        }
        self.mem.write_u32(FLAST, prev).expect("flast");
        self.mem.write_u32(D, d).expect("d");
    }

    // ---- host-facing API ---------------------------------------------

    /// Queue input bytes (keyboard data, pasted text, piped stdin).
    pub fn feed(&mut self, bytes: &[u8]) {
        self.input.extend(bytes);
    }

    /// Turn off the interactive prompt (`reva.f` prints `ok> ` before each
    /// line it reads).  For batch use: piped input, tests, embedding.
    pub fn quiet_prompt(&mut self) {
        self.mem
            .write_u32(st::PROMPT, Gate::Noop.addr())
            .expect("prompt");
    }

    /// Take the input not yet read, and whether input was closed, for the
    /// machine that replaces this one after [`Status::Restart`].  The rest
    /// of the line being interpreted is already in the VM and is dropped.
    pub fn take_input(&mut self) -> (Vec<u8>, bool) {
        (self.input.drain(..).collect(), self.input_closed)
    }

    /// Mark the end of input: once the queue drains, `key` returns EOF and
    /// the interpreter exits as Reva did on Ctrl-D.
    pub fn close_input(&mut self) {
        self.input_closed = true;
    }

    /// Stop the running code (Ctrl-C, a Stop button): at its next step,
    /// [`Machine::run`] drops queued input and recovers as from a fault
    /// (`*** interrupted (in WORD ...)`), back to the interpreter loop with
    /// the dictionary kept and a half-made definition removed.
    ///
    /// The host calls this between `run` calls, so code inside a nested
    /// call (which runs to completion within one step) cannot be stopped
    /// this way; a host that must always regain control runs the VM where
    /// it can discard it (the browser's worker: `terminate()`).
    pub fn interrupt(&mut self) {
        self.interrupt = true;
    }

    /// Run up to `budget` steps (one instruction or one gateway each).
    pub fn run(&mut self, host: &mut dyn Host, budget: u64) -> Status {
        for _ in 0..budget {
            if let Some(code) = self.halted {
                return Status::Halted(code);
            }
            if self.restart {
                return Status::Restart;
            }
            if self.pending_call.is_some() {
                return Status::HostCall;
            }
            if std::mem::take(&mut self.interrupt) {
                self.input.clear();
                self.recover(host, Fault::Interrupted);
                continue;
            }
            match self.step(host) {
                Ok(Flow::Continue) => {}
                Ok(Flow::NeedInput) => return Status::NeedInput,
                Ok(Flow::HostCall) => return Status::HostCall,
                Ok(Flow::Halt(code)) => {
                    self.halted = Some(code);
                    return Status::Halted(code);
                }
                Ok(Flow::Restart) => {
                    self.restart = true;
                    return Status::Restart;
                }
                Ok(Flow::NestedReturn) => self.recover(
                    host,
                    Fault::BadGate {
                        addr: Gate::NestedReturn.addr(),
                    },
                ),
                Err(f) => self.recover(host, f),
            }
        }
        match self.halted {
            Some(code) => Status::Halted(code),
            None if self.restart => Status::Restart,
            None => Status::Yielded,
        }
    }

    /// Run until the machine needs input or halts.
    pub fn run_until_blocked(&mut self, host: &mut dyn Host) -> Status {
        loop {
            match self.run(host, 1 << 20) {
                Status::Yielded => continue,
                s => return s,
            }
        }
    }

    /// The host call [`Status::HostCall`] waits on.
    pub fn pending_host_call(&self) -> Option<&crate::hostcall::HostCall> {
        self.pending_call.as_ref()
    }

    /// Answer the pending host call; the next [`Machine::run`] continues.
    pub fn resume(&mut self, reply: crate::hostcall::HostReply) {
        self.pending_call = None;
        self.host_reply = Some(crate::hostcall::HostAnswer::Reply(reply));
    }

    /// Refuse the pending host call: the line stops with `why`, as for a
    /// service the host does not have.
    pub fn refuse(&mut self, why: String) {
        self.pending_call = None;
        self.host_reply = Some(crate::hostcall::HostAnswer::Unavailable(why));
    }

    pub fn halted(&self) -> Option<i32> {
        self.halted
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn memory(&self) -> &Memory {
        &self.mem
    }

    /// Read a static variable (see [`st`]).
    pub fn var(&self, addr: u32) -> u32 {
        self.mem.read_u32(addr).unwrap_or(0)
    }

    /// The data stack, bottom first, TOS last.  Reva keeps TOS in EAX and
    /// the cell at `S0-4` is a placeholder, so depth is `(S0-ESI)/4`.
    pub fn data_stack(&self) -> Vec<u32> {
        let esi = self.cpu.r[ESI as usize];
        if esi >= S0 {
            return Vec::new();
        }
        let depth = (S0 - esi) / 4;
        let mut v: Vec<u32> = (1..depth)
            .map(|k| self.mem.read_u32(S0 - 4 - 4 * k).unwrap_or(0))
            .collect();
        v.push(self.cpu.r[EAX as usize]);
        v
    }

    /// Look a word up in the current search order; returns its xt.
    pub fn find(&self, name: &str) -> Option<u32> {
        self.find_entry(name.as_bytes())
            .and_then(|e| self.mem.read_u32(e + 4).ok())
    }

    /// Name the code at `addr`: a core routine, or the dictionary word whose
    /// code starts closest below it (with the offset into it).  Used in
    /// fault reports so a crash points at Forth source, not just a number.
    pub fn describe_addr(&self, addr: u32) -> String {
        if let Some(g) = Gate::from_addr(addr) {
            return format!("core:{}", g.name());
        }
        let mut best: Option<(u32, u32)> = None;
        let mut e = self.mem.read_u32(self.var(st::LAST)).unwrap_or(0);
        let mut guard = 0;
        while e != 0 && guard < 1 << 20 {
            let xt = self.mem.read_u32(e + 4).unwrap_or(0);
            if xt <= addr && best.is_none_or(|(bx, _)| xt > bx) {
                best = Some((xt, e));
            }
            e = self.mem.read_u32(e).unwrap_or(0);
            guard += 1;
        }
        match best {
            Some((xt, e)) => {
                let n = self.mem.read_u8(e + 8).unwrap_or(0) as u32;
                let name = self.mem.slice(e + 9, n).unwrap_or(b"?");
                format!("{}+{:#x}", String::from_utf8_lossy(name), addr - xt)
            }
            None => format!("{addr:#010x}"),
        }
    }

    /// Trace every step (gateway or instruction) to stderr.  Slow; for
    /// diagnosing the port, not for production.
    pub fn set_trace(&mut self, on: bool) {
        self.trace = on;
    }

    /// Install a translator (see [`crate::translate`]).
    pub fn set_translator(&mut self, t: Box<dyn Translator>) {
        self.translator = Some(t);
    }

    // ---- the dispatcher ----------------------------------------------

    pub(crate) fn step(&mut self, host: &mut dyn Host) -> Result<Flow, Fault> {
        self.steps += 1;
        let eip = self.cpu.eip;
        if self.trace {
            eprintln!(
                "{eip:#010x} {:<22} eax={:08x} esi={:08x} esp={:08x}",
                self.describe_addr(eip),
                self.cpu.r[EAX as usize],
                self.cpu.r[ESI as usize],
                self.cpu.r[ESP as usize]
            );
        }
        if (GATE_BASE..GATE_END).contains(&eip) {
            return self.enter_gate(host, eip);
        }
        if self.mem.has_dirty() {
            self.invalidate_dirty();
        }
        let insn = match self.cache.get(&eip) {
            Some(i) => *i,
            None => {
                let i = decode(&self.mem, eip)?;
                self.mem.mark_code(eip);
                self.mem.mark_code(eip.wrapping_add(i.len as u32 - 1));
                self.cache.insert(eip, i);
                i
            }
        };
        self.cpu.exec(&mut self.mem, &insn)?;
        Ok(Flow::Continue)
    }

    fn invalidate_dirty(&mut self) {
        for page in self.mem.take_dirty() {
            // An instruction (max 15 bytes) may start before the page.
            for a in page.saturating_sub(15)..page + PAGE_SIZE {
                self.cache.remove(&a);
            }
            if let Some(t) = self.translator.as_mut() {
                t.invalidate(page, PAGE_SIZE);
            }
        }
    }

    fn enter_gate(&mut self, host: &mut dyn Host, addr: u32) -> Result<Flow, Fault> {
        let g = Gate::from_addr(addr).ok_or(Fault::BadGate { addr })?;
        match self.gate(g, host)? {
            Next::Ret => {
                self.cpu.eip = self.cpu.pop32(&self.mem)?;
                Ok(Flow::Continue)
            }
            Next::Jump(t) => {
                self.cpu.eip = t;
                Ok(Flow::Continue)
            }
            Next::Call(t, k) => {
                self.cpu.push32(&mut self.mem, k.addr())?;
                self.cpu.eip = t;
                Ok(Flow::Continue)
            }
            Next::NeedInput => Ok(Flow::NeedInput),
            Next::Halt(c) => Ok(Flow::Halt(c)),
            Next::Restart => Ok(Flow::Restart),
            Next::NestedReturn => Ok(Flow::NestedReturn),
            Next::HostCall => Ok(Flow::HostCall),
        }
    }

    /// Call Forth code at `target` from inside a Rust routine and run it to
    /// completion on the Rust stack.
    ///
    /// Used only for callbacks that are expected to be short and
    /// non-interactive (`do_cr`, `prompt`, `iterate` handlers, overridden
    /// `emit`/`type`).  If such code asks for input that has not arrived,
    /// the host gets one chance to supply it synchronously
    /// ([`Host::read_input_blocking`]); otherwise this faults.  Budgets do
    /// not apply inside a nested call.
    pub(crate) fn call_nested(&mut self, host: &mut dyn Host, target: u32) -> Result<(), Fault> {
        let saved_eip = self.cpu.eip;
        let entry_esp = self.cpu.r[ESP as usize];
        self.cpu.push32(&mut self.mem, Gate::NestedReturn.addr())?;
        self.cpu.eip = target;
        self.nested += 1;
        let result = loop {
            match self.step(host) {
                Ok(Flow::Continue) => {
                    if self.cpu.r[ESP as usize] > entry_esp {
                        break Err(Fault::UnwoundThroughHost);
                    }
                }
                Ok(Flow::NestedReturn) => break Ok(()),
                Ok(Flow::NeedInput) => match host.read_input_blocking() {
                    Some(bytes) if !bytes.is_empty() => self.input.extend(bytes),
                    Some(_) => self.input_closed = true,
                    None => break Err(Fault::InputInNestedCall),
                },
                Ok(Flow::Halt(code)) => {
                    self.halted = Some(code);
                    break Ok(());
                }
                Ok(Flow::Restart) => {
                    self.restart = true;
                    break Ok(());
                }
                Ok(Flow::HostCall) => {
                    self.pending_call = None;
                    break Err(Fault::Unavailable(
                        "a host call cannot wait inside a nested call".into(),
                    ));
                }
                Err(f) => break Err(f),
            }
        };
        self.nested -= 1;
        self.cpu.eip = saved_eip;
        result
    }

    /// Report a fault and return to the interactive loop, cleaning up the
    /// way `interpret`'s failure path does.
    fn recover(&mut self, host: &mut dyn Host, f: Fault) {
        let eip = self.cpu.eip;
        let at = self.describe_addr(eip);
        host.write(format!("\n*** {f} (in {at}, eip {eip:#010x})\n").as_bytes());
        let _ = self.rollback_definition();
        for (a, v) in [
            (st::IS_COMPILING, 0),
            (st::DIDHEADER, 0),
            (st::SOURCE, 0),
            (st::HANDLER, 0),
            (st::RP0_SAVED, RP0),
        ] {
            let _ = self.mem.write_u32(a, v);
        }
        let tp = self.var(st::TP);
        let _ = self.mem.write_u32(st::TIN, tp);
        self.cpu.r[ESI as usize] = S0;
        self.cpu.r[EAX as usize] = 0;
        self.cpu.r[ESP as usize] = RP0;
        self.cpu.df = false;
        // Restart the interactive loop: reva.f's own (`exception-reset`,
        // which runs `interp` under `catch`) once booted; otherwise the
        // core's REPL, or `appstart` if a program installed its own.
        let entry = self.var(st::REPL_ENTRY);
        let app = self.var(st::APPSTART);
        self.cpu.eip = if entry != 0 {
            entry
        } else if app == Gate::Hello.addr() {
            Gate::Repl.addr()
        } else {
            app
        };
    }

    /// Undo a half-compiled definition (`interpret`'s `.fail` path).
    pub(crate) fn rollback_definition(&mut self) -> Result<(), Fault> {
        if self.var(st::IS_COMPILING) != 0 && self.var(st::DIDHEADER) != 0 {
            let lasth = self.var(st::LASTH);
            let lastd = self.var(st::LASTD);
            let lastl = self.var(st::LASTL);
            let lastp = self.var(st::LAST);
            self.mem.write_u32(st::H, lasth)?;
            self.mem.write_u32(st::D, lastd)?;
            self.mem.write_u32(lastp, lastl)?;
        }
        self.mem.write_u32(st::IS_COMPILING, 0)?;
        self.mem.write_u32(st::DIDHEADER, 0)
    }

    /// Dictionary search (`__find`), returning the entry address.
    pub(crate) fn find_entry(&self, name: &[u8]) -> Option<u32> {
        let mut e = self.mem.read_u32(self.var(st::LAST)).ok()?;
        while e != 0 {
            let n = self.mem.read_u8(e + 8).ok()?;
            if n as usize == name.len() & 0xff
                && let Ok(s) = self.mem.slice(e + 9, name.len() as u32)
                && s == name
            {
                return Some(e);
            }
            e = self.mem.read_u32(e).ok()?;
        }
        None
    }
}

// The statics block must fit its region (checked at compile time).
const _: () = assert!(STATIC_BASE + 4 * st::COUNT <= STATIC_END);
