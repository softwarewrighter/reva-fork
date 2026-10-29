//! The gates of the OS layer and of the file words.

use crate::call::Call;
use crate::loader::Loader;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next};
use reva_words_files::filewords::FileWords;
use reva_words_files::save::Save;
use reva_words_libc::syscall::Syscall;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    LoadLib => ret |m, _h| m.ffi_loadlib();
    UnloadLib => ret |m, _h| m.ffi_unloadlib();
    OsFunc => ret |m, h| m.ffi_func(h);
    CallFn => call_fn;
    Syscall => ret |m, h| m.ffi_syscall(h);
    OpenR => ret |m, h| m.word_open(h, reva_core::vfs::Access::Read);
    OpenRw => ret |m, h| m.word_open(h, reva_core::vfs::Access::ReadWrite);
    Creat => ret |m, h| m.word_open(h, reva_core::vfs::Access::Create);
    Close => ret |m, h| m.word_close(h);
    Read => ret |m, h| m.word_read(h);
    Write => ret |m, h| m.word_write(h);
    Fsize => ret |m, h| m.word_fsize(h);
    Save => ret |m, h| m.word_save(h);
    HostGetenv | HostPutenv | HostLocaltime | HostOpendir | HostReaddir | HostClosedir
        | HostSystem | TtForward | TtStep | TtRight | TtPen | TtColor | TtWidth | TtHome
        | TtClear | TtClean | TtSetxy | TtSetheading | TtSetstep | TtScale | TtShow | TtX
        | TtY | TtHeading | TtPush | TtPop | TtPath | TtSvg | TtBg | TtSegments | HlType
        | HlColor | SqRc | SqVersion | SqOpen | SqClose | SqExec | SqLastid | SqErrmsg
        | SqQuery | SqNext | SqGetcol | SqEndquery | SqPrepare | SqBindNull | SqBindInt
        | SqBindText | SqBindBlob | SqBindZeroblob | SqReset | SqClear | SqStep
        | SqFinalize | SqColumnInt | SqColumnText | SqColumnBlob | SqPtr | IcOpen | Iconv
        | IcClose => c_function;
    MakeExe | LzPack | LzDepack | LzMax => not_implemented;
};

/// `(call)`.
pub fn call_fn(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    m.ffi_call(host)
}

/// A virtual library's function reached by `call` or `jmp`, not `(call)`.
pub fn c_function(_: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    Err(Fault::NotImplemented("a C function called without (call)"))
}

/// Words the portable core does not provide.
pub fn not_implemented(_: &mut Machine, g: Gate, _: &mut dyn Host) -> R<Next> {
    Err(Fault::NotImplemented(match g {
        Gate::MakeExe => "makeexe",
        Gate::LzPack => ">lz",
        Gate::LzDepack => "lz>",
        Gate::LzMax => "lzmax",
        _ => g.name(),
    }))
}
