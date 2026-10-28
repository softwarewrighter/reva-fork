//! The gateway table: one [`Gate`] per Rust routine, each at a fixed
//! address in the gateway region (`layout::GATE_BASE`).  The routines
//! themselves are in reva-words and reva-vlib; the machine reaches them
//! through its dispatch function (rust/DESIGN.md 3.12).  Gates are only
//! ever appended: saved images hold their addresses.

use crate::layout::*;

macro_rules! gates {
    ($($v:ident),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u16)]
        pub enum Gate { $($v),* }
        impl Gate {
            pub const ALL: &'static [Gate] = &[$(Gate::$v),*];
            pub fn name(self) -> &'static str {
                match self { $(Gate::$v => stringify!($v)),* }
            }
        }
    };
}

gates! {
    // word classes (the CLASS field of a dictionary entry points at one)
    FClass, FClassNotail, MClass, MClassNotail, IClass, CClass, VClass,
    ValClass, DoesClass, DClass,
    // default targets of DEFER vectors
    Noop, OsEmit, OsType, OsKey, OsBye, Find, WordNotFound, HeaderDefault, Hello,
    // startup and the outer interpreter, split at every call into Forth
    Cold, ColdDone, Repl, Interpret, QueryKey, QueryKeyGot, InterpWord,
    InterpFound, InterpIsWord, Eval, EvalDone, Include, Include2, IncludeDone,
    Catch, CatchDone, Throw, NestedReturn, OnStartThunk, Tick, TickDone, Colon,
    Variable, Variable2, Variable2Done,
    // the remaining core words
    MakeExe, Syscall, Reset, LzPack, LzDepack, LzMax, OnExit, OnStartup,
    LoadLib, UnloadLib, CallFn, OsFunc, FnvHash, Bye, Cmp, Cmpi, Place,
    PlusPlace, Move, Fill, Zt, Zcount, OpenR, OpenRw, Creat, Close, Read, Write,
    Fsize, Allocate, Free, Resize, Header, Literal, SSemi, InlineSemi, Semi,
    LBracket, RBracket, Parsews, Align, StoreHere, Compile, Single, Double,
    FromDigit, ToDigit, TwoDup, Comma, Comma3, Comma2, Comma1, Parse, ParseEsc,
    Save, PrintR, Link, Link2, Iterate, Slurp, SetExtra, GetExtra,
    // functions of the virtual libc, reached through `(call)` (see ffi.rs)
    HostGetenv, HostPutenv, HostLocaltime,
    // end of the embedded boot: restart as the saved executable would
    BootRestart,
    // more virtual libc functions (appended: gate addresses are in images)
    HostOpendir, HostReaddir, HostClosedir,
    // `restart`: the host starts a new machine the way it started this one
    Restart,
    HostSystem,
    // `version`: print this build's version report (see version.rs)
    Version,
    // the virtual library `turtle` (turtle.rs, canvas.rs), through `(call)`
    TtForward, TtStep, TtRight, TtPen, TtColor, TtWidth, TtHome, TtClear,
    TtClean, TtSetxy, TtSetheading, TtSetstep, TtScale, TtShow, TtX, TtY,
    TtHeading, TtPush, TtPop, TtPath, TtSvg, TtBg, TtSegments,
    // the virtual library `highlight` (highlight.rs), through `(call)`
    HlType, HlColor,
    // the virtual library `reva-sqlite` (sqlite.rs), host calls through `(call)`
    SqRc, SqVersion, SqOpen, SqClose, SqExec, SqLastid, SqErrmsg, SqQuery,
    SqNext, SqGetcol, SqEndquery, SqPrepare, SqBindNull, SqBindInt, SqBindText,
    SqBindBlob, SqBindZeroblob, SqReset, SqClear, SqStep, SqFinalize,
    SqColumnInt, SqColumnText, SqColumnBlob, SqPtr,
    // the virtual library `libiconv` (iconv.rs), through `(call)`
    IcOpen, Iconv, IcClose,
}

impl Gate {
    #[inline]
    pub fn addr(self) -> u32 {
        GATE_BASE + GATE_STRIDE * self as u32
    }

    #[inline]
    pub fn from_addr(a: u32) -> Option<Gate> {
        let off = a.checked_sub(GATE_BASE)?;
        if off % GATE_STRIDE != 0 {
            return None;
        }
        Gate::ALL.get((off / GATE_STRIDE) as usize).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gates_fit_in_their_region() {
        assert!(Gate::ALL.len() as u32 * GATE_STRIDE <= GATE_END - GATE_BASE);
        for (i, g) in Gate::ALL.iter().enumerate() {
            assert_eq!(*g as usize, i);
            assert_eq!(Gate::from_addr(g.addr()), Some(*g));
        }
        assert_eq!(Gate::from_addr(GATE_BASE + 1), None);
        assert_eq!(Gate::from_addr(GATE_END), None);
    }
}
