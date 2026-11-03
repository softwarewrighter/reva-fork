//! A whole file: its findings line by line, and its migrated copy.  The
//! copy keeps the line ends as they were (reva-fork reads CR LF files), so
//! a diff against the original shows only the fixes.

use crate::rules::{Finding, check_line};

/// What a file holds and what it becomes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Migrated {
    /// Each finding with its 1-based line number.
    pub findings: Vec<(usize, Finding)>,
    /// The file with every safe fix applied.
    pub fixed: Vec<u8>,
}

impl Migrated {
    /// Findings a person still has to act on (no automatic fix).
    pub fn manual(&self) -> usize {
        self.findings
            .iter()
            .filter(|(_, f)| f.fix.is_none())
            .count()
    }
}

/// Check `src` and build its migrated copy.
pub fn migrate(src: &[u8]) -> Migrated {
    let mut m = Migrated::default();
    let lines: Vec<&[u8]> = src.split(|c| *c == b'\n').collect();
    for (n, raw) in lines.iter().enumerate() {
        let (line, cr) = match raw.strip_suffix(b"\r") {
            Some(l) => (l, true),
            None => (*raw, false),
        };
        let found = check_line(line);
        m.fixed.extend(apply(line, &found));
        if cr {
            m.fixed.push(b'\r');
        }
        if n + 1 < lines.len() {
            m.fixed.push(b'\n');
        }
        m.findings.extend(found.into_iter().map(|f| (n + 1, f)));
    }
    m
}

/// `line` with each finding's fix put in its place.
fn apply(line: &[u8], found: &[Finding]) -> Vec<u8> {
    let mut fixes: Vec<&Finding> = found.iter().filter(|f| f.fix.is_some()).collect();
    fixes.sort_by_key(|f| f.start);
    let mut out = Vec::with_capacity(line.len());
    let mut at = 0;
    for f in fixes {
        if f.start < at {
            continue; // overlaps the previous fix
        }
        out.extend_from_slice(&line[at..f.start]);
        out.extend_from_slice(f.fix.as_deref().unwrap_or_default());
        at = f.end;
    }
    out.extend_from_slice(&line[at..]);
    out
}
