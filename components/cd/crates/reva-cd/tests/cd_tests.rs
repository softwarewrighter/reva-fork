use reva_cd::calls::{CdFn, Done, Memory};
use reva_cd::draw::{Op, decode, encode};
use reva_cd::state::Cd;
use std::collections::HashMap;

/// Memory for the tests: byte strings at addresses.
#[derive(Default)]
struct Mem(HashMap<u32, Vec<u8>>);

impl Memory for Mem {
    fn bytes(&self, a: u32, n: u32) -> Option<Vec<u8>> {
        let (start, b) = self
            .0
            .iter()
            .find(|(s, b)| (**s..**s + b.len() as u32).contains(&a))?;
        let from = (a - start) as usize;
        Some(
            b[from..]
                .iter()
                .copied()
                .chain(std::iter::repeat(0))
                .take(n as usize)
                .collect(),
        )
    }
}

fn call(cd: &mut Cd, f: CdFn, a: &[u32], mem: &Mem) -> u32 {
    cd.call(f, a, mem).value
}

#[test]
fn operations_survive_their_text_form() {
    let ops = vec![
        Op::Clear(0x808080),
        Op::Line {
            from: (0, 0),
            to: (30, 30),
            color: 0x0000ff,
            style: 2,
            width: 1,
        },
        Op::Box {
            lo: (60, 0),
            hi: (90, 20),
            color: 0xff0000,
            filled: true,
        },
        Op::Mark {
            at: (40, 10),
            kind: 5,
            size: 10,
            color: 0,
        },
        Op::Text {
            at: (100, 10),
            color: 0xff0000,
            size: 12,
            text: "a; b\\c".into(),
        },
        Op::Image {
            at: (70, 70),
            size: (32, 32),
            w: 2,
            h: 1,
            pixels: vec![0x0000ff, 0xffff00],
        },
        Op::Poly {
            points: vec![(0, 0), (10, 0), (5, 8)],
            color: 0x00ff00,
            filled: true,
        },
    ];
    assert_eq!(decode(&encode(&ops)), ops);
    assert!(decode("").is_empty());
}

#[test]
fn a_redraw_as_cd_test_does_it() {
    let (mut cd, mem) = (Cd::default(), Mem::default());
    let ctx = call(&mut cd, CdFn::ContextIup, &[], &mem);
    let c = call(&mut cd, CdFn::CreateCanvas, &[ctx, 7], &mem);
    assert_eq!(call(&mut cd, CdFn::Activate, &[c], &mem), 0);
    assert_eq!(
        call(&mut cd, CdFn::Activate, &[12345], &mem),
        u32::MAX,
        "an unknown canvas is CD_ERROR"
    );
    assert_eq!(call(&mut cd, CdFn::ActiveCanvas, &[], &mem), c);
    call(&mut cd, CdFn::Background, &[0x808080], &mem);
    call(&mut cd, CdFn::Clear, &[], &mem);
    assert_eq!(
        call(&mut cd, CdFn::Foreground, &[0x0000ff], &mem),
        0x000000,
        "the old foreground"
    );
    assert_eq!(
        call(&mut cd, CdFn::Foreground, &[u32::MAX], &mem),
        0x0000ff,
        "CD_QUERY"
    );
    call(&mut cd, CdFn::LineStyle, &[2], &mem);
    call(&mut cd, CdFn::Line, &[0, 0, 30, 30], &mem);
    call(&mut cd, CdFn::Box, &[60, 90, 0, 20], &mem);
    let flushed = cd.flush();
    assert_eq!(flushed.len(), 1);
    assert_eq!(flushed[0].0, 7, "the IUP element's DRAW");
    assert_eq!(
        decode(&flushed[0].1),
        vec![
            Op::Clear(0x808080),
            Op::Line {
                from: (0, 0),
                to: (30, 30),
                color: 0x0000ff,
                style: 2,
                width: 1
            },
            Op::Box {
                lo: (60, 0),
                hi: (90, 20),
                color: 0x0000ff,
                filled: true
            },
        ]
    );
    assert!(cd.flush().is_empty(), "clean until drawn again");
    call(&mut cd, CdFn::Clear, &[], &mem);
    assert_eq!(
        decode(&cd.flush()[0].1),
        vec![Op::Clear(0x808080)],
        "clear starts a new frame"
    );
}

#[test]
fn bitmaps_and_the_canvas_size() {
    let mut mem = Mem::default();
    mem.0.insert(1000, vec![1, 0, 0, 1]);
    let colors: Vec<u8> = [0x0000ffu32, 0xffff00]
        .iter()
        .flat_map(|c| c.to_le_bytes())
        .chain(std::iter::repeat_n(0, 254 * 4))
        .collect();
    mem.0.insert(2000, colors);
    mem.0.insert(6000, b"hi\0".to_vec());
    let mut cd = Cd::default();
    let c = call(&mut cd, CdFn::CreateCanvas, &[0, 9], &mem);
    call(&mut cd, CdFn::Activate, &[c], &mem);
    let b = call(&mut cd, CdFn::InitBitmap, &[2, 2, 1, 1000, 2000], &mem);
    call(&mut cd, CdFn::PutBitmap, &[b, 5, 6, 16, 16], &mem);
    call(&mut cd, CdFn::Text, &[1, 2, 6000], &mem);
    let ops = decode(&cd.flush()[0].1);
    assert_eq!(
        ops[0],
        Op::Image {
            at: (5, 6),
            size: (16, 16),
            w: 2,
            h: 2,
            pixels: vec![0xffff00, 0x0000ff, 0x0000ff, 0xffff00]
        }
    );
    assert!(matches!(&ops[1], Op::Text { text, .. } if text == "hi"));
    cd.sizes.insert(9, (300, 150));
    // lib/ui/cd declares 5 cells: the four pointers are the last ones
    let Done { writes, .. } = cd.call(CdFn::GetCanvasSize, &[777, 4000, 4004, 0, 0], &mem);
    assert_eq!(
        writes,
        vec![
            (4000, 300u32.to_le_bytes().to_vec()),
            (4004, 150u32.to_le_bytes().to_vec())
        ]
    );
}
