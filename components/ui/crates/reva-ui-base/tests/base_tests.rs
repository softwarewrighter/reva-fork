use reva_ui_base::change::Change;
use reva_ui_base::codec::{decode, encode, escape, unescape};
use reva_ui_base::keys::{CTRL, SHIFT, code, from_name, name, printable};
use reva_ui_base::kind::{Kind, Role, from_name as kind_named, name as kind_name, role};

#[test]
fn kinds_by_iup_class_name() {
    assert_eq!(kind_name(Kind::MultiLine), "multiline");
    assert_eq!(kind_named("vbox"), Some(Kind::Vbox));
    assert_eq!(kind_named("window"), None);
    assert_eq!(role(Kind::Dialog), Role::Container);
    assert_eq!(role(Kind::Item), Role::MenuEntry);
    assert_eq!(role(Kind::Timer), Role::Other);
}

#[test]
fn key_names_as_tui_uses_them() {
    for n in [
        "K_F1", "K_F6", "K_F12", "K_sF6", "K_sF11", "K_cS", "K_cF6", "K_cF9", "K_a", "K_Z", "K_0",
        "K_SP", "K_CR",
    ] {
        let c = from_name(n).unwrap_or_else(|| panic!("{n}"));
        assert_eq!(name(c).as_deref(), Some(n), "{n} -> {c:#x}");
    }
    assert_eq!(from_name("K_F6"), Some(0xFFC3));
    assert_eq!(from_name("K_sF6"), Some(0xFFC3 | SHIFT));
    assert_eq!(from_name("K_cS"), Some('S' as u32 | CTRL));
    assert_eq!(from_name("K_xF6"), None);
    assert_eq!(from_name("F6"), None);
}

#[test]
fn key_codes_from_a_renderer() {
    assert_eq!(
        name(code(0xFFC3, true, false, false)).as_deref(),
        Some("K_sF6")
    );
    assert_eq!(
        name(code('s' as u32, false, true, false)).as_deref(),
        Some("K_cS")
    );
    // Shift is part of a character, not a modifier.
    assert_eq!(code('A' as u32, true, false, false), 'A' as u32);
    assert_eq!(printable('A' as u32), Some('A'));
    assert_eq!(printable('é' as u32), Some('é'));
    assert_eq!(printable('a' as u32 | CTRL), None);
    assert_eq!(printable(0xFFC3), None);
    assert_eq!(printable(13), None);
}

#[test]
fn changes_survive_their_text_form() {
    let all = [
        Change::Created {
            handle: 1,
            kind: Kind::Dialog,
        },
        Change::Appended {
            parent: 1,
            child: 2,
        },
        Change::Detached { child: 2 },
        Change::Destroyed { handle: 2 },
        Change::Attr {
            handle: 3,
            name: "VALUE".into(),
            value: Some("a b\\c\nd\te\r".into()),
        },
        Change::Attr {
            handle: 3,
            name: "TITLE".into(),
            value: Some(String::new()),
        },
        Change::Attr {
            handle: 3,
            name: "TIP".into(),
            value: None,
        },
        Change::Shown {
            handle: 1,
            x: -1,
            y: 20,
            modal: true,
        },
        Change::Hidden { handle: 1 },
        Change::Focus { handle: 3 },
    ];
    for c in all {
        let line = encode(&c);
        assert!(!line.contains('\n'), "{line}");
        assert_eq!(decode(&line), Some(c), "{line}");
    }
    assert_eq!(
        encode(&Change::Attr {
            handle: 3,
            name: "VALUE".into(),
            value: Some("a b".into())
        }),
        "attr 3 VALUE =a\\sb"
    );
    assert_eq!(decode("created 1 window"), None);
    assert_eq!(decode("shown 1 2"), None);
    assert_eq!(decode("attr 1 X =bad\\q"), None);
}

#[test]
fn escaping() {
    assert_eq!(escape("a b\n"), "a\\sb\\n");
    assert_eq!(unescape("a\\sb\\n").as_deref(), Some("a b\n"));
    assert_eq!(unescape("trailing\\"), None);
}
