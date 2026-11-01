use reva_ui_base::change::Handle;
use reva_ui_base::kind::Kind;
use reva_ui_layout::geom::{Rect, Size};
use reva_ui_layout::natural::{natural, title_size};
use reva_ui_layout::place::layout;
use reva_ui_tree::tree::Tree;

fn titled(t: &mut Tree, kind: Kind, title: &str) -> Handle {
    let h = t.create(kind);
    t.set(h, "TITLE", Some(title));
    h
}

fn under(t: &mut Tree, parent: Handle, kids: &[Handle]) {
    for k in kids {
        assert!(t.append(parent, *k));
    }
}

fn rect(col: u16, row: u16, cols: u16, rows: u16) -> Rect {
    Rect {
        col,
        row,
        cols,
        rows,
    }
}

fn where_is(t: &Tree, dialog: Handle, avail: Size, h: Handle) -> Rect {
    layout(t, dialog, avail)
        .into_iter()
        .find(|p| p.handle == h)
        .expect("placed")
        .rect
}

const SCREEN: Size = Size { cols: 80, rows: 24 };

#[test]
fn natural_sizes_of_controls() {
    let mut t = Tree::default();
    let label = titled(&mut t, Kind::Label, "Hello");
    let button = titled(&mut t, Kind::Button, "OK");
    let toggle = titled(&mut t, Kind::Toggle, "&Bold");
    let text = t.create(Kind::Text);
    let edit = t.create(Kind::MultiLine);
    assert_eq!(natural(&t, label), Size { cols: 5, rows: 1 });
    assert_eq!(natural(&t, button), Size { cols: 6, rows: 1 }, "[ OK ]");
    assert_eq!(natural(&t, toggle), Size { cols: 8, rows: 1 }, "[x] Bold");
    assert_eq!(natural(&t, text), Size { cols: 12, rows: 1 });
    assert_eq!(natural(&t, edit), Size { cols: 22, rows: 6 });
    t.set(edit, "SIZE", Some("400x100"));
    assert_eq!(
        natural(&t, edit),
        Size {
            cols: 100,
            rows: 13
        },
        "SIZE in quarter and eighth characters"
    );
    t.set(text, "RASTERSIZE", Some("80x"));
    assert_eq!(natural(&t, text), Size { cols: 10, rows: 1 });
    assert_eq!(title_size("a&&b\nlonger"), Size { cols: 6, rows: 2 });
}

/// A dialog over a vbox: a label, a text, and an hbox of OK, a fill, Cancel.
fn form() -> (Tree, [Handle; 7]) {
    let mut t = Tree::default();
    let (dialog, vbox, hbox) = (
        t.create(Kind::Dialog),
        t.create(Kind::Vbox),
        t.create(Kind::Hbox),
    );
    let label = titled(&mut t, Kind::Label, "Name");
    let text = t.create(Kind::Text);
    let ok = titled(&mut t, Kind::Button, "OK");
    let fill = t.create(Kind::Fill);
    let cancel = titled(&mut t, Kind::Button, "Cancel");
    under(&mut t, hbox, &[ok, fill, cancel]);
    under(&mut t, vbox, &[label, text, hbox]);
    under(&mut t, dialog, &[vbox]);
    (t, [dialog, vbox, label, text, ok, fill, cancel])
}

#[test]
fn a_form_at_its_natural_size() {
    let (t, [dialog, vbox, label, text, ok, fill, cancel]) = form();
    assert_eq!(natural(&t, dialog), Size { cols: 18, rows: 5 });
    let placed = layout(&t, dialog, SCREEN);
    assert_eq!(placed[0].rect, rect(0, 0, 18, 5), "the dialog first");
    for (h, r) in [
        (vbox, rect(1, 1, 16, 3)),
        (label, rect(1, 1, 4, 1)),
        (text, rect(1, 2, 12, 1)),
        (ok, rect(1, 3, 6, 1)),
        (fill, rect(7, 3, 0, 0)),
        (cancel, rect(7, 3, 10, 1)),
    ] {
        assert_eq!(where_is(&t, dialog, SCREEN, h), r, "handle {h}");
    }
}

#[test]
fn a_fill_takes_the_space_left() {
    let (mut t, [dialog, _, label, _, ok, fill, cancel]) = form();
    t.set(dialog, "SIZE", Some("120x"));
    assert_eq!(where_is(&t, dialog, SCREEN, dialog), rect(0, 0, 30, 5));
    assert_eq!(where_is(&t, dialog, SCREEN, ok), rect(1, 3, 6, 1));
    assert_eq!(where_is(&t, dialog, SCREEN, fill), rect(7, 3, 12, 0));
    assert_eq!(
        where_is(&t, dialog, SCREEN, cancel),
        rect(19, 3, 10, 1),
        "pushed right"
    );
    assert_eq!(where_is(&t, dialog, SCREEN, label), rect(1, 1, 4, 1));
    let vbox = t.node(label).unwrap().parent.unwrap();
    t.set(vbox, "ALIGNMENT", Some("ACENTER"));
    assert_eq!(
        where_is(&t, dialog, SCREEN, label),
        rect(13, 1, 4, 1),
        "centred in 28"
    );
    assert_eq!(
        where_is(&t, dialog, Size { cols: 20, rows: 4 }, dialog),
        rect(0, 0, 20, 4),
        "at most the window"
    );
}

#[test]
fn tui_panes_share_the_height() {
    let mut t = Tree::default();
    let (dialog, vbox) = (t.create(Kind::Dialog), t.create(Kind::Vbox));
    let (text, res) = (t.create(Kind::MultiLine), t.create(Kind::MultiLine));
    for m in [text, res] {
        t.set(m, "SIZE", Some("400x100"));
        t.set(m, "EXPAND", Some("YES"));
    }
    t.set(vbox, "MARGIN", Some("8x16"));
    t.set(vbox, "GAP", Some("16"));
    under(&mut t, vbox, &[text, res]);
    under(&mut t, dialog, &[vbox]);
    assert_eq!(
        natural(&t, dialog),
        Size {
            cols: 104,
            rows: 31
        }
    );
    t.set(dialog, "SIZE", Some("x400"));
    let big = Size {
        cols: 200,
        rows: 60,
    };
    assert_eq!(where_is(&t, dialog, big, dialog), rect(0, 0, 104, 50));
    assert_eq!(where_is(&t, dialog, big, vbox), rect(1, 1, 102, 48));
    assert_eq!(where_is(&t, dialog, big, text), rect(2, 2, 100, 23));
    assert_eq!(where_is(&t, dialog, big, res), rect(2, 26, 100, 22));
}

#[test]
fn frames_and_huge_sizes() {
    let mut t = Tree::default();
    let frame = titled(&mut t, Kind::Frame, "Options");
    let toggle = titled(&mut t, Kind::Toggle, "Bold");
    under(&mut t, frame, &[toggle]);
    assert_eq!(
        natural(&t, frame),
        Size { cols: 11, rows: 3 },
        "wide enough for its title"
    );
    let (dialog, vbox) = (t.create(Kind::Dialog), t.create(Kind::Vbox));
    let (a, b) = (t.create(Kind::Label), t.create(Kind::Label));
    for l in [a, b] {
        t.set(l, "SIZE", Some("999999x999999"));
    }
    under(&mut t, vbox, &[a, b, frame]);
    under(&mut t, dialog, &[vbox]);
    assert_eq!(natural(&t, dialog).rows, u16::MAX);
    assert_eq!(where_is(&t, dialog, SCREEN, dialog), rect(0, 0, 80, 24));
}
