use reva_ui::ui::{CLOSE, Call, DEFAULT, Event, IGNORE, Ui};
use reva_ui_base::change::{Change, Handle};
use reva_ui_base::codec::{decode, encode};
use reva_ui_base::keys::{CR, SHIFT, TAB, from_name};
use reva_ui_base::kind::Kind;

/// Tui.f's shape: a dialog over a vbox of a label, two edit boxes with
/// F-key callbacks, and an hbox of buttons.
struct Tui {
    ui: Ui,
    dialog: Handle,
    text: Handle,
    res: Handle,
    button: Handle,
}

fn tui() -> Tui {
    let mut ui = Ui::new();
    let [dialog, vbox, label, text, res, hbox, button] = [
        Kind::Dialog,
        Kind::Vbox,
        Kind::Label,
        Kind::MultiLine,
        Kind::MultiLine,
        Kind::Hbox,
        Kind::Button,
    ]
    .map(|k| ui.create(k));
    for (p, c) in [
        (vbox, label),
        (vbox, text),
        (vbox, res),
        (vbox, hbox),
        (hbox, button),
        (dialog, vbox),
    ] {
        assert!(ui.tree.append(p, c));
    }
    ui.set(label, "TITLE", Some(" last executed line |"));
    ui.set(button, "TITLE", Some("Reset stack"));
    ui.tree.set_callback(text, "K_F6", Some(0x1006));
    ui.tree.set_callback(text, "K_sF6", Some(0x2006));
    ui.tree.set_callback(dialog, "K_cS", Some(0x5));
    ui.tree.set_callback(button, "ACTION", Some(0xB));
    Tui {
        ui,
        dialog,
        text,
        res,
        button,
    }
}

fn key(name: &str) -> Event {
    Event::Key(from_name(name).unwrap())
}

#[test]
fn text_attributes_as_gui_reads_them() {
    let Tui { mut ui, text, .. } = tui();
    assert!(ui.set(text, "VALUE", Some("1 2 +\n3 4 *\n")));
    assert_eq!(ui.get(text, "CARET").as_deref(), Some("1,1"));
    assert!(ui.set(text, "CARET", Some("2,3")));
    assert_eq!(
        ui.get(text, "CARET").as_deref(),
        Some("2,3"),
        "caret-y? reads 2"
    );
    assert_eq!(ui.get(text, "CARETPOS").as_deref(), Some("8"));
    // line-text?: SELECTION "n,1:n,1000", then SELECTEDTEXT.
    assert!(ui.set(text, "SELECTION", Some("2,1:2,1000")));
    assert_eq!(ui.get(text, "SELECTEDTEXT").as_deref(), Some("3 4 *"));
    assert_eq!(ui.get(text, "SELECTION").as_deref(), Some("2,1:2,6"));
    assert!(ui.set(text, "SELECTEDTEXT", Some("5")));
    assert_eq!(ui.get(text, "VALUE").as_deref(), Some("1 2 +\n5\n"));
    assert_eq!(ui.get(text, "SELECTION"), None);
    assert!(ui.set(text, "INSERT", Some(" dup")));
    assert!(ui.set(text, "APPEND", Some("bye")));
    assert_eq!(
        ui.get(text, "VALUE").as_deref(),
        Some("1 2 +\n5 dup\n\nbye")
    );
    assert_eq!(ui.get(text, "LINECOUNT").as_deref(), Some("4"));
    assert!(ui.set(text, "SELECTIONPOS", Some("0:5")));
    assert_eq!(ui.get(text, "SELECTEDTEXT").as_deref(), Some("1 2 +"));
    assert!(!ui.set(text, "CARET", Some("x")));
    assert_eq!(ui.get(text, "FONT"), None);
}

#[test]
fn keys_run_callbacks_by_name() {
    let Tui {
        mut ui,
        dialog,
        text,
        ..
    } = tui();
    assert!(ui.show(dialog, -1, -1, false));
    assert_eq!(ui.focus, Some(text), "the first edit box has the focus");
    let call = ui.event(key("K_F6")).expect("F6 has a callback");
    assert_eq!(
        (
            call.handle,
            call.name.as_str(),
            call.func,
            call.args.clone()
        ),
        (text, "K_F6", 0x1006, vec![])
    );
    assert_eq!(ui.finish(&call, DEFAULT), None, "F6 does nothing more");
    assert_eq!(ui.event(key("K_sF6")).map(|c| c.func), Some(0x2006));
    let save = ui.event(key("K_cS")).expect("found on the dialog");
    assert_eq!((save.handle, save.func), (dialog, 0x5));
    assert_eq!(ui.event(key("K_F7")), None, "no callback, no edit");
}

