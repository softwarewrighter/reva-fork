//! The statics' first values and the core dictionary, as tables.

use reva_core::embedded::APP_PATH;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::layout::*;
use reva_core::machine::ICONV_VERSION_VALUE;
use reva_core::st::*;
use reva_core::{Gate, Gate::*, Machine, Next};

/// A first value: a number, or a gate's address.
enum Init {
    V(u32),
    G(Gate),
}
use Init::{G, V};

/// The statics and DEFER vectors with a first value (the rest start at 0).
const STATICS: &[(u32, Init)] = &[
    (LAST, V(FLAST)),
    (H, V(H0)),
    (D, V(D0)),
    (DEFAULT_CLASS, G(FClass)),
    (PADCHAR, V(32)),
    (ICONV_VERSION, V(ICONV_VERSION_VALUE)),
    (BASE, V(10)),
    (OS, V(OS_ID)),
    (APPNAME, V(APP_FILE_NAME)),
    (STDOUT, V(1)),
    (TP, V(TIB)),
    (TIN, V(TIB)),
    (S0V, V(S0)),
    (D0V, V(D0)),
    (H0V, V(H0)),
    (TIBV, V(TIB)),
    (RP0V, V(RP0_SAVED)),
    (RP0_SAVED, V(RP0)),
    (SSEMI_DO_TAIL, V(0x90)),
    (PROMPT, G(Noop)),
    (APPSTART, G(Hello)),
    (KEY, G(OsKey)),
    (FINDINTERNAL, G(Find)),
    (IS_WORD, G(WordNotFound)),
    (TYPE, G(OsType)),
    (EMIT, G(OsEmit)),
    (DO_CR, G(Noop)),
    (EXCEPTION, G(OsBye)),
    (CTRLC, G(Noop)),
    (HEAPGONE, G(OsBye)),
    (DICTGONE, G(OsBye)),
    (HEADER_, G(HeaderDefault)),
];

/// What a core word's entry points at: a gate's routine, or a statics
/// cell (variables, values, DEFER vectors).
enum Xt {
    G(Gate),
    C(u32),
}

