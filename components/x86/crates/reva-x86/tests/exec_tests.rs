use reva_x86::fault::Fault;
use reva_x86::mem::Memory;
use reva_x86::{Cpu, EAX, EBX, ESI, ESP, Exec, Op, Regs, Size, decode};

/// Run `code` placed at 0x2000 with the given data stack, Reva-style:
/// `stack` is bottom..top, TOS goes in EAX, the rest below ESI.
fn run(code: &[u8], stack: &[u32]) -> (Cpu, Vec<u32>) {
    let mut m = Memory::new(0x10000);
    m.write_bytes(0x2000, code).unwrap();
    m.write_u8(0x2000 + code.len() as u32, 0xf4).unwrap(); // hlt sentinel
    let mut c = Cpu {
        esp_floor: 0x8000,
        ..Cpu::default()
    };
    c.r[ESP as usize] = 0x9000;
    let s0 = 0x6000u32;
    let mut sp = s0;
    let (tos, rest) = stack.split_last().map(|(t, r)| (*t, r)).unwrap_or((0, &[]));
    for v in rest {
        sp -= 4;
        m.write_u32(sp, *v).unwrap();
    }
    c.r[ESI as usize] = sp;
    c.r[EAX as usize] = tos;
    c.eip = 0x2000;
    loop {
        let i = decode(&m, c.eip).unwrap();
        if i.op == Op::Hlt {
            break;
        }
        c.exec(&mut m, &i).unwrap();
        if c.eip >= 0x2000 + code.len() as u32 || c.eip < 0x2000 {
            break;
        }
    }
    let mut out = Vec::new();
    let mut a = s0;
    while a > c.r[ESI as usize] {
        a -= 4;
        out.push(m.read_u32(a).unwrap());
    }
    out.push(c.r[EAX as usize]);
    (c, out)
}

#[test]
fn cbw_and_cwd_sign_extend_the_low_half() {
    // cbw: AL into AX, the top half of EAX untouched
    let (c, _) = run(&[0x66, 0x98], &[0x1234_00f0]);
    assert_eq!(c.r[EAX as usize], 0x1234_fff0);
    let (c, _) = run(&[0x66, 0x98], &[0x1234_ff70]);
    assert_eq!(c.r[EAX as usize], 0x1234_0070);
    // cwd: AX's sign into DX, the top half of EDX untouched
    let (c, _) = run(&[0xba, 0x22, 0x22, 0x11, 0x11, 0x66, 0x99], &[0x8000]);
    assert_eq!(c.r[2], 0x1111_ffff);
    let (c, _) = run(&[0xba, 0x22, 0x22, 0x11, 0x11, 0x66, 0x99], &[0x7fff]);
    assert_eq!(c.r[2], 0x1111_0000);
}

// The byte sequences below are copied verbatim from src/reva.f.

#[test]
fn plus_minus_from_reva_f() {
    // : + inline{ 01 06 } drop ;   drop = 8b 06 8d 76 04
    assert_eq!(
        run(&[0x01, 0x06, 0x8b, 0x06, 0x8d, 0x76, 0x04], &[2, 3]).1,
        vec![5]
    );
    // : - inline{ 29 06 } drop ;
    assert_eq!(
        run(&[0x29, 0x06, 0x8b, 0x06, 0x8d, 0x76, 0x04], &[10, 3]).1,
        vec![7]
    );
}

#[test]
fn swap_rot_star() {
    // : swap inline{ 89 c3 8b 06 89 1e }
    assert_eq!(
        run(&[0x89, 0xc3, 0x8b, 0x06, 0x89, 0x1e], &[1, 2]).1,
        vec![2, 1]
    );
    // : rot inline{ 50 8B 46 04 8B 1E 89 5E 04 8F 06 }
    let rot = [
        0x50, 0x8b, 0x46, 0x04, 0x8b, 0x1e, 0x89, 0x5e, 0x04, 0x8f, 0x06,
    ];
    assert_eq!(run(&rot, &[1, 2, 3]).1, vec![2, 3, 1]);
    // : * inline{ f7 26 83 c6 04 }   (mul dword [esi]; add esi,4)
    assert_eq!(run(&[0xf7, 0x26, 0x83, 0xc6, 0x04], &[6, 7]).1, vec![42]);
}

#[test]
fn comparisons_via_flags() {
    // ((cmp)) = 3b 06 ad b8 00 00 00 00 ; < adds 7e 01 48
    let lt = [0x3b, 0x06, 0xad, 0xb8, 0, 0, 0, 0, 0x7e, 0x01, 0x48];
    assert_eq!(run(&lt, &[1, 2]).1, vec![u32::MAX]);
    assert_eq!(run(&lt, &[2, 1]).1, vec![0]);
    assert_eq!(run(&lt, &[(-5i32) as u32, 3]).1, vec![u32::MAX]);
    // : not inline{ 83 f8 01 19 c0 }   (cmp eax,1 ; sbb eax,eax)
    assert_eq!(run(&[0x83, 0xf8, 0x01, 0x19, 0xc0], &[0]).1, vec![u32::MAX]);
    assert_eq!(run(&[0x83, 0xf8, 0x01, 0x19, 0xc0], &[9]).1, vec![0]);
}

#[test]
fn division_and_errors() {
    // : / inline{ 89 C3 AD 99 F7 FB }
    let div = [0x89, 0xc3, 0xad, 0x99, 0xf7, 0xfb];
    assert_eq!(run(&div, &[(-7i32) as u32, 2]).1, vec![(-3i32) as u32]);
    let mut m = Memory::new(0x10000);
    m.write_bytes(0x2000, &div).unwrap();
    let mut c = Cpu {
        eip: 0x2000,
        ..Cpu::default()
    };
    c.r[ESI as usize] = 0x6000;
    m.write_u32(0x6000, 1).unwrap();
    for _ in 0..3 {
        let i = decode(&m, c.eip).unwrap();
        c.exec(&mut m, &i).unwrap();
    }
    let i = decode(&m, c.eip).unwrap();
    assert_eq!(c.exec(&mut m, &i), Err(Fault::DivideError { eip: 0x2004 }));
}

#[test]
fn string_ops_and_shifts() {
    // between: 89 c3 ad 89 c1 ad 93 40 29 c8 29 cb 29 c3 19 c0
    let between = [
        0x89, 0xc3, 0xad, 0x89, 0xc1, 0xad, 0x93, 0x40, 0x29, 0xc8, 0x29, 0xcb, 0x29, 0xc3, 0x19,
        0xc0,
    ];
    assert_eq!(run(&between, &[5, 1, 9]).1, vec![u32::MAX]);
    assert_eq!(run(&between, &[10, 1, 9]).1, vec![0]);
    // : << inline{ 89 c1 ad d3 e0 }
    assert_eq!(run(&[0x89, 0xc1, 0xad, 0xd3, 0xe0], &[3, 4]).1, vec![48]);
    // : >> inline{ 89 c1 ad d3 e8 }
    assert_eq!(
        run(&[0x89, 0xc1, 0xad, 0xd3, 0xe8], &[0x8000_0000, 31]).1,
        vec![1]
    );
}

#[test]
fn eight_bit_registers() {
    let mut c = Cpu::default();
    c.r[EBX as usize] = 0x1122_3344;
    assert_eq!(c.get_reg(7, Size::B), 0x33); // BH
    c.set_reg(3, Size::B, 0xaa); // BL
    c.set_reg(7, Size::B, 0xbb); // BH
    assert_eq!(c.r[EBX as usize], 0x1122_bbaa);
}
