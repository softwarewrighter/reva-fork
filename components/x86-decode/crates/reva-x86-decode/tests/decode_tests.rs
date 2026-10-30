use reva_x86_base::fault::Fault;
use reva_x86_decode::table::decode;
use reva_x86_mem::memory::Memory;
use reva_x86_types::cpu::{EAX, ESI, ESP};
use reva_x86_types::op::{Alu, Insn, Op, Rep, StrOp};
use reva_x86_types::operand::{MemRef, Opd, Size};

fn dec(bytes: &[u8]) -> Insn {
    let mut m = Memory::new(0x3000);
    m.write_bytes(0x2000, bytes).unwrap();
    decode(&m, 0x2000).unwrap()
}

fn esi(disp: u32) -> Opd {
    Opd::Mem(MemRef {
        base: Some(ESI),
        index: None,
        scale: 1,
        disp,
    })
}

#[test]
fn reva_dup() {
    // lea esi,[esi-4] ; mov [esi],eax
    let i = dec(&[0x8d, 0x76, 0xfc]);
    assert_eq!(i.len, 3);
    assert_eq!(
        i.op,
        Op::Lea {
            dst: ESI,
            mem: MemRef {
                base: Some(ESI),
                index: None,
                scale: 1,
                disp: 0xffff_fffc
            }
        }
    );
    let i = dec(&[0x89, 0x06]);
    assert_eq!(
        i.op,
        Op::Mov {
            size: Size::D,
            dst: esi(0),
            src: Opd::Reg(EAX)
        }
    );
}

#[test]
fn reva_plus_and_lodsd() {
    let i = dec(&[0x01, 0x06]);
    assert_eq!(
        i.op,
        Op::Alu {
            op: Alu::Add,
            size: Size::D,
            dst: esi(0),
            src: Opd::Reg(EAX)
        }
    );
    let i = dec(&[0xad]);
    assert_eq!(
        i.op,
        Op::Str {
            op: StrOp::Lods,
            size: Size::D,
            rep: Rep::None
        }
    );
    assert_eq!(i.len, 1);
}

#[test]
fn mov_moffs32() {
    // a3 is what lib/util/locals compiles: mov [locptr+n], eax
    let at = |disp| {
        Opd::Mem(MemRef {
            base: None,
            index: None,
            scale: 1,
            disp,
        })
    };
    let i = dec(&[0xa3, 0x8c, 0x6c, 0x14, 0x00]);
    assert_eq!(i.len, 5);
    assert_eq!(
        i.op,
        Op::Mov {
            size: Size::D,
            dst: at(0x146c8c),
            src: Opd::Reg(EAX)
        }
    );
    let i = dec(&[0xa1, 0x00, 0x21, 0, 0]);
    assert_eq!(
        i.op,
        Op::Mov {
            size: Size::D,
            dst: Opd::Reg(EAX),
            src: at(0x2100)
        }
    );
    let i = dec(&[0xa0, 0x00, 0x21, 0, 0]);
    assert_eq!(
        i.op,
        Op::Mov {
            size: Size::B,
            dst: Opd::Reg(EAX),
            src: at(0x2100)
        }
    );
    let i = dec(&[0xa2, 0x00, 0x21, 0, 0]);
    assert_eq!(
        i.op,
        Op::Mov {
            size: Size::B,
            dst: at(0x2100),
            src: Opd::Reg(EAX)
        }
    );
}

#[test]
fn branches_are_absolute() {
    let i = dec(&[0xe8, 0x10, 0, 0, 0]);
    assert_eq!(i.op, Op::Call(0x2015));
    let i = dec(&[0x75, 0x01]);
    assert_eq!(
        i.op,
        Op::Jcc {
            cc: 5,
            target: 0x2003
        }
    );
    let i = dec(&[0x0f, 0x84, 0xfa, 0xff, 0xff, 0xff]);
    assert_eq!(
        i.op,
        Op::Jcc {
            cc: 4,
            target: 0x2000
        }
    );
    let i = dec(&[0xff, 0x15, 0x00, 0x21, 0, 0]);
    assert_eq!(
        i.op,
        Op::CallInd(Opd::Mem(MemRef {
            base: None,
            index: None,
            scale: 1,
            disp: 0x2100
        }))
    );
}

#[test]
fn sib_and_prefixes() {
    // mov eax,[esp+8]
    let i = dec(&[0x8b, 0x44, 0x24, 0x08]);
    assert_eq!(
        i.op,
        Op::Mov {
            size: Size::D,
            dst: Opd::Reg(EAX),
            src: Opd::Mem(MemRef {
                base: Some(ESP),
                index: None,
                scale: 1,
                disp: 8
            })
        }
    );
    let i = dec(&[0xf3, 0xa4]);
    assert_eq!(
        i.op,
        Op::Str {
            op: StrOp::Movs,
            size: Size::B,
            rep: Rep::E
        }
    );
    let i = dec(&[0x66, 0xb8, 0x34, 0x12]);
    assert_eq!(
        i.op,
        Op::Mov {
            size: Size::W,
            dst: Opd::Reg(EAX),
            src: Opd::Imm(0x1234)
        }
    );
}

#[test]
fn unsupported_reports_bytes() {
    let mut m = Memory::new(0x3000);
    m.write_bytes(0x2000, &[0x0f, 0x0b, 0x90]).unwrap(); // ud2
    match decode(&m, 0x2000) {
        Err(Fault::BadOpcode { eip, bytes }) => {
            assert_eq!(eip, 0x2000);
            assert_eq!(&bytes[..3], &[0x0f, 0x0b, 0x90]);
        }
        other => panic!("{other:?}"),
    }
}
