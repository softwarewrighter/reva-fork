use reva_words_text::compare::{first_difference, fnv_hash, length_order};
use reva_words_text::digits::{digit_value, to_digit};
use reva_words_text::number::{format_r, parse_double, parse_single};

#[test]
fn digits() {
    assert_eq!(digit_value(b'7' as u32, 10), Some(7));
    assert_eq!(digit_value(b'f' as u32, 16), Some(15));
    assert_eq!(digit_value(b'F' as u32, 16), Some(15));
    assert_eq!(digit_value(b'9' as u32, 8), None);
    assert_eq!(digit_value(b'@' as u32, 36), None);
    assert_eq!(to_digit(11, 10), Some(b'B' as u32));
    assert_eq!(to_digit(20, 0), None);
}

#[test]
fn singles() {
    assert_eq!(parse_single(b"42", 10), Some(42));
    assert_eq!(parse_single(b"-$ff", 10), Some(0xffu32.wrapping_neg()));
    assert_eq!(parse_single(b"'A", 10), Some(65));
    assert_eq!(parse_single(b"'", 10), None);
    assert_eq!(parse_single(b"-", 10), None);
    assert_eq!(parse_single(b"#", 16), None);
    assert_eq!(parse_single(b"2147483648", 10), None);
    assert_eq!(parse_single(b"12x", 10), None);
}

#[test]
fn doubles() {
    assert_eq!(parse_double(b"1,000L", 10), Some((1000, 0)));
    assert_eq!(parse_double(b"-1L", 10), Some((u32::MAX, u32::MAX)));
    assert_eq!(parse_double(b"$100000000L", 10), Some((0, 1)));
    assert_eq!(parse_double(b"L", 10), None);
    assert_eq!(parse_double(b"'AL", 10), None);
    assert_eq!(parse_double(b"12", 10), None);
}

#[test]
fn formats() {
    assert_eq!(format_r(42, 10, 0, b' '), b"42");
    assert_eq!(format_r(-5i32 as u32, 10, 4, b' '), b"  -5");
    assert_eq!(format_r(255, 16, 4, b'0'), b"00FF");
}

#[test]
fn comparisons() {
    assert_eq!(length_order(2, 3), -1);
    assert_eq!(first_difference(b"abc", b"abd", false), Some(-1));
    assert_eq!(first_difference(b"ABC", b"abc", true), None);
    assert_eq!(fnv_hash(b""), 2_166_136_261);
}
