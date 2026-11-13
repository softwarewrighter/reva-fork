use reva_ui::ui::{Event, Ui};
use reva_ui_base::change::Handle;
use reva_ui_base::codec::encode;
use reva_ui_base::kind::Kind;
use reva_ui_grid::grid::Role;
use reva_ui_layout::geom::Size;
use reva_ui_mirror::mirror::Mirror;
use reva_ui_paint::paint::{click, paint};

/// The renderer's copy of `ui`, kept up through its change stream.
fn mirror(ui: &mut Ui, m: &mut Mirror) {
    let lines: Vec<String> = ui.tree.take_changes().iter().map(encode).collect();
    m.apply_lines(&lines.join("\n"));
}

/// A dialog "Form": a label, a text field, a multiline and two buttons.
fn form() -> (Ui, [Handle; 7]) {
    let mut ui = Ui::new();
    let [d, v, l, t, e, h, ok] = [
        Kind::Dialog,
        Kind::Vbox,
        Kind::Label,
        Kind::Text,
        Kind::MultiLine,
        Kind::Hbox,
        Kind::Button,
    ]
    .map(|k| ui.create(k));
    let cancel = ui.create(Kind::Button);
    for (p, c) in [(h, ok), (h, cancel), (v, l), (v, t), (v, e), (v, h), (d, v)] {
        ui.tree.append(p, c);
    }
    ui.set(d, "TITLE", Some("Form"));
    ui.set(l, "TITLE", Some("Name"));
    ui.set(ok, "TITLE", Some("OK"));
    ui.set(cancel, "TITLE", Some("No"));
    ui.set(e, "VISIBLECOLUMNS", Some("12"));
    ui.set(e, "VISIBLELINES", Some("2"));
    (ui, [d, l, t, e, ok, cancel, h])
}

#[test]
fn a_form_on_the_grid() {
    let (mut ui, [d, _, t, e, ..]) = form();
    ui.show(d, -1, -1, false);
    ui.set(e, "VALUE", Some("one\ntwo\nthree"));
    ui.set(e, "CARET", Some("3,6"));
    let mut m = Mirror::default();
    mirror(&mut ui, &mut m);
    assert_eq!(m.focus, Some(t));
    let g = paint(&m, Size { cols: 24, rows: 10 });
    let want = "    ┌ Form ─────[×]┐\n    │Name          │\n    │[          ]  │\n    │┌────────────┐│\n    ││two         ││\n    ││three       ││\n    │└────────────┘│\n    │[ OK ][ No ]  │\n    └──────────────┘\n\ncursor 6,2\n";
    assert_eq!(
        g.to_text(),
        want,
        "centred; the multiline scrolled to its caret"
    );
    assert_eq!(
        g.cell(5, 2).map(|c| c.role),
        Some(Role::Focus),
        "the focused field's bracket"
    );
    assert_eq!(g.cell(6, 4).map(|c| c.role), Some(Role::Field));
    assert_eq!(g.cell(0, 0).map(|c| c.role), Some(Role::Screen));
    assert_eq!(g.cell(6, 0).map(|c| c.role), Some(Role::Title));
}

#[test]
fn selection_focus_and_clicks() {
    let (mut ui, [d, _, t, e, ok, cancel, _]) = form();
    ui.show(d, 0, 0, false);
    ui.set(t, "VALUE", Some("Ada"));
    ui.set(t, "SELECTION", Some("1:3"));
    ui.event(Event::Focus(e));
    let mut m = Mirror::default();
    mirror(&mut ui, &mut m);
    let screen = Size { cols: 20, rows: 10 };
    let g = paint(&m, screen);
    assert_eq!(g.cursor, Some((2, 4)), "in the multiline now");
    let roles: Vec<Role> = (2..5).map(|c| g.cell(c, 2).unwrap().role).collect();
    assert_eq!(
        roles,
        [Role::Selection, Role::Selection, Role::Field],
        "\"Ad\" selected"
    );
    assert_eq!(
        g.cell(1, 3).map(|c| (c.ch, c.role)),
        Some(('┌', Role::Active))
    );
    assert_eq!(
        click(&m, screen, 2, 7),
        [format!("focus {ok}"), format!("click {ok}")]
    );
    assert_eq!(
        click(&m, screen, 8, 7),
        [format!("focus {cancel}"), format!("click {cancel}")]
    );
    assert_eq!(click(&m, screen, 3, 2), [format!("focus {t}")]);
    assert!(click(&m, screen, 3, 1).is_empty(), "a label takes no click");
    assert!(click(&m, screen, 19, 9).is_empty(), "nor the screen");
    for col in 12..16 {
        assert_eq!(
            click(&m, screen, col, 0),
            [format!("close {d}")],
            "the close box [×] and its corner, at column {col}"
        );
    }
    assert!(click(&m, screen, 11, 0).is_empty(), "the border before it");
}

#[test]
fn a_modal_dialog_on_top() {
    let (mut ui, [d, _, t, ..]) = form();
    let (p, b) = (ui.create(Kind::Dialog), ui.create(Kind::Button));
    ui.tree.append(p, b);
    ui.set(b, "TITLE", Some("Sure?"));
    ui.show(d, 0, 0, false);
    ui.show(p, -1, -1, true);
    let mut m = Mirror::default();
    mirror(&mut ui, &mut m);
    let screen = Size { cols: 30, rows: 12 };
    let text = paint(&m, screen).to_text();
    assert!(text.lines().nth(5).unwrap().contains("[ Sure? ]"), "{text}");
    assert!(
        click(&m, screen, 3, 2).is_empty(),
        "the dialog under a modal one takes no clicks: {:?}",
        t
    );
    assert_eq!(click(&m, screen, 13, 5).len(), 2);
    ui.hide(p);
    mirror(&mut ui, &mut m);
    assert_eq!(m.top(), Some(d));
    ui.destroy(d);
    mirror(&mut ui, &mut m);
    assert!(m.shown.is_empty() && m.tree.node(t).is_none());
    assert_eq!(paint(&m, screen).to_text().trim(), "", "nothing shown");
}
