//! A host call as JSON for the page.

use reva_vm::hostcall::HostCall;

/// `call` as the JSON [`reva_host_call_ptr`] gives.  Service and operation
/// names are plain identifiers, so they need no escaping.
pub fn to_json(call: &HostCall) -> String {
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
