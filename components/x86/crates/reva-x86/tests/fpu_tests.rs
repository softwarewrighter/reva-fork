use reva_x86::mem::Memory;
use reva_x86::{Cpu, EAX, EBX, Exec, decode};
use reva_x86_float::f80::f64_to_f80;

/// Run x87 code at 0x2000 (data at 0x4000) and return the CPU and memory.
fn run(code: &[u8], data: &[u8]) -> (Cpu, Memory) {
    let mut m = Memory::new(0x8000);
    m.write_bytes(0x2000, code).unwrap();
    m.write_bytes(0x4000, data).unwrap();
    let mut c = Cpu {
        esp_floor: 0x6000,
        ..Cpu::default()
    };
    c.r[EBX as usize] = 0x4000;
    c.eip = 0x2000;
    while c.eip < 0x2000 + code.len() as u32 {
        let i = decode(&m, c.eip).unwrap();
        c.exec(&mut m, &i).unwrap();
    }
    (c, m)
}

#[test]
fn floats_library_idioms() {
    // fld1 fld1 faddp (f+ is `de c1`), fldpi, fdivp: st0 = 2/pi
    let (mut c, _) = run(
        &[0xd9, 0xe8, 0xd9, 0xe8, 0xde, 0xc1, 0xd9, 0xeb, 0xde, 0xf9],
        &[],
    );
    assert_eq!(c.fpu.st(0), 2.0 / std::f64::consts::PI);
    assert_eq!(c.fpu.top, 7);
    // f- is `de e9`: a b -- a-b
    let mut data = 10.0f64.to_le_bytes().to_vec();
    data.extend(4.0f64.to_le_bytes());
    let (mut c, _) = run(&[0xdd, 0x03, 0xdd, 0x43, 0x08, 0xde, 0xe9], &data);
    assert_eq!(c.fpu.st(0), 6.0);
    // fstp tbyte [ebx+16], fld tbyte [ebx+16]: 80-bit round trip
    let (mut c, m) = run(&[0xd9, 0xeb, 0xdb, 0x7b, 0x10, 0xdb, 0x6b, 0x10], &[]);
    assert_eq!(c.fpu.st(0), std::f64::consts::PI);
    assert_eq!(
        m.slice(0x4010, 10).unwrap(),
        &f64_to_f80(std::f64::consts::PI)
    );
}

#[test]
fn integer_conversion_follows_the_rounding_control() {
    // fild dword [ebx]; fld1; fadd st0,st0 (2); fdivp (7/2 = 3.5); fist [ebx+4]
    let code = [
        0xdb, 0x03, 0xd9, 0xe8, 0xd8, 0xc0, 0xde, 0xf9, 0xdb, 0x53, 0x04,
    ];
    let (_, m) = run(&code, &7u32.to_le_bytes());
    assert_eq!(m.read_u32(0x4004).unwrap(), 4); // nearest-even: 3.5 -> 4
    // the same after `fldcw` with RC = truncate (floats' `finit`)
    let mut data = 7u32.to_le_bytes().to_vec();
    data.extend([0, 0, 0, 0]);
    data.extend(0x0f3fu16.to_le_bytes());
    let mut code2 = vec![0xd9, 0x6b, 0x08];
    code2.extend(code);
    let (_, m) = run(&code2, &data);
    assert_eq!(m.read_u32(0x4004).unwrap(), 3);
    // out of range: the integer indefinite
    let (_, m) = run(&[0xd9, 0xe8, 0xd9, 0xee, 0xde, 0xf9, 0xdb, 0x1b], &[]);
    assert_eq!(m.read_u32(0x4000).unwrap(), 0x8000_0000);
}

#[test]
fn comparisons_reach_the_status_word_and_eflags() {
    // fld1 fldz fcompp fnstsw ax: 0 < 1 -> C0 (bit 8)
    let (c, _) = run(&[0xd9, 0xe8, 0xd9, 0xee, 0xde, 0xd9, 0xdf, 0xe0], &[]);
    assert_eq!(c.r[EAX as usize] & 0x4500, 0x0100);
    assert_eq!(c.fpu.valid, 0);
    // fldz fld1 fcomip st1: 1 > 0 -> CF = ZF = 0
    let (c, _) = run(&[0xd9, 0xee, 0xd9, 0xe8, 0xdf, 0xf1], &[]);
    assert!(!c.cf && !c.zf && !c.pf);
    // fxam on an empty register: C3 and C0
    let (c, _) = run(&[0xd9, 0xe5, 0xdf, 0xe0], &[]);
    assert_eq!(c.r[EAX as usize] & 0x4700, 0x4100);
}

#[test]
fn transcendentals() {
    // fldln2 fld1 fadd st0,st0 ... fyl2x: ln(2) * log2(2) = ln 2;  fsin of pi/2
    let (mut c, _) = run(
        &[0xd9, 0xe8, 0xd8, 0xc0, 0xd9, 0xed, 0xd9, 0xc9, 0xd9, 0xf1],
        &[],
    );
    assert_eq!(c.fpu.st(0), std::f64::consts::LN_2);
    let (mut c, _) = run(
        &[0xd9, 0xeb, 0xd9, 0xe8, 0xd8, 0xc0, 0xde, 0xf9, 0xd9, 0xfe],
        &[],
    );
    assert_eq!(c.fpu.st(0), 1.0);
    // fxtract of 12: exponent 3 below significand 1.5
    let mut data = 12.0f64.to_le_bytes().to_vec();
    data.extend([0; 8]);
    let (mut c, _) = run(&[0xdd, 0x03, 0xd9, 0xf4], &data);
    assert_eq!((c.fpu.st(0), c.fpu.st(1)), (1.5, 3.0));
}
