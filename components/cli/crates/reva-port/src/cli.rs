//! `reva --port-check [--fix DIR] FILE...`: the report, and with `--fix`
//! the migrated copies in DIR (never over a source file) plus
//! DIR/port-report.txt.  Exit status: 0 when nothing is left to do, 1 when
//! findings remain, 2 for a usage or file error.

use crate::migrate::{Migrated, migrate};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// What `reva --port-check` with no files prints.
pub const USAGE: &str = "usage: reva --port-check [--fix DIR] FILE...
  Reports what ties Forth sources to Windows (drive paths, \\ as the path
  separator, del/copy/dir through the shell, .bat files), as
  FILE:LINE:COL.  --fix DIR writes migrated copies to DIR, never over
  FILE, with the report in DIR/port-report.txt.  docs/porting-from-windows.md
";

/// Run the checker on `args` (what follows `--port-check`).
pub fn run(args: &[String]) -> ExitCode {
    let (dir, files) = match parse(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("reva --port-check: {e}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let mut text = String::new();
    let (mut left, mut fixed) = (0, 0);
    for path in &files {
        let src = match fs::read(path) {
            Ok(s) => s,
            Err(e) => return fail(&format!("{}: {e}", path.display())),
        };
        let m = migrate(&src);
        text += &report(&path.display().to_string(), &src, &m);
        fixed += m.findings.len() - m.manual();
        left += if dir.is_some() {
            m.manual()
        } else {
            m.findings.len()
        };
        if let Some(d) = &dir
            && let Err(e) = write_copy(d, path, &m.fixed)
        {
            return fail(&e);
        }
    }
    text += &summary(files.len(), fixed, left, dir.is_some());
    print!("{text}");
    if let Some(d) = &dir
        && let Err(e) = fs::write(d.join("port-report.txt"), &text)
    {
        return fail(&format!("{}: {e}", d.display()));
    }
    ExitCode::from(u8::from(left > 0))
}

/// `--fix DIR` and the files.
fn parse(args: &[String]) -> Result<(Option<PathBuf>, Vec<PathBuf>), String> {
    let mut dir = None;
    let mut files = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--fix" => dir = Some(PathBuf::from(it.next().ok_or("--fix needs a folder")?)),
            s if s.starts_with('-') => return Err(format!("unknown option {s}")),
            s => files.push(PathBuf::from(s)),
        }
    }
    if files.is_empty() {
        return Err("no files".into());
    }
    let mut names: Vec<_> = files.iter().filter_map(|f| f.file_name()).collect();
    names.sort();
    if dir.is_some() && names.windows(2).any(|w| w[0] == w[1]) {
        return Err("--fix: two files with the same name would overwrite each other's copy".into());
    }
    Ok((dir, files))
}

/// The report for one file.
fn report(name: &str, src: &[u8], m: &Migrated) -> String {
    let lines: Vec<&[u8]> = src.split(|c| *c == b'\n').collect();
    let mut out = String::new();
    for (n, f) in &m.findings {
        let line = lines.get(n - 1).copied().unwrap_or_default();
        let old = String::from_utf8_lossy(line.get(f.start..f.end).unwrap_or_default());
        out += &format!(
            "{name}:{n}:{}: {}: {}\n",
            f.start + 1,
            f.rule.name(),
            f.advice
        );
        out += &match &f.fix {
            Some(new) => format!("    fix: {old} -> {}\n", String::from_utf8_lossy(new)),
            None => format!("    by hand: {}\n", String::from_utf8_lossy(line).trim()),
        };
    }
    out
}

/// The last line of the report.
fn summary(files: usize, fixed: usize, left: usize, fixing: bool) -> String {
    match (fixing, left) {
        (_, 0) if fixed == 0 => {
            format!("port-check: {files} file(s), nothing Windows-only found\n")
        }
        (true, _) => format!(
            "port-check: {files} file(s): {fixed} fixed in the copies, {left} to do by hand\n"
        ),
        (false, _) => format!(
            "port-check: {files} file(s): {left} finding(s); --fix DIR writes copies with the safe fixes\n"
        ),
    }
}

/// Write `path`'s migrated copy into `dir`, never over a source file.
fn write_copy(dir: &Path, path: &Path, fixed: &[u8]) -> Result<(), String> {
    let name = path
        .file_name()
        .ok_or(format!("{}: not a file", path.display()))?;
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let out = dir.join(name);
    let same = fs::canonicalize(&out).ok() == fs::canonicalize(path).ok();
    if same {
        return Err(format!(
            "{}: --fix would write over the source; choose another folder",
            out.display()
        ));
    }
    fs::write(&out, fixed).map_err(|e| format!("{}: {e}", out.display()))
}

/// Report an error and give status 2.
fn fail(msg: &str) -> ExitCode {
    eprintln!("reva --port-check: {msg}");
    ExitCode::from(2)
}
