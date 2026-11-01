use reva_ui::ui::{Event, Ui};
use reva_ui_base::kind::Kind;
use reva_vlib_iup::data::{changes, decode_event, encode_event};
use reva_vlib_iup::names::{FN_BASE, FUNCS, IUP_NAMES, Op, function, lookup};

#[test]
fn every_function_has_an_address_in_the_null_page() {
    for (i, (name, op)) in FUNCS.iter().enumerate() {
        let addr = lookup(name).unwrap();
        assert_eq!(addr, FN_BASE + 4 * i as u32, "{name}: names are unique");
        assert!(addr < 0x1000, "{name}: never runnable");
        assert_eq!(function(addr), Some((*name, *op)));
    }
    assert_eq!(lookup("IupNoSuch"), None);
    assert_eq!(function(FN_BASE + 2), None);
    assert_eq!(function(FN_BASE + 4 * FUNCS.len() as u32), None);
    assert_eq!(
        function(lookup("IupMultiLine").unwrap()).map(|f| f.1),
        Some(Op::New(Kind::MultiLine, reva_vlib_iup::names::Args::None))
    );
    assert!(IUP_NAMES.contains(&"libiupall.so"), "reva.f's Linux name");
}

#[test]
fn the_functions_lib_ui_iup_binds_are_all_there() {
    let bound = String::from_utf8_lossy(include_bytes!("../../../../../lib/ui/iup"));
    for line in bound.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        if let [_, "func:" | "vfunc:", name, ..] = words[..] {
            assert!(lookup(name).is_some(), "lib/ui/iup binds {name}");
        }
    }
}

#[test]
fn event_lines() {
    for e in [
        Event::Key(0xFFC3),
        Event::Click(5),
        Event::Focus(3),
        Event::Close(1),
    ] {
        assert_eq!(decode_event(&encode_event(&e)), Some(e));
    }
    assert_eq!(
        decode_event("key K_sF6"),
        Some(Event::Key(0xFFC3 | 0x1000_0000))
    );
    assert_eq!(decode_event(" click 7 "), Some(Event::Click(7)));
    assert_eq!(decode_event("key K_NOPE"), None);
    assert_eq!(decode_event("wiggle 1"), None);
    assert_eq!(decode_event("click"), None);
}

#[test]
fn changes_go_out_as_lines() {
    let mut ui = Ui::new();
    let l = ui.create(Kind::Label);
    ui.set(l, "TITLE", Some("a b"));
    assert_eq!(
        String::from_utf8(changes(&mut ui)).unwrap(),
        "created 1 label\nattr 1 TITLE =a\\sb"
    );
    assert!(changes(&mut ui).is_empty(), "taken once");
}
