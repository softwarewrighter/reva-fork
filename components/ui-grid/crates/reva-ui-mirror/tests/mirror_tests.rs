use reva_ui_base::kind::Kind;
use reva_ui_mirror::mirror::{Mirror, Shown};

#[test]
fn the_change_stream_rebuilds_the_model() {
    let mut m = Mirror::default();
    let lines = "created 1 dialog\ncreated 2 vbox\ncreated 3 multiline\nappended 2 3\nappended 1 2\n\
                 attr 3 VALUE =a\\sb\nnot a change\nshown 1 -1 -1 modeless\nfocus 3";
    assert_eq!(m.apply_lines(lines), 8, "the bad line skipped");
    assert_eq!(
        m.tree.node(3).map(|n| (n.kind, n.parent)),
        Some((Kind::MultiLine, Some(2)))
    );
    assert_eq!(m.tree.attr(3, "VALUE").as_deref(), Some("a b"));
    assert_eq!(
        m.shown,
        [Shown {
            handle: 1,
            x: -1,
            y: -1,
            modal: false
        }]
    );
    assert_eq!((m.top(), m.focus), (Some(1), Some(3)));
    assert!(m.tree.changes.is_empty(), "a copy records nothing");
    m.apply_lines("shown 4 0 0 modal\nhidden 1\nattr 3 VALUE");
    assert_eq!(m.top(), Some(4));
    assert_eq!(m.tree.attr(3, "VALUE"), None);
    m.apply_lines("detached 3\ndestroyed 3\ndestroyed 1");
    assert!(m.tree.node(3).is_none() && m.focus.is_none());
    assert!(
        m.tree.node(2).is_none(),
        "what was in the dialog went with it"
    );
}

#[test]
fn handles_come_from_the_vm() {
    let mut m = Mirror::default();
    m.apply_lines("created 5 label\nattr 5 TITLE =far");
    assert_eq!(m.tree.node(5).map(|n| n.kind), Some(Kind::Label));
    assert!(m.tree.node(4).is_none());
    m.apply_lines("created 0 label");
    assert_eq!(m.tree.nodes.len(), 5, "0 is NULL");
}
