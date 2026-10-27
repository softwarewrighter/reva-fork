//! # The virtual library `reva-sqlite`
//!
//! `lib/db/sqlite` reaches SQLite through these functions (`lib`/`func:`
//! and `(call)`, like `turtle`).  None of them takes or returns a C
//! pointer: handles are small integers, strings come in as `( a n )` and go
//! out as a length, with the bytes in a VM buffer whose address `rsql_ptr`
//! gives.  Each call becomes a [`HostCall`] for service `sqlite`, which the
//! host answers (rust/DESIGN.md 3.11; the CLI and the desktop app with the
//! reva-sqlite crate).

use crate::fault::Fault;
use crate::gates::Gate;
use crate::host::Host;
use crate::hostcall::{HostAnswer, HostCall, HostReply};
use crate::machine::{Machine, Next};
use crate::x86::ESI;

/// Handle `(lib)` returns for this library ("RSQL").
pub const SQL_HANDLE: u32 = 0x5253_514c;

/// Library names accepted for it.
pub const SQL_NAMES: &[&str] = &["reva-sqlite"];

/// An argument: one cell, or a string as two cells `( a n )`.
#[derive(Clone, Copy)]
enum Arg {
    Int,
    Str,
}
use Arg::{Int, Str};

/// Every function: Forth name, gate, host operation, argument shape.  The
/// reply is a number, or bytes returned as their length (see `rsql_ptr`).
const FUNCS: &[(&str, Gate, &str, &[Arg])] = &[
    ("rsql_rc", Gate::SqRc, "rc", &[]),
    ("rsql_version", Gate::SqVersion, "version", &[]),
    ("rsql_open", Gate::SqOpen, "open", &[Str]),
    ("rsql_close", Gate::SqClose, "close", &[Int]),
    ("rsql_exec", Gate::SqExec, "exec", &[Int, Str]),
    ("rsql_lastid", Gate::SqLastid, "lastid", &[Int]),
    ("rsql_errmsg", Gate::SqErrmsg, "errmsg", &[Int]),
    ("rsql_query", Gate::SqQuery, "query", &[Int, Str]),
    ("rsql_next", Gate::SqNext, "next", &[Int]),
    ("rsql_getcol", Gate::SqGetcol, "getcol", &[Int, Int]),
    ("rsql_endquery", Gate::SqEndquery, "endquery", &[Int]),
    ("rsql_prepare", Gate::SqPrepare, "prepare", &[Int, Str]),
    ("rsql_bind_null", Gate::SqBindNull, "bind_null", &[Int, Int]),
    (
        "rsql_bind_int",
        Gate::SqBindInt,
        "bind_int",
        &[Int, Int, Int],
    ),
    (
        "rsql_bind_text",
        Gate::SqBindText,
        "bind_text",
        &[Int, Int, Str],
    ),
    (
        "rsql_bind_blob",
        Gate::SqBindBlob,
        "bind_blob",
        &[Int, Int, Str],
    ),
    (
        "rsql_bind_zeroblob",
        Gate::SqBindZeroblob,
        "bind_zeroblob",
        &[Int, Int, Int],
    ),
    ("rsql_reset", Gate::SqReset, "reset", &[Int]),
    ("rsql_clear", Gate::SqClear, "clear", &[Int]),
    ("rsql_step", Gate::SqStep, "step", &[Int]),
    ("rsql_finalize", Gate::SqFinalize, "finalize", &[Int]),
    (
        "rsql_column_int",
        Gate::SqColumnInt,
        "column_int",
        &[Int, Int],
    ),
    (
        "rsql_column_text",
        Gate::SqColumnText,
        "column_text",
        &[Int, Int],
    ),
    (
        "rsql_column_blob",
        Gate::SqColumnBlob,
        "column_blob",
        &[Int, Int],
    ),
];

/// `rsql_ptr` ( -- a ): where the last bytes reply was put.
pub const PTR_NAME: &str = "rsql_ptr";

