//! What ties a line to Windows, and the fix where one is safe.  The rules
//! look at code and string literals, never at comments.

use crate::lex::{Kind, Piece, pieces};

/// A kind of finding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// `C:\...` or `C:/...`: a drive no other system has.
    DrivePath,
    /// A string that is just `\`, given to a word that splits or joins
    /// paths: Windows reports paths with `\`, reva-fork with `/`.
    PathSeparator,
    /// `del`, `copy`, `dir`, ... run through the shell.
    WindowsCommand,
    /// A `.bat` file.
    BatFile,
}

impl Rule {
    /// The rule's name in the report.
    pub fn name(self) -> &'static str {
        match self {
            Rule::DrivePath => "drive-path",
            Rule::PathSeparator => "path-separator",
            Rule::WindowsCommand => "windows-command",
            Rule::BatFile => "bat-file",
        }
    }
}

/// A finding in one line: where (0-based byte column), what, and either a
/// replacement for `start..end` or advice for a person.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub rule: Rule,
    pub start: usize,
    pub end: usize,
    pub fix: Option<Vec<u8>>,
    pub advice: String,
}

/// Words that split or join a path at a separator (Reva's and CoSy's).
const SPLIT_WORDS: &[&[u8]] = &[b"ss", b"toksplt", b"split", b"ssc", b"cut"];
/// Words that run a shell command (Reva's `shell`, CoSy's variants).
const SHELL_WORDS: &[&[u8]] = &[b"shell", b"shell^", b"shell>", b"shell$", b"system"];
/// Windows commands, and what portable Forth uses instead.
const COMMANDS: &[(&str, &str)] = &[
    ("del", "delete ( a n -- )"),
    ("erase", "delete ( a n -- )"),
    ("copy", "slurp, then creat/write/close (or os/fs)"),
    ("xcopy", "slurp, then creat/write/close (or os/fs)"),
    ("move", "rename ( a n a n -- )"),
    ("ren", "rename ( a n a n -- )"),
    ("rename", "rename ( a n a n -- )"),
    ("dir", "os/dir (opendir/readdir), not a shell"),
    ("type", "slurp type"),
    (
        "start",
        "the host's opener for a file or URL ((open-url)), not cmd.exe",
    ),
    (
        "cd",
        "paths relative to the program (no shell current directory)",
    ),
    ("md", "os/fs mkdir"),
    ("mkdir", "os/fs mkdir"),
    ("rd", "os/fs rmdir"),
    ("rmdir", "os/fs rmdir"),
];

/// Every finding in `line`.
pub fn check_line(line: &[u8]) -> Vec<Finding> {
    let ps = pieces(line);
    let mut out = Vec::new();
    for (i, p) in ps.iter().enumerate() {
        if p.kind == Kind::Comment {
            continue;
        }
        let text = &line[p.start..p.end];
        let cmd = (p.kind == Kind::Str)
            .then(|| command(text, p, &ps[i + 1..], line))
            .flatten();
        if cmd.is_none() {
            // a drive inside a shell command goes with the command, by hand
            out.extend(drive_paths(text, p.start));
        }
        out.extend(cmd);
        out.extend(bat_file(text, p.start));
        if p.kind == Kind::Str {
            out.extend(separator(text, p, next_code(line, &ps[i + 1..])));
        }
    }
    out
}

/// The next code word after a piece, if any.
fn next_code<'a>(line: &'a [u8], rest: &[Piece]) -> Option<&'a [u8]> {
    rest.iter()
        .find(|p| p.kind == Kind::Code)
        .map(|p| &line[p.start..p.end])
}

/// Drive paths in `text` (at column `at`): fixed to `./NAME`, the file in
/// the program's folder, for a person to check.
fn drive_paths(text: &[u8], at: usize) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 2 < text.len() {
        let starts = text[i].is_ascii_alphabetic()
            && text[i + 1] == b':'
            && matches!(text[i + 2], b'\\' | b'/')
            && (i == 0 || !text[i - 1].is_ascii_alphanumeric());
        if !starts {
            i += 1;
            continue;
        }
        let end = text[i..]
            .iter()
            .position(|c| c.is_ascii_whitespace() || *c == b'"')
            .map_or(text.len(), |p| i + p);
        let path = &text[i..end];
        let name = path
            .rsplit(|c| *c == b'\\' || *c == b'/')
            .next()
            .unwrap_or(b"");
        let fix = [b"./".as_slice(), name].concat();
        let advice = format!(
            "the drive exists only on Windows: a path relative to the program, {}",
            String::from_utf8_lossy(&fix)
        );
        out.push(Finding {
            rule: Rule::DrivePath,
            start: at + i,
            end: at + end,
            fix: Some(fix),
            advice,
        });
        i = end;
    }
    out
}

/// A `\` string before a word that splits paths: becomes `/`.
fn separator(text: &[u8], p: &Piece, next: Option<&[u8]>) -> Option<Finding> {
    let sep = text == b"\\" || text == b"\\\\";
    if !sep || !next.is_some_and(|w| SPLIT_WORDS.contains(&w)) {
        return None;
    }
    let advice =
        "paths in reva-fork use / on every host (Windows reported them with \\)".to_string();
    Some(Finding {
        rule: Rule::PathSeparator,
        start: p.start,
        end: p.end,
        fix: Some(b"/".to_vec()),
        advice,
    })
}

/// A string starting with a Windows command, on a line that runs the shell.
fn command(text: &[u8], p: &Piece, rest: &[Piece], line: &[u8]) -> Option<Finding> {
    let word = text
        .split(|c| c.is_ascii_whitespace())
        .find(|w| !w.is_empty())?;
    let word = String::from_utf8_lossy(word).to_ascii_lowercase();
    let (_, instead) = COMMANDS.iter().find(|(c, _)| *c == word)?;
    let shells = rest
        .iter()
        .any(|q| q.kind == Kind::Code && SHELL_WORDS.contains(&&line[q.start..q.end]));
    shells.then(|| Finding {
        rule: Rule::WindowsCommand,
        start: p.start,
        end: p.end,
        fix: None,
        advice: format!("`{word}` is a Windows shell command; instead: {instead}"),
    })
}

/// A `.bat` file named in `text`.
fn bat_file(text: &[u8], at: usize) -> Option<Finding> {
    let lower = text.to_ascii_lowercase();
    let i = lower.windows(4).position(|w| w == b".bat")?;
    let after = lower.get(i + 4).copied();
    if after.is_some_and(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some(Finding {
        rule: Rule::BatFile,
        start: at,
        end: at + text.len(),
        fix: None,
        advice: "a Windows batch file: do its steps in Forth, or document them per OS".to_string(),
    })
}