/// The core dictionary in `newcore.asm`'s `DICT` order (so the most recent
/// core word is `getextra`): name, what it points at, its class.
const CORE_WORDS: &[(&str, Xt, Gate)] = &[
    ("'", Xt::G(Tick), FClass),
    ("makeexe", Xt::G(MakeExe), FClass),
    ("exception", Xt::C(EXCEPTION), DClass),
    ("heapgone", Xt::C(HEAPGONE), DClass),
    ("dictgone", Xt::C(DICTGONE), DClass),
    ("ctrl-c", Xt::C(CTRLC), DClass),
    ("do_cr", Xt::C(DO_CR), DClass),
    ("s0", Xt::C(S0V), CClass),
    ("d0", Xt::C(D0V), VClass),
    ("h0", Xt::C(H0V), VClass),
    ("ioerr", Xt::C(IOR), VClass),
    ("rp0", Xt::C(RP0V), CClass),
    ("padchar", Xt::C(PADCHAR), VClass),
    (">in", Xt::C(TIN), VClass),
    ("tib", Xt::C(TIBV), VClass),
    ("tp", Xt::C(TP), VClass),
    ("src", Xt::C(SOURCE), VClass),
    ("state", Xt::C(IS_COMPILING), VClass),
    ("dict", Xt::C(D), VClass),
    ("(here)", Xt::C(H), VClass),
    ("last", Xt::C(LAST), CClass),
    ("base", Xt::C(BASE), VClass),
    ("(argv)", Xt::C(ARGV), ValClass),
    ("argc", Xt::C(ARGC), ValClass),
    ("hinst", Xt::C(HINSTANCE), ValClass),
    ("stdout", Xt::C(STDOUT), ValClass),
    ("stdin", Xt::C(STDIN), ValClass),
    ("os", Xt::C(OS), CClass),
    ("syscall", Xt::G(Syscall), FClass),
    ("nosavedict", Xt::C(NOSAVEDICT), VClass),
    ("cold", Xt::G(Cold), FClass),
    ("reset", Xt::G(Reset), FClass),
    ("prompt", Xt::C(PROMPT), DClass),
    ("default_class", Xt::C(DEFAULT_CLASS), VClass),
    (">lz", Xt::G(LzPack), FClass),
    ("lz>", Xt::G(LzDepack), FClass),
    ("lzmax", Xt::G(LzMax), FClass),
    ("'variable", Xt::G(VClass), FClass),
    ("'constant", Xt::G(CClass), FClass),
    ("'does", Xt::G(DoesClass), FClass),
    ("'value", Xt::G(ValClass), FClass),
    ("'forth", Xt::G(FClass), FClass),
    ("'notail", Xt::G(FClassNotail), FClass),
    ("'macro", Xt::G(MClass), FClass),
    ("'inline", Xt::G(IClass), FClass),
    ("'macront", Xt::G(MClassNotail), FClass),
    ("'defer", Xt::G(DClass), FClass),
    ("catch", Xt::G(Catch), FClass),
    ("throw", Xt::G(Throw), FClass),
    ("onexit", Xt::G(OnExit), FClass),
    ("onstartup", Xt::G(OnStartup), FClass),
    ("(lib)", Xt::G(LoadLib), FClass),
    ("(-lib)", Xt::G(UnloadLib), FClass),
    ("(call)", Xt::G(CallFn), FClass),
    ("(func)", Xt::G(OsFunc), FClass),
    ("fnvhash", Xt::G(FnvHash), FClass),
    ("(bye)", Xt::G(Bye), FClass),
    ("cmp", Xt::G(Cmp), FClass),
    ("cmpi", Xt::G(Cmpi), FClass),
    ("place", Xt::G(Place), FClass),
    ("+place", Xt::G(PlusPlace), FClass),
    ("move", Xt::G(Move), FClass),
    ("fill", Xt::G(Fill), FClass),
    ("zt", Xt::G(Zt), FClass),
    ("zcount", Xt::G(Zcount), FClass),
    ("open/r", Xt::G(OpenR), FClass),
    ("open/rw", Xt::G(OpenRw), FClass),
    ("creat", Xt::G(Creat), FClass),
    ("close", Xt::G(Close), FClass),
    ("read", Xt::G(Read), FClass),
    ("write", Xt::G(Write), FClass),
    ("fsize", Xt::G(Fsize), FClass),
    ("allocate", Xt::G(Allocate), FClass),
    ("free", Xt::G(Free), FClass),
    ("resize", Xt::G(Resize), FClass),
    ("header", Xt::G(Header), FClass),
    ("(header)", Xt::C(HEADER_), DClass),
    ("literal", Xt::G(Literal), MClass),
    (";;", Xt::G(SSemi), MClass),
    (";inline", Xt::G(InlineSemi), MClass),
    ("parsews", Xt::G(Parsews), FClass),
    ("align", Xt::G(Align), FClass),
    ("here,", Xt::G(StoreHere), FClass),
    ("compile", Xt::G(Compile), FClass),
    ("interp", Xt::G(Interpret), FClass),
    ("key", Xt::C(KEY), DClass),
    ("appstart", Xt::C(APPSTART), DClass),
    ("emit", Xt::C(EMIT), DClass),
    ("find-dict", Xt::C(FINDINTERNAL), DClass),
    (">single", Xt::G(Single), FClass),
    (">double", Xt::G(Double), FClass),
    ("word?", Xt::C(IS_WORD), DClass),
    ("type", Xt::C(TYPE), DClass),
    ("digit>", Xt::G(FromDigit), FClass),
    (">digit", Xt::G(ToDigit), FClass),
    (":", Xt::G(Colon), MClass),
    (";", Xt::G(Semi), MClass),
    ("[", Xt::G(LBracket), MClass),
    ("]", Xt::G(RBracket), MClass),
    ("2dup", Xt::G(TwoDup), FClass),
    ("variable", Xt::G(Variable), FClass),
    ("variable,", Xt::G(Variable2), FClass),
    (",", Xt::G(Comma), FClass),
    ("3,", Xt::G(Comma3), FClass),
    ("2,", Xt::G(Comma2), FClass),
    ("1,", Xt::G(Comma1), FClass),
    ("include", Xt::G(Include), FClass),
    ("(include)", Xt::G(Include2), FClass),
    ("eval", Xt::G(Eval), FClass),
    ("parse", Xt::G(Parse), FClass),
    ("parse/", Xt::G(ParseEsc), FClass),
    ("appname", Xt::C(APPNAME), CClass),
    ("(save)", Xt::G(Save), FClass),
    ("(.r)", Xt::G(PrintR), FClass),
    ("link", Xt::G(Link), FClass),
    ("-link", Xt::G(Link2), FClass),
    ("iterate", Xt::G(Iterate), FClass),
    ("slurp", Xt::G(Slurp), FClass),
    ("setextra", Xt::G(SetExtra), FClass),
    ("getextra", Xt::G(GetExtra), FClass),
    ("(boot-restart)", Xt::G(BootRestart), FClass),
    ("restart", Xt::G(Restart), FClass),
    ("version", Xt::G(Version), FClass),
];

/// The dispatch of a machine built without one: every gateway faults.
pub fn unwired(_: &mut Machine, _: Gate, _: &mut dyn Host) -> Result<Next, Fault> {
    Err(Fault::NotImplemented(
        "gateway dispatch not installed (build machines with reva_vm)",
    ))
}

reva_macros::machine_trait! {
    /// Filling a fresh machine's statics and dictionary.
    pub trait Tables {
        /// The statics' first values, and the application's file name.
        fn init_statics(&mut self) {
            for (a, v) in STATICS {
                let v = match v {
                    V(n) => *n,
                    G(g) => g.addr(),
                };
                self.mem.write_u32(*a, v).expect("statics");
            }
            self.mem.write_bytes(APP_FILE_NAME, APP_PATH.as_bytes()).expect("appname");
            self.mem
                .write_u8(APP_FILE_NAME + APP_PATH.len() as u32, 0)
                .expect("appname");
        }

        /// Create the core dictionary from [`CORE_WORDS`].
        fn build_dictionary(&mut self) {
            let mut d = D0;
            let mut prev = 0;
            for (name, xt, class) in CORE_WORDS {
                let xt = match xt {
                    Xt::G(g) => g.addr(),
                    Xt::C(cell) => *cell,
                };
                let (entry, n) = (d + 4, name.len() as u32);
                let w = &mut self.mem;
                w.write_u32(d, class.addr()).expect("dict");
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
    }
}
