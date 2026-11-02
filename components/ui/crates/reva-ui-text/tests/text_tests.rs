use reva_ui_base::keys::{BS, CR, CTRL, DEL, DOWN, END, HOME, LEFT, SHIFT, UP};
use reva_ui_text::attr::{format_pos, format_range, parse_pos, parse_range};
use reva_ui_text::pos::{index, line_bounds, line_col, line_count, moved};
use reva_ui_text::text::{Motion, Text};

fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

fn value(t: &Text) -> String {
    t.chars.iter().collect()
}

#[test]
fn lines_and_columns() {
    let c = chars("ab\ncde\n\nf");
    assert_eq!(line_count(&c), 4);
    assert_eq!(line_count(&[]), 1);
    assert_eq!(line_col(&c, 0), (0, 0));
    assert_eq!(line_col(&c, 4), (1, 1));
    assert_eq!(line_col(&c, 7), (2, 0));
    assert_eq!(line_col(&c, 99), (3, 1));
    assert_eq!(line_bounds(&c, 1), (3, 6));
    assert_eq!(line_bounds(&c, 2), (7, 7));
    assert_eq!(line_bounds(&c, 9), (9, 9));
    assert_eq!(index(&c, 1, 1000), 6);
    assert_eq!(moved(&c, 5, Motion::Up), 2);
    assert_eq!(moved(&c, 1, Motion::Down), 4);
    assert_eq!(moved(&c, 8, Motion::Down), 9);
    assert_eq!(moved(&c, 4, Motion::LineEnd), 6);
}

#[test]
fn iup_positions() {
    let m = Text::new("ab\ncde", true);
    assert_eq!(format_pos(&m, 4), "2,2");
    assert_eq!(parse_pos(&m, "2,2"), Some(4));
    assert_eq!(parse_pos(&m, "1,1000"), Some(2));
    // gui's line-text?: SELECTION "n,1:n,1000", then SELECTEDTEXT.
    assert_eq!(parse_range(&m, "2,1:2,1000"), Some((3, 6)));
    assert_eq!(format_range(&m, (3, 6)), "2,1:2,4");
    assert_eq!(parse_range(&m, "ALL"), Some((0, 6)));
    assert_eq!(parse_range(&m, "NONE"), Some((6, 6)));
    assert_eq!(parse_range(&m, "2,1"), None);
    let s = Text::new("hello", false);
    assert_eq!(format_pos(&s, 2), "3");
    assert_eq!(parse_pos(&s, "3"), Some(2));
    assert_eq!(parse_pos(&s, "x"), None);
}

#[test]
fn editing() {
    let mut t = Text::new("hello world", false);
    t.select(0, 5);
    assert_eq!(t.selection(), Some((0, 5)));
    t.insert("bye");
    assert_eq!(
        (value(&t), t.caret, t.selection()),
        ("bye world".into(), 3, None)
    );
    t.delete(false);
    t.delete(true);
    assert_eq!(value(&t), "byworld");
    t.insert("two\nlines");
    assert_eq!(value(&t), "bytwolinesworld", "a single line drops newlines");
    t.select(2, 2);
    t.delete(false);
    t.select(0, 0);
    t.delete(false);
    assert_eq!(value(&t), "btwolinesworld");
}

#[test]
fn keys_edit() {
    let mut t = Text::new("ab\ncd", true);
    for k in [HOME | CTRL, 'x' as u32, DOWN, END, CR, 'y' as u32] {
        assert!(t.key(k), "{k:#x}");
    }
    assert_eq!(value(&t), "xab\ncd\ny");
    assert!(t.key(UP));
    assert!(t.key(HOME | SHIFT));
    assert!(t.key(LEFT | SHIFT));
    assert_eq!(
        t.selection(),
        Some((3, 5)),
        "the selection grows with Shift"
    );
    assert!(t.key(BS));
    assert_eq!(value(&t), "xabd\ny");
    assert!(t.key(END | CTRL) && t.key(DEL) && t.key(BS));
    assert_eq!(value(&t), "xabd\n");
    assert!(!t.key(0xFFC3), "F6 does not edit");
    assert!(!t.key('s' as u32 | CTRL));
    let mut s = Text::new("", false);
    assert!(!s.key(CR), "Enter does not edit a single line");
}

#[test]
fn emacs_keys() {
    let ctrl = |c: char| c as u32 | CTRL;
    let mut t = Text::new("one two\nthree", true);
    t.select(2, 2);
    assert!(t.key(ctrl('E')) && t.caret == 7, "Ctrl-E: end of the line");
    assert!(t.key(ctrl('A')) && t.caret == 0, "Ctrl-A: its start");
    assert!(t.key(ctrl('F')) && t.key(ctrl('F')) && t.key(ctrl('B')) && t.caret == 1);
    assert!(t.key(ctrl('N')) && t.caret == 9, "Ctrl-N: down");
    assert!(t.key(ctrl('P')) && t.caret == 1);
    assert!(t.key(ctrl('D')));
    assert_eq!(value(&t), "oe two\nthree");
    assert!(t.key(ctrl('K')));
    assert_eq!(value(&t), "o\nthree", "Ctrl-K kills to the end of the line");
    assert!(t.key(ctrl('K')));
    assert_eq!(value(&t), "othree", "and at its end joins the next");
    assert!(t.key(ctrl('H')));
    assert_eq!(value(&t), "three");
}

#[test]
fn undo_and_redo() {
    let ctrl = |c: char| c as u32 | CTRL;
    let mut t = Text::new("", true);
    for c in "hello world".chars() {
        t.key(c as u32);
    }
    assert!(t.key(CR));
    for c in "two".chars() {
        t.key(c as u32);
    }
    assert_eq!(value(&t), "hello world\ntwo");
    assert!(t.key(ctrl('Z')));
    assert_eq!(value(&t), "hello world\n", "typing is one step");
    assert!(t.key(ctrl('Z')));
    assert_eq!(value(&t), "hello world", "Enter is its own step");
    assert!(t.key(ctrl('Y')));
    assert_eq!(value(&t), "hello world\n", "Ctrl-Y redoes");
    t.select(0, 5);
    t.insert("bye");
    assert_eq!(value(&t), "bye world\n");
    assert!(
        !t.key(ctrl('Y')) || value(&t) == "bye world\n",
        "an edit ends the redo"
    );
    assert!(t.key(ctrl('Z')));
    assert_eq!(value(&t), "hello world\n");
    assert_eq!(t.caret, 5, "the caret goes back to where it was");
    assert!(t.key(ctrl('K')));
    assert_eq!(value(&t), "hello\n", "Ctrl-K, a step of its own");
    assert!(t.key(ctrl('Z')));
    assert_eq!(value(&t), "hello world\n");
    while t.undo() {}
    assert_eq!(value(&t), "", "back to the start");
    assert!(t.key(ctrl('Z')), "nothing left to undo: still the undo key");
    t.key(LEFT);
    assert!(t.key('x' as u32) && t.key('y' as u32));
    t.key(LEFT);
    t.key('z' as u32);
    assert!(t.undo());
    assert_eq!(value(&t), "xy", "moving the caret ends a typing step");
}
