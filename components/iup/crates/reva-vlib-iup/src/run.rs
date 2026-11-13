//! Showing dialogs, the event loop, and callbacks.
//!
//! `IupMainLoop` and `IupPopup` (of a dialog) pop their `(call)` and jump
//! to the gate `IupLoop`, which asks the host for an event (a `ui` `wait`
//! host call carrying the changes since the last one; a host that answers
//! later pauses the VM there).  An event that runs a callback calls the
//! Forth word as IUP called it: its arguments and the element on the stack
//! as C pushes them, returning to the gate `IupResume`, which restores the
//! data stack as C's convention keeps it, hands the result to the model
//! and goes on waiting.  When the loop ends, the `(call)` returns 0.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::hostcall::{HostAnswer, HostCall, HostReply};
use reva_core::x86::{Arith, EAX, ESI, ESP};
use reva_core::{Gate, Next, st};
use reva_ui::ui::{CLOSE, Call, DEFAULT, Event, Loop};
use reva_ui_base::kind::Kind;

use reva_core_ffi::args::CallArgs;

use crate::data::{IupData, changes, decode_event};
use crate::names::Op;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Dialogs, the event loop and callbacks.
    pub trait IupRun: IupData {
        /// `IupShow`, `IupShowXY`, `IupHide`, `IupPopup` and `IupMainLoop`
        /// with the arguments `a` of a `(call)` of `n` cells.
        fn iup_show(&mut self, host: &mut dyn Host, op: Op, a: &[u32], n: u32) -> R<Next> {
            let arg = |i: usize| a.get(i).copied().unwrap_or(0);
            let ih = arg(0);
            let kind = self.ui.tree.node(ih).map(|n| n.kind);
            let dialog = self.ui.tree.dialog_path(ih).last().copied().unwrap_or(0);
            let (x, y) = (arg(1) as i32, arg(2) as i32);
            match op {
                Op::Popup if matches!(kind, Some(Kind::FileDlg | Kind::MessageDlg)) => return self.iup_file(host, ih, n),
                Op::Popup | Op::MainLoop | Op::LoopStep => {
                    if op == Op::Popup {
                        self.ui.show(dialog, x, y, true);
                    }
                    for _ in 0..n + 2 {
                        self.ffi_pop()?;
                    }
                    self.ui.loops.push(match op {
                        Op::Popup => Loop::Popup(dialog),
                        Op::LoopStep => Loop::Step(false),
                        _ => Loop::Main,
                    });
                    return Ok(Next::Jump(Gate::IupLoop.addr()));
                }
                Op::Show => drop(self.ui.show(dialog, -1, -1, false)),
                Op::ShowXY => drop(self.ui.show(dialog, x, y, false)),
                _ => self.ui.hide(dialog),
            }
            self.ffi_finish(n, 0)?;
            Ok(Next::Ret)
        }

        /// The gate `IupLoop`: end the innermost loop, or wait for an event
        /// and do what it does.
        fn iup_loop(&mut self, host: &mut dyn Host) -> R<Next> {
            let step = match self.ui.loops.last() {
                Some(Loop::Step(done)) => Some(*done),
                _ => None,
            };
            if !self.ui.running() || step == Some(true) {
                let ret = match step {
                    Some(_) if !self.ui.running() => CLOSE,
                    Some(_) => DEFAULT,
                    None => 0,
                };
                self.ui.end_loop();
                self.ffi_push(ret as u32)?;
                return Ok(Next::Ret);
            }
            let Some(event) = self.iup_wait(host)? else {
                return Ok(Next::HostCall);
            };
            if let Some(Loop::Step(done)) = self.ui.loops.last_mut() {
                *done = true;
            }
            match self.ui.event(event) {
                Some(call) => self.iup_callback(call),
                None => Ok(Next::Jump(Gate::IupLoop.addr())),
            }
        }

        /// The host's next event; `None` while the host has yet to answer.
        fn iup_wait(&mut self, host: &mut dyn Host) -> R<Option<Event>> {
            let answer = match self.host_reply.take() {
                Some(answer) => answer,
                None => {
                    let call = HostCall { service: "ui", op: "wait", ints: vec![], bytes: vec![changes(&mut self.ui)] };
                    match host.host_call(&call) {
                        HostAnswer::Later => {
                            self.pending_call = Some(call);
                            return Ok(None);
                        }
                        answer => answer,
                    }
                }
            };
            match answer {
                HostAnswer::Reply(HostReply::Bytes(b)) => {
                    let line = String::from_utf8_lossy(&b).into_owned();
                    decode_event(&line).map(Some).ok_or_else(|| Fault::Unavailable(format!("ui: not an event: {line}")))
                }
                HostAnswer::Unavailable(why) => Err(Fault::Unavailable(why)),
                _ => Err(Fault::Unavailable("ui: the host answered a wait without an event".into())),
            }
        }

        /// Run `call`'s Forth word as IUP calls a callback.
        fn iup_callback(&mut self, call: Call) -> R<Next> {
            let (esi, eax) = (self.cpu.r[ESI as usize], self.cpu.r[EAX as usize]);
            for v in call.args.iter().rev().map(|a| *a as u32).chain([call.handle, Gate::IupResume.addr()]) {
                self.cpu.push32(&mut self.mem, v)?;
            }
            let func = call.func;
            let saved = [esi, eax, self.cpu.r[ESP as usize], self.var(st::HANDLER)];
            self.ui.calls.push((call, saved));
            Ok(Next::Jump(func))
        }

        /// The gate `IupResume`: a callback returned its result in EAX.
        fn iup_resume(&mut self) -> R<Next> {
            let ret = self.cpu.r[EAX as usize] as i32;
            let (call, [esi, eax, ..]) = self.ui.calls.pop().ok_or(Fault::NotImplemented("iup: a return with no callback running"))?;
            let esp = &mut self.cpu.r[ESP as usize];
            *esp = esp.wrapping_add(4 * (1 + call.args.len() as u32));
            self.cpu.r[ESI as usize] = esi;
            self.cpu.r[EAX as usize] = eax;
            match self.ui.finish(&call, ret) {
                Some(next) => self.iup_callback(next),
                None => Ok(Next::Jump(Gate::IupLoop.addr())),
            }
        }

        /// `IupPopup` of a file or message dialog: the host asks the user
        /// (a `ui` `file` call with DIALOGTYPE, TITLE, FILTER, DIRECTORY and
        /// MULTIPLEFILES); the answer is the dialog's VALUE, and STATUS is 0,
        /// or -1 if the user cancelled.
        fn iup_file(&mut self, host: &mut dyn Host, ih: u32, n: u32) -> R<Next> {
            let attrs = ["DIALOGTYPE", "TITLE", "FILTER", "DIRECTORY", "MULTIPLEFILES"];
            let bytes = attrs.iter().map(|a| self.ui.get(ih, a).unwrap_or_default().into_bytes()).collect();
            let call = HostCall { service: "ui", op: "file", ints: vec![], bytes };
            let answer = match self.host_reply.take() {
                Some(answer) => answer,
                None => host.host_call(&call),
            };
            let (value, status) = match answer {
                HostAnswer::Later => {
                    self.pending_call = Some(call);
                    return Ok(Next::HostCall);
                }
                HostAnswer::Unavailable(why) => return Err(Fault::Unavailable(why)),
                HostAnswer::Reply(HostReply::Bytes(b)) => (Some(String::from_utf8_lossy(&b).into_owned()), "0"),
                HostAnswer::Reply(HostReply::Int(_)) => (None, "-1"),
            };
            self.ui.set(ih, "VALUE", value.as_deref());
            self.ui.set(ih, "STATUS", Some(status));
            self.ffi_finish(n, 0)?;
            Ok(Next::Ret)
        }
    }
}
