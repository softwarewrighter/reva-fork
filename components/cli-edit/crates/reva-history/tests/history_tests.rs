use reva_history::history::{History, MAX};
use std::fs;

#[test]
fn skips_blank_and_repeats() {
    let mut h = History::in_memory();
    for l in ["1 .", "", "  ", "1 .", "2 .", "1 ."] {
        h.add(l);
    }
    assert_eq!(h.entries, ["1 .", "2 .", "1 ."]);
}

#[test]
fn searches_both_ways() {
    let mut h = History::in_memory();
    for l in ["dup", "swap", "dup dup", "drop"] {
        h.add(l);
    }
    assert_eq!(h.search_back("dup", 4), Some(2));
    assert_eq!(h.search_back("dup", 2), Some(0));
    assert_eq!(h.search_back("dup", 0), None);
    assert_eq!(h.search_forward("dup", 0), Some(2));
    assert_eq!(h.search_forward("dup", 2), None);
}

#[test]
fn file_round_trip_and_trim() {
    let dir = std::env::temp_dir().join(format!("reva-hist-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("h");
    let many: String = (0..2 * MAX + 5).map(|i| format!("{i} .\n")).collect();
    fs::write(&path, many).unwrap();
    let mut h = History::open(path.clone());
    assert_eq!(h.len(), MAX);
    assert_eq!(h.get(MAX - 1), Some(&*format!("{} .", 2 * MAX + 4)));
    h.add("words");
    let again = History::open(path.clone());
    assert_eq!(again.get(again.len() - 1), Some("words"));
    assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), MAX + 1);
    let _ = fs::remove_dir_all(dir);
}
