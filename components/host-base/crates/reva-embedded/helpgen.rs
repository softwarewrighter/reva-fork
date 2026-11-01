//! The help index (a part of build.rs): every `def:` entry of
//! `src/help.txt` and of `lib/`'s files (after their `|||` line), as one
//! record per word for `lib/helper`'s `help`:
//!
//! ```text
//! \x01dup
//! dup  ( a -- a a )
//!   context: ~   source: src/reva.f
//!   Duplicate the top stack item
//! ```
//!
//! Entries of one name are merged into one record; the removed words
//! (`orig: REMOVED`) and the German descriptions (`de desc:`) are left out.

/// One documented word.
#[derive(Default)]
pub struct Entry {
    pub name: String,
    pub source: String,
    pub fields: Vec<(String, String)>,
    pub desc: Vec<String>,
}

/// A description, as genhelp.f reads it: `val`'s first character is the
/// delimiter, and the text runs to its next occurrence, on this line or a
/// later one (`hex dump.=` ends one).  Its lines, one tab of indent off.
fn block<'a>(lines: &mut impl Iterator<Item = &'a str>, val: &str) -> Vec<String> {
    let mut chars = val.chars();
    let Some(delim) = chars.next() else {
        return Vec::new();
    };
    let mut text = chars.as_str().to_string();
    while !text.contains(delim) {
        let Some(line) = lines.next() else { break };
        text.push('\n');
        text.push_str(line);
    }
    let text = text.split(delim).next().unwrap_or_default();
    let tidy = |l: &str| {
        l.strip_prefix('\t')
            .unwrap_or(l.trim_start())
            .replace('\t', "    ")
            .trim_end()
            .to_string()
    };
    let mut out: Vec<String> = text.split('\n').map(tidy).collect();
    while out.first().is_some_and(|l| l.is_empty()) {
        out.remove(0);
    }
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    out
}

/// The entries of `text`, from `source`; `docs` says whether the whole
/// text is documentation (help.txt) or only what follows `|||` (lib/).
pub fn entries(text: &str, source: &str, mut docs: bool) -> Vec<Entry> {
    let (mut out, mut cur, mut src) = (Vec::new(), None::<Entry>, source.to_string());
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if !docs {
            docs = line.trim() == "|||";
            continue;
        }
        let Some((key, val)) = line.split_once(':') else {
            continue;
        };
        let val = val.trim();
        match key {
            "orig" => src = val.to_string(),
            "def" => {
                out.extend(cur.take());
                cur = Some(Entry {
                    name: val.to_string(),
                    source: src.clone(),
                    ..Entry::default()
                });
            }
            "desc" => cur.iter_mut().for_each(|e| e.desc = block(&mut lines, val)),
            k if k.ends_with(" desc") => drop(block(&mut lines, val)),
            "stack" | "ctx" | "ver" | "os" | "related" | "alias" => cur
                .iter_mut()
                .for_each(|e| e.fields.push((key.to_string(), val.to_string()))),
            _ => {}
        }
    }
    out.extend(cur);
    out.retain(|e| e.source != "REMOVED" && !e.name.is_empty());
    out
}

/// One entry as `help` shows it.
pub fn text(e: &Entry) -> String {
    let field = |k: &str| {
        e.fields
            .iter()
            .find(|f| f.0 == k)
            .map(|f| f.1.as_str())
            .filter(|v| !v.is_empty())
    };
    let mut s = match field("stack") {
        Some(st) => format!("{}  ( {st} )\n", e.name),
        None => format!("{}\n", e.name),
    };
    let mut meta = vec![
        format!("context: {}", field("ctx").unwrap_or("~")),
        format!("source: {}", e.source),
    ];
    meta.extend(field("ver").map(|v| format!("version: {v}")));
    meta.extend(field("os").map(|v| format!("os: {v}")));
    s.push_str(&format!("  {}\n", meta.join("   ")));
    for line in &e.desc {
        s.push_str(&format!("  {line}\n").replace("  \n", "\n"));
    }
    for (k, label) in [("alias", "alias"), ("related", "see also")] {
        if let Some(v) = field(k) {
            s.push_str(&format!("  {label}: {v}\n"));
        }
    }
    s
}

/// The index: a record per name, in the order names first appear.
pub fn index(entries: &[Entry]) -> String {
    let mut names: Vec<&str> = Vec::new();
    for e in entries {
        if !names.contains(&e.name.as_str()) {
            names.push(&e.name);
        }
    }
    let mut out = String::new();
    for name in names {
        out.push('\u{1}');
        out.push_str(name);
        out.push('\n');
        let texts: Vec<String> = entries
            .iter()
            .filter(|e| e.name == name)
            .map(text)
            .collect();
        out.push_str(&texts.join("\n"));
    }
    out
}
