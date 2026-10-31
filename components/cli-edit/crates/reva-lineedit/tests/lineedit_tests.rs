use reva_history::history::History;
use reva_keys::decoder::Decoder;
use reva_keys::key::Key;
use reva_lineedit::editor::{Editor, Outcome};
use reva_lineedit::view::visible_width;

fn keys(bytes: &[u8]) -> Vec<Key> {
    let mut d = Decoder::default();
    bytes.iter().filter_map(|&b| d.push(b)).collect()
}

fn typed(e: &mut Editor, h: &History, bytes: &[u8]) -> Vec<Outcome> {
    keys(bytes)
        .into_iter()
        .map(|k| e.key(k, h))
        .filter(|o| *o != Outcome::Edit)
        .collect()
}

fn hist(lines: &[&str]) -> History {
    let mut h = History::in_memory();
    for l in lines {
        h.add(l);
    }
    h
}

#[test]
fn decodes_keys() {
    use Key::*;
    assert_eq!(
        keys(b"a\x01\x7f\r\n\x1b[A\x1b[D\x1bOH\x1b[3~\x1b[1;5C\x1bb\x1b\x7f\x1b[200~"),
        [
            Char('a'),
            Ctrl('a'),
            Backspace,
            Enter,
            Up,
            Left,
            Home,
            Delete,
            WordRight,
            Alt('b'),
            AltBackspace,
            Ignored
        ]
    );
    assert_eq!(keys("é€".as_bytes()), [Char('é'), Char('€')]);
}

#[test]
fn edits() {
    let h = History::in_memory();
    let mut e = Editor::default();
    typed(&mut e, &h, b"dup swap drop\x1bb\x0b\x01\x19");
    assert_eq!(e.text(), "dropdup swap ");
    typed(&mut e, &h, b"\x05\x17");
    assert_eq!(e.text(), "dropdup ");
    typed(&mut e, &h, b"\x15ab\x14");
    assert_eq!(e.text(), "ba");
    assert_eq!(typed(&mut e, &h, b"\r"), [Outcome::Line("ba".into())]);
}

#[test]
fn ctrl_d_and_c() {
    let h = History::in_memory();
    let mut e = Editor::default();
    assert_eq!(typed(&mut e, &h, b"ab\x02\x04"), []);
    assert_eq!(e.text(), "a");
    assert_eq!(typed(&mut e, &h, b"\x03"), [Outcome::Cancel]);
    e.reset();
    assert_eq!(typed(&mut e, &h, b"\x04"), [Outcome::Eof]);
}

#[test]
fn history_recall_keeps_draft() {
    let h = hist(&["1 .", "2 ."]);
    let mut e = Editor::default();
    typed(&mut e, &h, b"dr\x1b[A");
    assert_eq!(e.text(), "2 .");
    typed(&mut e, &h, b"\x10\x10");
    assert_eq!(e.text(), "1 .");
    typed(&mut e, &h, b"\x1b[B\x0e");
    assert_eq!(e.text(), "dr");
}

#[test]
fn reverse_search() {
    let h = hist(&["1 2 + .", ": sq dup * ;", "7 sq ."]);
    let mut e = Editor::default();
    typed(&mut e, &h, b"xy\x12sq");
    assert_eq!(e.text(), "7 sq .");
    assert!(e.render("ok> ", 80).contains("(reverse-i-search)`sq': "));
    typed(&mut e, &h, b"\x12");
    assert_eq!(e.text(), ": sq dup * ;");
    typed(&mut e, &h, b"\x12");
    assert!(e.render("ok> ", 80).contains("failed"));
    typed(&mut e, &h, b"\x07");
    assert_eq!(e.text(), "xy");
    // Ctrl-R Ctrl-R repeats the last search; Ctrl-E keeps the match.
    typed(&mut e, &h, b"\x12\x12\x05");
    assert_eq!(e.text(), "7 sq .");
    assert_eq!(e.render("ok> ", 80), "\rok> 7 sq .\x1b[K\r\x1b[10C");
    assert_eq!(
        typed(&mut e, &h, b"\x12+\r"),
        [Outcome::Line("1 2 + .".into())]
    );
}

#[test]
fn scrolls_long_lines() {
    let h = History::in_memory();
    let mut e = Editor::default();
    typed(&mut e, &h, "x".repeat(50).as_bytes());
    let r = e.render("ok> ", 20);
    assert_eq!(r, format!("\rok> {}\x1b[K\r\x1b[19C", "x".repeat(15)));
    typed(&mut e, &h, b"\x01");
    assert!(e.render("ok> ", 20).ends_with("\r\x1b[4C"));
    assert_eq!(visible_width("\x1b[1mok>\x1b[0m "), 4);
}