#[test]
fn typing_edits_the_focused_text() {
    let Tui {
        mut ui,
        dialog,
        text,
        res,
        ..
    } = tui();
    ui.show(dialog, 0, 0, false);
    for c in "2 3 +".chars() {
        assert_eq!(ui.event(Event::Key(c as u32)), None);
    }
    ui.event(Event::Key(CR));
    assert_eq!(ui.get(text, "VALUE").as_deref(), Some("2 3 +\n"));
    ui.event(Event::Key(TAB));
    assert_eq!(ui.focus, Some(res));
    ui.event(Event::Key('x' as u32));
    assert_eq!(ui.get(res, "VALUE").as_deref(), Some("x"));
    ui.event(Event::Key(TAB | SHIFT));
    assert_eq!(ui.focus, Some(text));
}

#[test]
fn k_any_gets_the_code_and_decides() {
    let Tui {
        mut ui,
        dialog,
        text,
        ..
    } = tui();
    ui.tree.set_callback(dialog, "K_ANY", Some(0xA));
    ui.show(dialog, 0, 0, false);
    let call = ui.event(Event::Key('q' as u32)).expect("K_ANY");
    assert_eq!((call.func, call.args.clone()), (0xA, vec!['q' as i32]));
    assert_eq!(ui.finish(&call, IGNORE), None);
    assert_eq!(ui.get(text, "VALUE").as_deref(), Some(""), "ignored");
    let call = ui.event(Event::Key('q' as u32)).unwrap();
    ui.finish(&call, DEFAULT);
    assert_eq!(ui.get(text, "VALUE").as_deref(), Some("q"), "typed");
}

#[test]
fn buttons_toggles_and_closing() {
    let Tui {
        mut ui,
        dialog,
        button,
        ..
    } = tui();
    ui.show(dialog, 0, 0, false);
    assert!(ui.running());
    let call: Call = ui.event(Event::Click(button)).unwrap();
    assert_eq!((call.name.as_str(), call.func), ("ACTION", 0xB));
    ui.event(Event::Focus(button));
    assert_eq!(
        ui.event(Event::Key(CR)).map(|c| c.func),
        Some(0xB),
        "Enter clicks"
    );
    let t = ui.create(Kind::Toggle);
    ui.tree.set_callback(t, "ACTION", Some(0x7));
    assert_eq!(ui.event(Event::Click(t)).map(|c| c.args), Some(vec![1]));
    assert_eq!(ui.get(t, "VALUE").as_deref(), Some("ON"));
    assert_eq!(ui.event(Event::Click(t)).map(|c| c.args), Some(vec![0]));
    assert_eq!(ui.finish(&call, CLOSE), None);
    assert!(!ui.running(), "CLOSE ends the main loop");
}

#[test]
fn the_close_box_hides_and_destroy_forgets() {
    let Tui {
        mut ui,
        dialog,
        text,
        ..
    } = tui();
    ui.show(dialog, 0, 0, false);
    assert_eq!(ui.event(Event::Close(dialog)), None);
    assert!(!ui.running());
    ui.destroy(dialog);
    assert!(ui.texts.is_empty() && ui.focus.is_none());
    assert!(ui.tree.node(text).is_none());
    assert!(!ui.show(dialog, 0, 0, false));
}

#[test]
fn the_change_stream_follows_everything() {
    let Tui {
        mut ui,
        dialog,
        text,
        ..
    } = tui();
    ui.show(dialog, -1, -1, false);
    ui.event(Event::Key('a' as u32));
    let changes = ui.tree.take_changes();
    assert!(changes.contains(&Change::Shown {
        handle: dialog,
        x: -1,
        y: -1,
        modal: false
    }));
    assert!(changes.contains(&Change::Focus { handle: text }));
    assert!(changes.contains(&Change::Attr {
        handle: text,
        name: "VALUE".into(),
        value: Some("a".into())
    }));
    assert!(changes.contains(&Change::Attr {
        handle: text,
        name: "CARET".into(),
        value: Some("1,2".into())
    }));
    for c in &changes {
        assert_eq!(decode(&encode(c)).as_ref(), Some(c));
    }
}
