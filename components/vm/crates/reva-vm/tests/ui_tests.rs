//! IUP programs on a host that answers the `ui` calls later, as the
//! desktop app's page does: the VM pauses at each wait, and runs the
//! callbacks the events ask for.

use reva_vm::host::{Console, Platform, Resources};
use reva_vm::hostcall::{HostAnswer, HostCall, HostReply};
use reva_vm::prelude::*;
use reva_vm::{BufferHost, Machine, Status, embedded};

struct Later {
    inner: BufferHost,
}

impl Console for Later {
    fn write(&mut self, bytes: &[u8]) {
        self.inner.write(bytes);
    }
}

impl Platform for Later {}

impl Resources for Later {
    fn fs(&mut self) -> Option<&mut dyn reva_vm::vfs::FileSystem> {
        self.inner.fs()
    }

    fn host_call(&mut self, _: &HostCall) -> HostAnswer {
        HostAnswer::Later
    }
}

fn booted() -> (Machine, Later) {
    let mut vm = Machine::new(&["reva", embedded::BOOT_PATH]);
    let mut host = Later {
        inner: BufferHost::new(),
    };
    vm.run_until_blocked(&mut host);
    vm.quiet_prompt();
    host.inner.take_output();
    (vm, host)
}

/// Run until the VM waits for a UI event; the changes it sent.
fn wait(vm: &mut Machine, host: &mut Later) -> String {
    assert_eq!(
        vm.run_until_blocked(host),
        Status::HostCall,
        "{}",
        host.inner.take_output()
    );
    let call = vm.pending_host_call().expect("a host call").clone();
    assert_eq!((call.service, call.op), ("ui", "wait"));
    String::from_utf8(call.bytes[0].clone()).unwrap()
}

fn answer(vm: &mut Machine, event: &str) {
    vm.resume(HostReply::Bytes(event.as_bytes().to_vec()));
}

const DIALOG: &str = "needs ui/gui with~ ~ui\n\
    : bye-cb callback .\" Goodbye\" cr gui-close ;\n\
    dialog[ \" Bye\" button[ ' bye-cb action ]w\n\
    \" Stay\" button[ { 1 2 3 .\" stay\" cr gui-default } action ]w ]d\n";

#[test]
fn the_vm_waits_for_events_and_runs_callbacks() {
    let (mut vm, mut host) = booted();
    vm.feed(
        format!("{DIALOG} 7 swap show gui-main-loop drop .\" depth \" depth . cr\n").as_bytes(),
    );
    let sent = wait(&mut vm, &mut host);
    assert!(
        sent.contains("created 2 button\nattr 2 TITLE =Bye"),
        "{sent}"
    );
    assert!(sent.ends_with("shown 4 -1 -1 modeless\nfocus 2"), "{sent}");
    answer(&mut vm, "click 3");
    assert_eq!(wait(&mut vm, &mut host), "", "nothing changed");
    assert_eq!(host.inner.take_output(), "stay\n");
    answer(&mut vm, "key K_TAB");
    assert_eq!(wait(&mut vm, &mut host), "focus 3");
    answer(&mut vm, "key K_sTAB");
    assert_eq!(wait(&mut vm, &mut host), "focus 2");
    answer(&mut vm, "key K_CR");
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    // The plain block's 1 2 3 are gone: a callback keeps the data stack as
    // C keeps its callee-saved registers.
    assert_eq!(host.inner.take_output(), "Goodbye\ndepth 1 \n");
    assert!(vm.ui.loops.is_empty() && vm.ui.calls.is_empty());
}

#[test]
fn a_fault_in_a_callback_leaves_the_loop() {
    let (mut vm, mut host) = booted();
    vm.feed(b"needs ui/gui with~ ~ui\ndialog[ \" Bad\" button[ { 0 @ } action ]w ]d show gui-main-loop\n");
    wait(&mut vm, &mut host);
    answer(&mut vm, "click 2");
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    assert!(host.inner.take_output().contains("***"));
    assert!(vm.ui.loops.is_empty() && vm.ui.calls.is_empty());
    vm.feed(b"1 2 + .\n");
    vm.run_until_blocked(&mut host);
    assert_eq!(host.inner.take_output(), "3 ");
}

#[test]
fn file_dialogs_and_hosts_without_a_ui() {
    let (mut vm, mut host) = booted();
    vm.feed(b"needs ui/gui with~ ~ui\nfile-open-dialog[ \" *.f\" \" FILTER\" attr ]fd popup getval type .\"  \" \" STATUS\" attr? type cr\n");
    assert_eq!(vm.run_until_blocked(&mut host), Status::HostCall);
    let call = vm.pending_host_call().unwrap().clone();
    assert_eq!(
        (call.op, call.bytes[0].as_slice(), call.bytes[2].as_slice()),
        ("file", &b"OPEN"[..], &b"*.f"[..])
    );
    vm.resume(HostReply::Bytes(b"/home/a.f".to_vec()));
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    assert_eq!(host.inner.take_output(), "/home/a.f 0\n");
    vm.feed(b"dialog[ ]d show gui-main-loop\n");
    assert_eq!(vm.run_until_blocked(&mut host), Status::HostCall);
    vm.refuse("ui is not available on this page".into());
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    assert!(
        host.inner
            .take_output()
            .contains("ui is not available on this page")
    );
}
