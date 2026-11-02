use reva_ui_base::keys::{CTRL, SHIFT, name};
use reva_ui_web::keys::key_code;

#[test]
fn browser_keys_as_iup_codes() {
    let n = |k: &str, s, c, a| key_code(k, s, c, a).and_then(name);
    assert_eq!(n("F6", false, false, false).as_deref(), Some("K_F6"));
    assert_eq!(n("F6", true, false, false).as_deref(), Some("K_sF6"));
    assert_eq!(n("s", false, true, false).as_deref(), Some("K_cS"));
    assert_eq!(
        n("ArrowLeft", false, false, false).as_deref(),
        Some("K_LEFT")
    );
    assert_eq!(n("Enter", false, false, false).as_deref(), Some("K_CR"));
    assert_eq!(n("Tab", true, false, false).as_deref(), Some("K_sTAB"));
    assert_eq!(
        key_code("A", true, false, false),
        Some('A' as u32),
        "Shift is in the character"
    );
    assert_eq!(key_code("é", false, false, false), Some('é' as u32));
    assert_eq!(key_code(" ", false, false, false), Some(32));
    assert_eq!(key_code("Home", false, true, false), Some(0xFF50 | CTRL));
    assert_eq!(key_code("End", true, false, false), Some(0xFF57 | SHIFT));
    assert_eq!(key_code("Shift", true, false, false), None);
    assert_eq!(key_code("AudioVolumeUp", false, false, false), None);
}

#[test]
fn clipboard_keys_and_pasted_text() {
    use reva_ui_web::keys::{clipboard_key, pasted};
    assert!(clipboard_key("v", true, false) && clipboard_key("C", false, true));
    assert!(!clipboard_key("v", false, false), "a plain v types");
    assert!(!clipboard_key("s", true, false), "Ctrl-S is the program's");
    assert!(clipboard_key("x", false, true), "Cmd-X cuts");
    assert!(!clipboard_key("x", true, false), "Ctrl-X is the program's");
    assert_eq!(
        pasted("6 7 *\n."),
        [
            "key 54", "key 32", "key 55", "key 32", "key 42", "key 13", "key 46"
        ]
    );
    assert_eq!(pasted("a\tb\r\n"), ["key 97", "key 32", "key 98", "key 13"]);
}
