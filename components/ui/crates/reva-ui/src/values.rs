//! Attributes (`IupStoreAttribute`, `IupGetAttribute`).  A text element's
//! VALUE, CARET, SELECTION and the like are read from and applied to its
//! edited text; every other attribute is the tree's.

use reva_ui_base::change::{Change, Handle};
use reva_ui_text::attr::{format_pos, format_range, parse_pos, parse_range};
use reva_ui_text::pos::{line_bounds, line_col, line_count};
use reva_ui_text::text::Text;
use reva_ui_tree::tree::normal;

use crate::ui::Ui;

/// Two numbers `"a:b"` (SELECTIONPOS), or one (CARETPOS) for both.
fn offsets(t: &Text, s: &str) -> Option<(usize, usize)> {
    let n = |s: &str| s.trim().parse::<usize>().ok();
    match s.trim() {
        "NONE" => Some((t.caret, t.caret)),
        "ALL" => Some((0, t.chars.len())),
        s => match s.split_once(':') {
            Some((a, b)) => Some((n(a)?, n(b)?)),
            None => n(s).map(|p| (p, p)),
        },
    }
}

impl Ui {
    /// Set attribute `name` of `handle` (`None` removes it).  False if
    /// there is no such element or the value makes no sense.
    pub fn set(&mut self, handle: Handle, name: &str, value: Option<&str>) -> bool {
        let name = normal(name);
        match (self.texts.get_mut(&handle), value) {
            (Some(t), Some(v)) => match set_text(t, &name, v) {
                Some(done) => {
                    self.note_text(handle);
                    done
                }
                None => self.tree.set(handle, &name, value),
            },
            _ => self.tree.set(handle, &name, value),
        }
    }

    /// Attribute `name` of `handle` as IUP reads it.
    pub fn get(&self, handle: Handle, name: &str) -> Option<String> {
        let name = normal(name);
        match self.texts.get(&handle) {
            Some(t) => get_text(t, &name).unwrap_or_else(|| self.tree.attr(handle, &name)),
            None => self.tree.attr(handle, &name),
        }
    }

    /// Record a text element's VALUE, CARET and SELECTION as changes.
    pub(crate) fn note_text(&mut self, handle: Handle) {
        let Some(t) = self.texts.get(&handle) else {
            return;
        };
        for name in ["VALUE", "CARET", "SELECTION"] {
            let value = get_text(t, name).flatten();
            self.tree.changes.push(Change::Attr {
                handle,
                name: name.to_string(),
                value,
            });
        }
    }
}

/// Apply text attribute `name`: `Some(applied)`, or `None` if `name` is
/// not one of the text's own.
fn set_text(t: &mut Text, name: &str, v: &str) -> Option<bool> {
    let range = match name {
        "VALUE" => {
            *t = Text {
                caret: 0,
                ..Text::new(v, t.multiline)
            };
            return Some(true);
        }
        "CARET" => parse_pos(t, v).map(|p| (p, p)),
        "SELECTION" => parse_range(t, v),
        "CARETPOS" | "SELECTIONPOS" => offsets(t, v),
        "SELECTEDTEXT" if t.selection().is_none() => return Some(false),
        "SELECTEDTEXT" | "INSERT" => {
            t.insert(v);
            return Some(true);
        }
        "APPEND" => {
            let sep = if t.multiline && !t.chars.is_empty() {
                "\n"
            } else {
                ""
            };
            t.select(t.chars.len(), t.chars.len());
            t.insert(&format!("{sep}{v}"));
            return Some(true);
        }
        _ => return None,
    };
    Some(range.map(|(a, b)| t.select(a, b)).is_some())
}

/// Text attribute `name`: `Some(value)`, or `None` if `name` is not one of
/// the text's own.
fn get_text(t: &Text, name: &str) -> Option<Option<String>> {
    let sel = t.selection();
    let line = line_col(&t.chars, t.caret).0;
    Some(match name {
        "VALUE" => Some(t.chars.iter().collect()),
        "CARET" => Some(format_pos(t, t.caret)),
        "CARETPOS" => Some(t.caret.to_string()),
        "SELECTION" => sel.map(|r| format_range(t, r)),
        "SELECTIONPOS" => sel.map(|(a, b)| format!("{a}:{b}")),
        "SELECTEDTEXT" => sel.map(|(a, b)| t.chars[a..b].iter().collect()),
        "COUNT" => Some(t.chars.len().to_string()),
        "LINECOUNT" => Some(line_count(&t.chars).to_string()),
        "LINEVALUE" => Some({
            let (a, b) = line_bounds(&t.chars, line);
            t.chars[a..b].iter().collect()
        }),
        _ => return None,
    })
}
