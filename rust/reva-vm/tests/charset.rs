//! The virtual iconv's encodings (reva_vm::charset): names, round trips,
//! and the two ways input can be bad.

use reva_vm::charset::{Bad, Enc, decode, encode, parse};

fn round_trip(enc: Enc, s: &str) -> String {
    let bytes: Vec<u8> = s.chars().flat_map(|c| encode(enc, c).unwrap()).collect();
    let (mut out, mut i) = (String::new(), 0);
    while i < bytes.len() {
        let (c, n) = decode(enc, &bytes[i..]).unwrap();
        out.push(c);
        i += n;
    }
    out
}

#[test]
fn names_as_iconv_spells_them() {
    assert_eq!(parse("utf-8"), Some(Enc::Utf8));
    assert_eq!(parse("UCS-2LE"), Some(Enc::Ucs2 { le: true }));
    assert_eq!(parse("ucs_2le//TRANSLIT"), Some(Enc::Ucs2 { le: true }));
    assert_eq!(parse("ISO-8859-1"), Some(Enc::Latin1));
    assert_eq!(parse("EBCDIC-US"), None);
}

#[test]
fn every_encoding_round_trips_what_it_can_hold() {
    let all = "héllo €𝄞";
    for enc in [
        Enc::Utf8,
        Enc::Utf16 { le: true },
        Enc::Utf16 { le: false },
        Enc::Utf32 { le: true },
    ] {
        assert_eq!(round_trip(enc, all), all, "{enc:?}");
    }
    assert_eq!(round_trip(Enc::Ucs2 { le: true }, "héllo €"), "héllo €");
    assert_eq!(round_trip(Enc::Latin1, "héllo"), "héllo");
    assert_eq!(encode(Enc::Ucs2 { le: true }, '𝄞'), None);
    assert_eq!(encode(Enc::Latin1, '€'), None);
    assert_eq!(encode(Enc::Ucs2 { le: true }, 'A'), Some(vec![0x41, 0]));
}

#[test]
fn bad_input_is_incomplete_or_invalid() {
    assert_eq!(decode(Enc::Utf8, &[0xe2, 0x82]), Err(Bad::Incomplete));
    assert_eq!(decode(Enc::Utf8, &[0xe2, 0x41]), Err(Bad::Invalid));
    assert_eq!(decode(Enc::Utf8, &[0xff]), Err(Bad::Invalid));
    assert_eq!(decode(Enc::Ascii, &[0xc3]), Err(Bad::Invalid));
    assert_eq!(
        decode(Enc::Ucs2 { le: true }, &[0x41]),
        Err(Bad::Incomplete)
    );
    assert_eq!(
        decode(Enc::Utf16 { le: true }, &[0x3d, 0xd8]),
        Err(Bad::Incomplete)
    );
}
