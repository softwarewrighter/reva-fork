use reva_ui_base::change::Change;
use reva_ui_base::kind::Kind;
use reva_ui_tree::tree::Tree;

#[test]
fn append_follows_iups_rules() {
    let mut t = Tree::default();
    let (d, v, l, b) = (
        t.create(Kind::Dialog),
        t.create(Kind::Vbox),
        t.create(Kind::Label),
        t.create(Kind::Button),
    );
    assert!(t.append(v, l) && t.append(v, b));
    assert!(!t.append(l, b), "a label holds nothing");
    assert!(!t.append(v, d), "a dialog is in nothing");
    assert!(!t.append(l, l));
    assert!(t.append(d, v));
    assert!(!t.append(d, b), "a dialog holds one child");
    let h = t.create(Kind::Hbox);
    assert!(!t.append(l, h));
    assert!(t.append(v, h));
    assert!(!t.append(h, v), "no cycles");
    assert!(t.append(h, b), "appending moves");
    assert_eq!(t.node(v).unwrap().children, vec![l, h]);
    assert_eq!(t.node(b).unwrap().parent, Some(h));
}

#[test]
fn attributes_inherit_and_default() {
    let mut t = Tree::default();
    let (d, v, m) = (
        t.create(Kind::Dialog),
        t.create(Kind::Vbox),
        t.create(Kind::MultiLine),
    );
    t.append(d, v);
    t.append(v, m);
    t.set(d, "font", Some("Courier, 12"));
    t.set(d, "TITLE", Some("Tui"));
    assert_eq!(
        t.attr(m, "FONT").as_deref(),
        Some("Courier, 12"),
        "FONT is inherited"
    );
    assert_eq!(t.attr(m, "title"), None, "TITLE is not");
    assert_eq!(t.attr(m, "EXPAND").as_deref(), Some("NO"));
    assert_eq!(t.attr(v, "EXPAND").as_deref(), Some("YES"));
    t.set(m, "EXPAND", Some("YES"));
    assert_eq!(t.attr(m, "EXPAND").as_deref(), Some("YES"));
    t.set(m, "EXPAND", None);
    assert_eq!(t.attr(m, "EXPAND").as_deref(), Some("NO"));
    assert!(!t.set(99, "X", Some("1")));
}

#[test]
fn relations_and_callbacks() {
    let mut t = Tree::default();
    let (d, v, a, b) = (
        t.create(Kind::Dialog),
        t.create(Kind::Vbox),
        t.create(Kind::Text),
        t.create(Kind::Button),
    );
    t.append(d, v);
    t.append(v, a);
    t.append(v, b);
    assert_eq!(t.next_child(v, None), Some(a));
    assert_eq!(t.next_child(v, Some(a)), Some(b));
    assert_eq!(t.next_child(v, Some(b)), None);
    assert_eq!(t.dialog_path(b), vec![b, v, d]);
    t.set(b, "NAME", Some("ok"));
    assert_eq!(t.named(d, "ok"), Some(b));
    assert_eq!(t.named(d, "no"), None);
    assert_eq!(t.set_callback(d, "K_F6", Some(100)), None);
    assert_eq!(t.set_callback(a, "K_F6", Some(200)), None);
    assert_eq!(t.callback(a, "K_F6"), Some((a, 200)));
    assert_eq!(t.callback(b, "K_F6"), Some((d, 100)), "found on a parent");
    assert_eq!(t.set_callback(a, "K_F6", None), Some(200));
    assert_eq!(t.callback(a, "K_F6"), Some((d, 100)));
}

#[test]
fn destroy_and_the_change_stream() {
    let mut t = Tree::default();
    let (d, v, l) = (
        t.create(Kind::Dialog),
        t.create(Kind::Vbox),
        t.create(Kind::Label),
    );
    t.append(d, v);
    t.append(v, l);
    t.set(l, "TITLE", Some("hi"));
    t.take_changes();
    t.destroy(v);
    assert!(t.node(v).is_none() && t.node(l).is_none());
    assert!(t.node(d).unwrap().children.is_empty());
    assert_eq!(
        t.take_changes(),
        vec![
            Change::Detached { child: l },
            Change::Destroyed { handle: l },
            Change::Detached { child: v },
            Change::Destroyed { handle: v },
        ]
    );
    assert!(t.take_changes().is_empty());
    assert_eq!(t.create(Kind::Label), 4, "handles are not reused");
}
