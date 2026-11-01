use reva_x86_fpustate::fpu::{C1, Fpu, IE, INDEFINITE, SF};

#[test]
fn stack_tags_overflow_and_underflow() {
    let mut f = Fpu::default();
    assert!(f.st(0).is_nan());
    assert_eq!(f.status() & (IE | SF), IE | SF);
    f.reset();
    for i in 0..8 {
        f.push(f64::from(i));
    }
    assert_eq!(f.st(0), 7.0);
    assert_eq!(f.status() & IE, 0);
    f.push(8.0);
    assert!(f.st(0).is_nan());
    assert_eq!(f.status() & (IE | SF | C1), IE | SF | C1);
}

#[test]
fn indefinite_is_the_x87_bit_pattern() {
    assert_eq!(INDEFINITE.to_bits(), 0xfff8_0000_0000_0000);
}
