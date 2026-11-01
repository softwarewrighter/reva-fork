//! Answering the `ui` host calls.

use reva_hostcall::hostcall::{HostAnswer, HostCall, HostReply};
use reva_ui_layout::geom::Size;
use reva_ui_layout::units::pair;
use reva_ui_paint::paint::paint;

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

    /// The screens the script asks for next (`screen`, or `screen 100x30`;
    /// 80 by 24 cells by default), painted.
    fn screens(&mut self) -> String {
        let mut out = String::new();
        while let Some(rest) = self
            .lines
            .front()
            .and_then(|l| l.strip_prefix("screen"))
            .map(str::to_string)
        {
            self.lines.pop_front();
            let (cols, rows) = pair(rest.trim());
            let size = Size {
                cols: cols.unwrap_or(80).min(500) as u16,
                rows: rows.unwrap_or(24).min(200) as u16,
            };
            out.push_str(&format!("ui| screen {}x{}\n", size.cols, size.rows));
            out.extend(
                paint(&self.mirror, size)
                    .to_text()
                    .lines()
                    .map(|l| format!("ui| {l}\n")),
            );
        }
        out
    }
}
