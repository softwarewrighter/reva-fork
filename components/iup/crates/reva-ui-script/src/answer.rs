//! Answering the `ui` host calls.

use reva_hostcall::hostcall::{HostAnswer, HostCall, HostReply};

use crate::script::{UiScript, lines};

impl UiScript {
    /// A host that gives the events of script `text`.
    pub fn new(text: &str) -> UiScript {
        UiScript { lines: lines(text) }
    }

    /// The answer to `call`, and the transcript of it: the changes the
    /// library sent (`ui< `), the event given (`ui> `).
    pub fn answer(&mut self, call: &HostCall) -> (HostAnswer, String) {
        let mut log: String = call.bytes.first().map_or(String::new(), |b| {
            let sent = String::from_utf8_lossy(b);
            sent.lines().map(|l| format!("ui< {l}\n")).collect()
        });
        if call.op == "file" {
            log = format!(
                "ui< file {}\n",
                call.bytes
                    .iter()
                    .map(|b| String::from_utf8_lossy(b))
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
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
}
