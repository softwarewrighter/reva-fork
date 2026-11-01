//! Events: which callback a key, click or close runs, and what happens
//! when there is none or it lets the event through.
//!
//! A key goes to the focused element: its callback named for the key
//! (`K_F6`, `K_sF6`, `K_cS`), looked up on the element and then its
//! parents, else `K_ANY` (given the key code) the same way.  When neither
//! exists, or the callback returns `DEFAULT` or `CONTINUE`, the key does
//! what it does by default: Tab and Shift-Tab move the focus, Enter and
//! space click a button, other keys edit a text element.  (IUP calls
//! `K_ANY` before the key's own callback; programs set one or the other.)

use reva_ui_base::change::{Change, Handle};
use reva_ui_base::keys::{CR, SHIFT, SP, TAB, name};
use reva_ui_base::kind::Kind;

use crate::ui::{CLOSE, CONTINUE, Call, DEFAULT, Event, Ui};

impl Ui {
    /// What `event` does: a callback for the host to run (then
    /// [`Ui::finish`]), or `None` when the model has done it all.
    pub fn event(&mut self, event: Event) -> Option<Call> {
        match event {
            Event::Key(code) => {
                let target = self.focus.or_else(|| self.shown.last().copied())?;
                let named = name(code).and_then(|n| self.call(target, &n, vec![], event));
                named
                    .or_else(|| self.call(target, "K_ANY", vec![code as i32], event))
                    .or_else(|| self.default_key(code))
            }
            Event::Click(h) => self.click(h, event),
            Event::Focus(h) => {
                self.focus = Some(h);
                self.tree.changes.push(Change::Focus { handle: h });
                None
            }
            Event::Close(d) => self.call(d, "CLOSE_CB", vec![], event).or_else(|| {
                self.hide(d);
                None
            }),
        }
    }

    /// After the host ran `call`, which returned `ret`: `CLOSE` ends the
    /// main loop; a key callback's `DEFAULT` or `CONTINUE` lets the key
    /// do what it does by default (which may be another callback).
    pub fn finish(&mut self, call: &Call, ret: i32) -> Option<Call> {
        if ret == CLOSE {
            self.closed = true;
            return None;
        }
        match call.event {
            Event::Key(code) if ret == DEFAULT || ret == CONTINUE => self.default_key(code),
            _ => None,
        }
    }

    /// Callback `name` of `handle` or a parent, as a [`Call`].
    fn call(&self, handle: Handle, name: &str, args: Vec<i32>, event: Event) -> Option<Call> {
        let (handle, func) = self.tree.callback(handle, name)?;
        Some(Call {
            handle,
            name: name.to_string(),
            func,
            args,
            event,
        })
    }

    /// A click on `h`: a toggle flips its VALUE; buttons, toggles and menu
    /// items run their ACTION (a toggle's given its new state, 1 or 0).
    fn click(&mut self, h: Handle, event: Event) -> Option<Call> {
        let kind = self.tree.node(h)?.kind;
        let args = match kind {
            Kind::Toggle => {
                let on = self.get(h, "VALUE").is_some_and(|v| v == "ON");
                self.set(h, "VALUE", Some(if on { "OFF" } else { "ON" }));
                vec![i32::from(!on)]
            }
            Kind::Button | Kind::Item => vec![],
            _ => return None,
        };
        self.call(h, "ACTION", args, event)
    }

    /// What key `code` does with no callback taking it.
    fn default_key(&mut self, code: u32) -> Option<Call> {
        let focus = self.focus?;
        if code & !SHIFT == TAB {
            let dialog = *self.tree.dialog_path(focus).last()?;
            let all = self.focusable(dialog);
            let at = all.iter().position(|h| *h == focus)?;
            let next = if code & SHIFT != 0 {
                at + all.len() - 1
            } else {
                at + 1
            };
            return self.event(Event::Focus(all[next % all.len()]));
        }
        let kind = self.tree.node(focus)?.kind;
        if matches!(kind, Kind::Button | Kind::Toggle) && (code == CR || code == SP) {
            return self.event(Event::Click(focus));
        }
        if self.texts.get_mut(&focus).is_some_and(|t| t.key(code)) {
            self.note_text(focus);
        }
        None
    }
}
