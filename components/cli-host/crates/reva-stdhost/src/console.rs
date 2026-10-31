//! Output and the terminal.

use std::io::Write;

use crate::host::StdHost;
use reva_term::reader::Input;
use reva_vm::host::Console;

impl Console for StdHost {
    fn write(&mut self, bytes: &[u8]) {
        match &mut self.capture {
            Some(c) => c.extend_from_slice(bytes),
            None => {
                let _ = self.out.write_all(bytes);
                match bytes.iter().rposition(|&b| b == b'\n') {
                    Some(i) => {
                        self.tail.clear();
                        self.tail.extend_from_slice(&bytes[i + 1..]);
                    }
                    None => self.tail.extend_from_slice(bytes),
                }
                if self.tail.len() > 256 {
                    self.tail.drain(..self.tail.len() - 256);
                }
            }
        }
    }

    fn color(&mut self) -> bool {
        self.color
    }

    /// VM notes go to stderr, after what Forth printed so far.
    fn diag(&mut self, line: &str) {
        let _ = self.out.flush();
        eprint!("{line}");
    }

    fn read_input_blocking(&mut self) -> Option<Vec<u8>> {
        if self.capture.is_some() {
            return None;
        }
        match self.read_line() {
            Input::Line(l) => Some(l.into_bytes()),
            Input::Eof => Some(Vec::new()),
        }
    }
}
