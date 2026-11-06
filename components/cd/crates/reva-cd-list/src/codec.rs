//! One operation of the display list in text form, and read back.

use crate::draw::Op;

/// A point as two fields.
fn xy(p: &(i32, i32)) -> String {
    format!("{} {}", p.0, p.1)
}

/// One operation in text form.
pub(crate) fn item(op: &Op) -> String {
    match op {
        Op::Clear(c) => format!("c {c:06x}"),
        Op::Line {
            from,
            to,
            color,
            style,
            width,
        } => {
            format!("l {} {} {color:06x} {style} {width}", xy(from), xy(to))
        }
        Op::Box {
            lo,
            hi,
            color,
            filled,
        } => {
            format!(
                "{} {} {} {color:06x}",
                if *filled { "b" } else { "r" },
                xy(lo),
                xy(hi)
            )
        }
        Op::Mark {
            at,
            kind,
            size,
            color,
        } => format!("m {} {kind} {size} {color:06x}", xy(at)),
        _ => more(op),
    }
}

/// Texts, images and polygons in text form.
fn more(op: &Op) -> String {
    match op {
        Op::Text {
            at,
            color,
            size,
            text,
        } => {
            let text = text.replace('\\', "\\\\").replace(';', r"\;");
            format!("t {} {color:06x} {size} {text}", xy(at))
        }
        Op::Image {
            at,
            size,
            w,
            h,
            pixels,
        } => {
            let px: String = pixels.iter().map(|p| format!("{p:06x}")).collect();
            format!("i {} {} {w} {h} {px}", xy(at), xy(size))
        }
        Op::Poly {
            points,
            color,
            filled,
        } => {
            let pts: Vec<String> = points.iter().map(xy).collect();
            format!(
                "{} {color:06x} {}",
                if *filled { "P" } else { "p" },
                pts.join(" ")
            )
        }
        _ => unreachable!("item takes the others"),
    }
}

/// One item in text form, read back.
pub(crate) fn parse(item: &str) -> Option<Op> {
    let (verb, rest) = item.split_once(' ')?;
    let f: Vec<&str> = rest.split(' ').collect();
    let n = |i: usize| f.get(i)?.parse::<i32>().ok();
    let c = |i: usize| u32::from_str_radix(f.get(i)?, 16).ok();
    let p = |i: usize| Some((n(i)?, n(i + 1)?));
    Some(match verb {
        "c" => Op::Clear(c(0)?),
        "l" => Op::Line {
            from: p(0)?,
            to: p(2)?,
            color: c(4)?,
            style: n(5)?,
            width: n(6)?,
        },
        "b" | "r" => Op::Box {
            lo: p(0)?,
            hi: p(2)?,
            color: c(4)?,
            filled: verb == "b",
        },
        "m" => Op::Mark {
            at: p(0)?,
            kind: n(2)?,
            size: n(3)?,
            color: c(4)?,
        },
        "t" => Op::Text {
            at: p(0)?,
            color: c(2)?,
            size: n(3)?,
            text: f.get(4..)?.join(" "),
        },
        "i" => image(&f)?,
        "p" | "P" => Op::Poly {
            points: (1..f.len()).step_by(2).filter_map(p).collect(),
            color: c(0)?,
            filled: verb == "P",
        },
        _ => return None,
    })
}

/// An image's fields read back: at, size, w, h, then the pixels' hex.
fn image(f: &[&str]) -> Option<Op> {
    let n = |i: usize| f.get(i)?.parse::<i32>().ok();
    let hex = f.get(6).copied().unwrap_or("");
    let pixels = (0..hex.len() / 6)
        .filter_map(|k| u32::from_str_radix(hex.get(6 * k..6 * k + 6)?, 16).ok())
        .collect();
    Some(Op::Image {
        at: (n(0)?, n(1)?),
        size: (n(2)?, n(3)?),
        w: n(4)?,
        h: n(5)?,
        pixels,
    })
}
