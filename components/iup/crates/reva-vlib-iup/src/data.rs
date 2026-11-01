//! Data in and out: C strings from the VM, strings returned into it, the
//! change stream sent to the host and the events it answers with.
//!
//! An event is one line of text: `key K_F6` (or `key 65471`), `click 5`,
//! `focus 3`, `close 1` (handles are the model's).

use reva_core::fault::Fault;
use reva_core_ffi::args::CallArgs;
use reva_ui::ui::{Event, Ui};
use reva_ui_base::codec::encode;
use reva_ui_base::keys::from_name;

type R<T = ()> = Result<T, Fault>;

/// The changes not yet sent, one line each.
pub fn changes(ui: &mut Ui) -> Vec<u8> {
    let lines: Vec<String> = ui.tree.take_changes().iter().map(encode).collect();
    lines.join("\n").into_bytes()
}

/// The event a host's line names.
pub fn decode_event(line: &str) -> Option<Event> {
    let (verb, arg) = line.trim().split_once(' ')?;
    let arg = arg.trim();
    let num = || arg.parse::<u32>().ok();
    match verb {
        "key" => from_name(arg).or_else(num).map(Event::Key),
        "click" => num().map(Event::Click),
        "focus" => num().map(Event::Focus),
        "close" => num().map(Event::Close),
        _ => None,
    }
}

/// `event` as a line ([`decode_event`] reads it back).
pub fn encode_event(event: &Event) -> String {
    match event {
        Event::Key(code) => format!("key {code}"),
        Event::Click(h) => format!("click {h}"),
        Event::Focus(h) => format!("focus {h}"),
        Event::Close(h) => format!("close {h}"),
    }
}

reva_macros::machine_trait! {
    /// Strings between the VM and the model.
    pub trait IupData: CallArgs {
        /// The C string at `a`; `None` for NULL.
        fn iup_zstr(&self, a: u32) -> R<Option<String>> {
            if a == 0 { Ok(None) } else { self.ffi_zstring(a).map(Some) }
        }

        /// `s` as a C string in the VM (NULL for `None`), in a buffer kept
        /// for element `h`'s attribute `name`: valid until that attribute
        /// is read again, as IUP's own strings are until they change.
        fn iup_out(&mut self, h: u32, name: &str, s: Option<String>) -> R<u32> {
            let Some(s) = s else { return Ok(0) };
            let len = s.len() as u32 + 1;
            let key = (h, name.to_string());
            let p = match self.ui_bufs.get(&key) {
                Some(&(p, cap)) if cap >= len => p,
                old => {
                    if let Some(&(p, _)) = old {
                        self.heap.free(p);
                    }
                    let cap = len.max(64);
                    let p = self.heap.alloc(cap).ok_or(Fault::OutOfSpace("heap"))?;
                    self.ui_bufs.insert(key, (p, cap));
                    p
                }
            };
            self.mem.write_bytes(p, s.as_bytes())?;
            self.mem.write_u8(p + len - 1, 0)?;
            Ok(p)
        }
    }
}
