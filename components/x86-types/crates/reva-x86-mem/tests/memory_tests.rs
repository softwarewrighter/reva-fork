use reva_x86_mem::memory::Memory;

#[test]
fn null_guard_and_bounds() {
    let mut m = Memory::new(0x2000);
    assert!(m.read_u32(0).is_err());
    assert!(m.read_u32(0x1ffd).is_err());
    m.write_u32(0x1ffc, 0xdeadbeef).unwrap();
    assert_eq!(m.read_u32(0x1ffc).unwrap(), 0xdeadbeef);
    assert_eq!(m.read_u8(0x1fff).unwrap(), 0xde);
}

#[test]
fn writes_to_code_pages_are_reported_once() {
    let mut m = Memory::new(0x4000);
    m.write_u8(0x1100, 1).unwrap();
    assert!(!m.has_dirty());
    m.mark_code(0x1100);
    m.write_u32(0x11fe, 7).unwrap(); // spans pages 0x1100 and 0x1200
    assert_eq!(m.take_dirty(), vec![0x1100]);
    m.write_u8(0x1100, 2).unwrap();
    assert!(!m.has_dirty());
}

#[test]
fn overlapping_forward_move_smears_like_reva() {
    let mut m = Memory::new(0x2000);
    m.write_bytes(0x1000, b"ab....").unwrap();
    m.move_forward(0x1000, 0x1002, 4).unwrap();
    assert_eq!(m.slice(0x1000, 6).unwrap(), b"ababab");
}