/// The gate for a function name (`rsql_ptr` included).
pub fn lookup(name: &str) -> Option<Gate> {
    if name == PTR_NAME {
        return Some(Gate::SqPtr);
    }
    FUNCS.iter().find(|f| f.0 == name).map(|f| f.1)
}

fn spec(g: Gate) -> Option<(&'static str, &'static [Arg])> {
    FUNCS.iter().find(|f| f.1 == g).map(|f| (f.2, f.3))
}

impl Machine {
    /// `(call)` of a reva-sqlite function with `n` cells: ask the host and,
    /// once the reply is in, consume the call's cells and push the result.
    /// `None` if `g` is not one of them.
    pub(crate) fn sqlite_call(
        &mut self,
        host: &mut dyn Host,
        g: Gate,
        n: u32,
    ) -> Option<Result<Next, Fault>> {
        if g == Gate::SqPtr {
            return Some(self.ffi_finish(n, self.sql_ptr).map(|()| Next::Ret));
        }
        let (op, shape) = spec(g)?;
        Some(self.sqlite_answer(host, op, shape, n))
    }

    fn sqlite_answer(
        &mut self,
        host: &mut dyn Host,
        op: &'static str,
        shape: &[Arg],
        n: u32,
    ) -> Result<Next, Fault> {
        let call = self.sqlite_args(op, shape, n)?;
        let answer = match self.host_reply.take() {
            Some(reply) => HostAnswer::Reply(reply),
            None => host.host_call(&call),
        };
        match answer {
            HostAnswer::Reply(HostReply::Int(v)) => self.ffi_finish(n, v as u32)?,
            HostAnswer::Reply(HostReply::Bytes(b)) => {
                let p = self.sqlite_buffer(&call, b.len() as u32)?;
                self.mem.write_bytes(p, &b)?;
                self.sql_ptr = p;
                self.ffi_finish(n, b.len() as u32)?;
            }
            HostAnswer::Later => {
                self.pending_call = Some(call);
                return Ok(Next::HostCall);
            }
            HostAnswer::Unavailable(why) => return Err(Fault::Unavailable(why)),
        }
        Ok(Next::Ret)
    }

    /// The call's arguments, read without popping (`n` cells under the
    /// function address and the count, the first argument deepest).
    fn sqlite_args(&self, op: &'static str, shape: &[Arg], n: u32) -> Result<HostCall, Fault> {
        let width: u32 = shape.iter().map(|a| matches!(a, Str) as u32 + 1).sum();
        if width != n {
            return Err(Fault::NotImplemented(
                "reva-sqlite: wrong number of arguments",
            ));
        }
        let esi = self.cpu.r[ESI as usize];
        let cell = |i: u32| self.mem.read_u32(esi + 4 * (n - i));
        let (mut ints, mut bytes, mut i) = (Vec::new(), Vec::new(), 0);
        for a in shape {
            match a {
                Int => ints.push(i64::from(cell(i)? as i32)),
                Str => {
                    let (a, len) = (cell(i)?, cell(i + 1)?);
                    bytes.push(self.mem.slice(a, len)?.to_vec());
                    i += 1;
                }
            }
            i += 1;
        }
        Ok(HostCall {
            service: "sqlite",
            op,
            ints,
            bytes,
        })
    }

    /// A heap buffer for a bytes reply of `len` bytes.  Column replies get
    /// one per (statement, column), so the texts of a row's columns stay
    /// valid together until the next step, as SQLite's own pointers do;
    /// other replies share one.  Host state: not part of an image.
    fn sqlite_buffer(&mut self, call: &HostCall, len: u32) -> Result<u32, Fault> {
        let key = match call.op {
            "column_text" | "column_blob" | "getcol" => (call.ints[0], call.ints[1]),
            _ => (0, -1),
        };
        if let Some(&(p, cap)) = self.sql_bufs.get(&key) {
            if cap >= len {
                return Ok(p);
            }
            self.heap.free(p);
        }
        let cap = len.max(64);
        let p = self.heap.alloc(cap).ok_or(Fault::OutOfSpace("heap"))?;
        self.sql_bufs.insert(key, (p, cap));
        Ok(p)
    }
}
