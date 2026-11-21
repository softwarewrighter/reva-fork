//! Answering the `ui` host calls.

use reva_hostcall::hostcall::{HostAnswer, HostCall, HostReply};
use reva_ui_layout::geom::Size;
use reva_ui_layout::units::pair;
use reva_ui_paint::paint::{click, paint};

use crate::script::{UiScript, lines};

/// What the library sent with `call`, for the transcript (`ui< `).
fn sent(call: &HostCall) -> String {
    let text = |b: &Vec<u8>| String::from_utf8_lossy(b).into_owned();
    if call.op == "file" {
        let attrs: Vec<String> = call.bytes.iter().map(text).collect();
        return format!("ui< file {}\n", attrs.join(" | "));
    }
    call.bytes
        .first()
        .map(text)
        .unwrap_or_default()
        .lines()
        .map(|l| format!("ui< {l}\n"))
        .collect()
}

impl UiScript {
    /// A host that gives the events of script `text`.
    pub fn new(text: &str) -> UiScript {
        UiScript {
            lines: lines(text),
            ..UiScript::default()
        }
    }

    /// The answer to `call`, and the transcript of it: the changes the
    /// library sent (`ui< `), the screens asked for (`ui| `), the event
    /// given (`ui> `).
    pub fn answer(&mut self, call: &HostCall) -> (HostAnswer, String) {
        if call.op == "wait" {
            self.mirror.apply_lines(&String::from_utf8_lossy(
                call.bytes.first().map_or(&[][..], |b| b),
            ));
        }
        let mut log = sent(call) + &self.screens();
        let Some(line) = self.lines.pop_front() else {
            return (
                HostAnswer::Unavailable("the ui script has no more events".into()),
                log,
            );
        };
        log.push_str(&format!("ui> {line}\n"));
        let answer = match (call.op, line.split_once(' ')) {
            ("file", Some(("file", path))) => {
                HostAnswer::Reply(HostReply::Bytes(path.as_bytes().to_vec()))
            }
            ("file", _) if line == "cancel" => HostAnswer::Reply(HostReply::Int(-1)),
            ("wait", _) => HostAnswer::Reply(HostReply::Bytes(line.into_bytes())),
            _ => HostAnswer::Unavailable(format!(
                "the ui script has {line:?} where the program asks for {}",
                call.op
            )),
        };
        (answer, log)
    }

    /// `tap COL ROW`: a click there on the 80 by 24 screen as it is now,
    /// as the events it gives (as in the page and the terminal), put next.
    fn tap(&mut self, at: &str) -> String {
        let n: Vec<u16> = at
            .split_whitespace()
            .filter_map(|n| n.parse().ok())
            .collect();
        let (col, row) = (
            n.first().copied().unwrap_or(0),
            n.get(1).copied().unwrap_or(0),
        );
        let screen = Size { cols: 80, rows: 24 };
        paint(&mut self.mirror, screen);
        for line in click(&self.mirror, screen, col, row).into_iter().rev() {
            self.lines.push_front(line);
        }
        format!("ui| tap {col} {row}\n")
    }

    /// `screen`, or `screen 100x30` (80 by 24 cells by default): the
    /// screen painted.
    fn screen(&mut self, size: &str) -> String {
        let (cols, rows) = pair(size.trim());
        let size = Size {
            cols: cols.unwrap_or(80).min(500) as u16,
            rows: rows.unwrap_or(24).min(200) as u16,
        };
        let mut out = format!("ui| screen {}x{}\n", size.cols, size.rows);
        out.extend(
            paint(&mut self.mirror, size)
                .to_text()
                .lines()
                .map(|l| format!("ui| {l}\n")),
        );
        out
    }

    /// The screens and taps the script has next.
    fn screens(&mut self) -> String {
        let mut out = String::new();
        while let Some(line) = self
            .lines
            .front()
            .filter(|l| l.starts_with("screen") || l.starts_with("tap "))
            .cloned()
        {
            self.lines.pop_front();
            out += &match line.strip_prefix("tap ") {
                Some(at) => self.tap(at),
                None => self.screen(&line["screen".len()..]),
            };
        }
        out
    }
}
