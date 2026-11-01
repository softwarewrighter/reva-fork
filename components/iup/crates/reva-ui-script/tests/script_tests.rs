use reva_hostcall::hostcall::{HostAnswer, HostCall, HostReply};
use reva_ui_script::script::{UiScript, lines};

fn call(op: &'static str, bytes: &[&str]) -> HostCall {
    HostCall {
        service: "ui",
        op,
        ints: vec![],
        bytes: bytes.iter().map(|b| b.as_bytes().to_vec()).collect(),
    }
}

#[test]
fn scripts_have_one_event_a_line() {
    let l = lines("# setup\n\nkey K_F6\ntype a b\n  click 3  \n");
    assert_eq!(
        Vec::from(l),
        ["key K_F6", "key 97", "key 32", "key 98", "click 3"]
    );
}

#[test]
fn waits_get_events_and_show_the_changes() {
    let mut s = UiScript::new("click 2\nfile /tmp/x\ncancel\n");
    let (a, log) = s.answer(&call("wait", &["created 1 dialog\nshown 1 -1 -1 modeless"]));
    assert_eq!(a, HostAnswer::Reply(HostReply::Bytes(b"click 2".to_vec())));
    assert_eq!(
        log,
        "ui< created 1 dialog\nui< shown 1 -1 -1 modeless\nui> click 2\n"
    );
    let (a, log) = s.answer(&call("file", &["OPEN", "", "*.f", "", ""]));
    assert_eq!(a, HostAnswer::Reply(HostReply::Bytes(b"/tmp/x".to_vec())));
    assert_eq!(log, "ui< file OPEN |  | *.f |  | \nui> file /tmp/x\n");
    let (a, _) = s.answer(&call("file", &[]));
    assert_eq!(a, HostAnswer::Reply(HostReply::Int(-1)), "cancelled");
    let (a, _) = s.answer(&call("wait", &[""]));
    assert!(matches!(a, HostAnswer::Unavailable(_)), "no more events");
}

#[test]
fn a_file_answer_where_an_event_is_due_is_an_error() {
    let mut s = UiScript::new("cancel\n");
    assert!(matches!(
        s.answer(&call("file", &[])).0,
        HostAnswer::Reply(_)
    ));
    let mut s = UiScript::new("click 1\n");
    assert!(matches!(
        s.answer(&call("file", &[])).0,
        HostAnswer::Unavailable(_)
    ));
}
