//! Changes as lines of text, for renderers in another process or language.
//!
//! One change per line, words separated by one space:
//!
//! ```text
//! created 3 multiline
//! appended 2 3
//! attr 3 VALUE =hello\sworld\n
//! attr 3 TIP
//! shown 1 -1 -1 modeless
//! ```
//!
//! An attribute's value follows `=`, with `\` written `\\`, space `\s`,
//! newline `\n`, carriage return `\r` and tab `\t`; no `=` means removed.

use crate::change::Change;
use crate::kind::{from_name, name};

/// `change` as one line (no newline).
pub fn encode(change: &Change) -> String {
    match change {
        Change::Created { handle, kind } => format!("created {handle} {}", name(*kind)),
        Change::Appended { parent, child } => format!("appended {parent} {child}"),
        Change::Detached { child } => format!("detached {child}"),
        Change::Destroyed { handle } => format!("destroyed {handle}"),
        Change::Attr {
            handle,
            name,
            value: None,
        } => format!("attr {handle} {name}"),
        Change::Attr {
            handle,
            name,
            value: Some(v),
        } => format!("attr {handle} {name} ={}", escape(v)),
        Change::Shown {
            handle,
            x,
            y,
            modal,
        } => {
            format!(
                "shown {handle} {x} {y} {}",
                if *modal { "modal" } else { "modeless" }
            )
        }
        Change::Hidden { handle } => format!("hidden {handle}"),
        Change::Focus { handle } => format!("focus {handle}"),
    }
}

/// The change a line of [`encode`] stands for; `None` if it is not one.
pub fn decode(line: &str) -> Option<Change> {
    let w: Vec<&str> = line.split(' ').collect();
    let n = |i: usize| w.get(i)?.parse::<i64>().ok();
    let h = |i: usize| n(i).and_then(|v| u32::try_from(v).ok());
    Some(match (w[0], w.len()) {
        ("created", 3) => Change::Created {
            handle: h(1)?,
            kind: from_name(w[2])?,
        },
        ("appended", 3) => Change::Appended {
            parent: h(1)?,
            child: h(2)?,
        },
        ("detached", 2) => Change::Detached { child: h(1)? },
        ("destroyed", 2) => Change::Destroyed { handle: h(1)? },
        ("attr", 3) => Change::Attr {
            handle: h(1)?,
            name: w[2].to_string(),
            value: None,
        },
        ("attr", 4) => Change::Attr {
            handle: h(1)?,
            name: w[2].to_string(),
            value: Some(unescape(w[3].strip_prefix('=')?)?),
        },
        ("shown", 5) => Change::Shown {
            handle: h(1)?,
            x: n(2)? as i32,
            y: n(3)? as i32,
            modal: w[4] == "modal",
        },
        ("hidden", 2) => Change::Hidden { handle: h(1)? },
        ("focus", 2) => Change::Focus { handle: h(1)? },
        _ => return None,
    })
}

/// `text` with no space, newline or tab left (see the module docs).
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ' ' => out.push_str("\\s"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

/// The text [`escape`] made `text` from; `None` for a bad escape.
pub fn unescape(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        out.push(match c {
            '\\' => match chars.next()? {
                '\\' => '\\',
                's' => ' ',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => return None,
            },
            c => c,
        });
    }
    Some(out)
}
