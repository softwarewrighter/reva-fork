//! Making the machine, and remaking it on `restart`.

use crate::state::{PENDING, WEB, Web, code};
use reva_vm::prelude::*;
use reva_vm::{BufferHost, Machine, Start, Status, embedded};

/// Keep `vm` (made by `start`) as the module's VM, boot it until it first
/// waits, and return the status.
pub fn begin(start: Start, mut vm: Machine) -> i32 {
    let mut host = BufferHost::new();
    // The page renders ANSI colour escapes (www/ansi.js), so `pp` colours.
    host.color = true;
    // The page answers host calls (hostcall.rs).
    host.host_calls = true;
    if let Some(fs) = PENDING.with(|p| p.borrow_mut().take()) {
        host.fs = fs;
    }
    let status = startup(&mut vm, &mut host);
    WEB.with(|w| {
        *w.borrow_mut() = Some(Web {
            vm,
            host,
            input: Vec::new(),
            replay: Vec::new(),
            start,
            canvas_reset: false,
        })
    });
    code(status)
}

/// Run a new machine until it first waits, keeping what it printed except
/// reva.f's progress marks.
pub fn startup(vm: &mut Machine, host: &mut BufferHost) -> Status {
    let before = host.out.len();
    let status = vm.run_until_blocked(host);
    let out = String::from_utf8_lossy(&host.out[before..]).into_owned();
    let (_, rest) = embedded::split_boot_output(&out);
    host.out.truncate(before);
    host.out.extend_from_slice(rest.as_bytes());
    status
}

/// `restart`: a new machine made as the first one was, with the old one's
/// files and output, the start input again, then the unread input.
pub fn restart(w: &mut Web) -> Status {
    let (input, closed) = w.vm.take_input();
    // The same start made the first machine (an image was checked then).
    w.vm = w.start.machine().expect("a start that worked once");
    w.canvas_reset = true;
    match startup(&mut w.vm, &mut w.host) {
        Status::NeedInput => {}
        s => return s,
    }
    w.vm.feed(&w.replay);
    w.vm.feed(&input);
    if closed {
        w.vm.close_input();
    }
    Status::Yielded
}
