//! Host calls to the page (docs/design.md 3.11).  The module's host answers
//! every host call *later*: `reva_run` returns 3 (HOST_CALL), the page reads
//! the call as JSON, asks whoever serves it (the desktop app's native side;
//! the browser refuses until it has SQLite) and answers with one of the
//! `reva_resume_*` / `reva_refuse` exports, then runs again.

use reva_vm::hostcall::{HostCall, HostReply};
use reva_vm::prelude::*;

use super::{FSBUF, with};

thread_local! {
    /// The pending call as JSON (see [`reva_host_call_ptr`]).
    static CALL: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// The pending call as JSON:
/// `{"service":"sqlite","op":"step","ints":[7],"bytes":[[104,105]]}`
/// (byte strings as arrays of numbers), or `null`.  Its length is
/// [`reva_host_call_len`]; the text stays until the next call to this.
#[unsafe(no_mangle)]
pub extern "C" fn reva_host_call_ptr() -> *const u8 {
    let json = with(|w| w.vm.pending_host_call().map(to_json), None).unwrap_or("null".into());
    CALL.with(|c| {
        *c.borrow_mut() = json;
        c.borrow().as_ptr()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn reva_host_call_len() -> u32 {
    CALL.with(|c| c.borrow().len() as u32)
}

/// Answer the pending call with a number.
#[unsafe(no_mangle)]
pub extern "C" fn reva_resume_int(v: f64) {
    with(|w| w.vm.resume(HostReply::Int(v as i64)), ())
}

/// Answer the pending call with the first `len` bytes of the buffer from
/// `reva_fs_buffer`.
#[unsafe(no_mangle)]
pub extern "C" fn reva_resume_bytes(len: u32) {
    let bytes = buffer(len);
    with(|w| w.vm.resume(HostReply::Bytes(bytes)), ())
}

/// Refuse the pending call: the line stops with the message in the first
/// `len` bytes of the `reva_fs_buffer` buffer.
#[unsafe(no_mangle)]
pub extern "C" fn reva_refuse(len: u32) {
    let why = String::from_utf8_lossy(&buffer(len)).into_owned();
    with(|w| w.vm.refuse(why), ())
}

fn buffer(len: u32) -> Vec<u8> {
    FSBUF.with(|b| {
        let b = b.borrow();
        b[..(len as usize).min(b.len())].to_vec()
    })
}

/// `call` as the JSON [`reva_host_call_ptr`] gives.  Service and operation
/// names are plain identifiers, so they need no escaping.
pub(crate) fn to_json(call: &HostCall) -> String {
    let list = |v: Vec<String>| v.join(",");
    let ints = list(call.ints.iter().map(i64::to_string).collect());
    let bytes = list(
        call.bytes
            .iter()
            .map(|b| format!("[{}]", list(b.iter().map(u8::to_string).collect())))
            .collect(),
    );
    format!(
        r#"{{"service":"{}","op":"{}","ints":[{ints}],"bytes":[{bytes}]}}"#,
        call.service, call.op
    )
}
