use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use reva_ui_base::keys::name;
use reva_ui_grid::grid::{Grid, Role};
use reva_ui_layout::geom::Size;
use reva_ui_term::keys::{is_quit, key_code};
use reva_ui_term::screen::draw;

fn key(code: KeyCode, m: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, m)
}

#[test]
fn terminal_keys_as_iup_codes() {
    let n = |k: KeyEvent| key_code(&k).and_then(name);
    assert_eq!(
        n(key(KeyCode::F(6), KeyModifiers::NONE)).as_deref(),
        Some("K_F6")
    );
    assert_eq!(
        n(key(KeyCode::F(6), KeyModifiers::SHIFT)).as_deref(),
        Some("K_sF6")
    );
    assert_eq!(
        n(key(KeyCode::Enter, KeyModifiers::CONTROL)).as_deref(),
        Some("K_cCR")
    );
    assert_eq!(
        n(key(KeyCode::Char('s'), KeyModifiers::CONTROL)).as_deref(),
        Some("K_cS")
    );
    assert_eq!(
        n(key(KeyCode::BackTab, KeyModifiers::SHIFT)).as_deref(),
        Some("K_sTAB")
    );
    assert_eq!(
        n(key(KeyCode::Home, KeyModifiers::NONE)).as_deref(),
        Some("K_HOME")
    );
    assert_eq!(
        key_code(&key(KeyCode::Char('A'), KeyModifiers::SHIFT)),
        Some('A' as u32)
    );
    let mut release = key(KeyCode::Char('a'), KeyModifiers::NONE);
    release.kind = KeyEventKind::Release;
    assert_eq!(key_code(&release), None);
    assert!(is_quit(&key(KeyCode::Char('c'), KeyModifiers::CONTROL)));
    assert!(!is_quit(&key(KeyCode::Char('c'), KeyModifiers::NONE)));
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn only_changed_cells_are_drawn() {
    let mut a = Grid::new(Size { cols: 6, rows: 2 }, Role::Screen);
    a.text(0, 0, "hello", Role::Text, 6);
    let mut all = Vec::new();
    draw(&mut all, None, &a).unwrap();
    assert!(
        text(&all).contains("hello"),
        "the first screen draws everything"
    );
    let mut b = a.clone();
    b.put(4, 0, '!', Role::Text);
    b.cursor = Some((5, 1));
    let mut diff = Vec::new();
    draw(&mut diff, Some(&a), &b).unwrap();
    let d = text(&diff);
    assert!(
        d.contains('!') && !d.contains("hell"),
        "only the changed cell: {d:?}"
    );
    assert!(
        d.contains("\u{1b}[2;6H"),
        "then the cursor at its cell: {d:?}"
    );
    let mut none = Vec::new();
    draw(&mut none, Some(&b), &b).unwrap();
    assert!(!text(&none).contains('!'), "nothing changed, nothing drawn");
}
