use reva_x86_float::f80::{f64_to_f80, f80_to_f64};
use reva_x86_float::math::{ldexp, split};

#[test]
fn f80_round_trips_every_kind_of_double() {
    for v in [
        0.0,
        -0.0,
        1.0,
        -2.5,
        std::f64::consts::PI,
        f64::MAX,
        f64::MIN_POSITIVE,
        5e-324,
        1e-310,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert_eq!(f80_to_f64(&f64_to_f80(v)).to_bits(), v.to_bits(), "{v}");
    }
    assert!(f80_to_f64(&f64_to_f80(f64::NAN)).is_nan());
}

#[test]
fn f80_bytes_match_the_x87() {
    // 1.0 and pi as `fstp tbyte` writes them
    assert_eq!(f64_to_f80(1.0), [0, 0, 0, 0, 0, 0, 0, 0x80, 0xff, 0x3f]);
    let pi80 = [0x35, 0xc2, 0x68, 0x21, 0xa2, 0xda, 0x0f, 0xc9, 0x00, 0x40];
    assert_eq!(f80_to_f64(&pi80), std::f64::consts::PI);
    // more precision than a double holds: rounded to nearest
    let third = [0xab, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xfd, 0x3f];
    assert_eq!(f80_to_f64(&third), 1.0 / 3.0);
    // beyond the double range
    assert_eq!(
        f80_to_f64(&[0, 0, 0, 0, 0, 0, 0, 0x80, 0x00, 0x44]),
        f64::INFINITY
    );
    assert_eq!(f80_to_f64(&[0, 0, 0, 0, 0, 0, 0, 0x80, 0x01, 0x00]), 0.0);
}

#[test]
fn helpers() {
    assert_eq!(split(12.0), (1.5, 3));
    assert_eq!(split(5e-324), (1.0, -1074));
    assert_eq!(ldexp(1.0, 1030), f64::INFINITY);
    assert_eq!(ldexp(1.0, -1074), 5e-324);
    assert_eq!(ldexp(3.0, 4), 48.0);
}
