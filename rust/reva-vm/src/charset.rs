//! Character encodings for the virtual iconv (iconv.rs): names as iconv
//! spells them, and one character at a time in and out.  The standard
//! library is enough for the Unicode forms, Latin-1 and ASCII, which is
//! what Reva's libraries convert between (lib/string/iconv: UTF-8 and
//! UCS-2LE); an encoding not listed fails at `iconv_open`, as iconv does.

/// An encoding iconv can open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enc {
    Utf8,
    Utf16 {
        le: bool,
    },
    /// UTF-16 without surrogates: characters above U+FFFF do not fit.
    Ucs2 {
        le: bool,
    },
    Utf32 {
        le: bool,
    },
    Latin1,
    Ascii,
}

/// Why a character could not be decoded.
#[derive(Debug, PartialEq, Eq)]
pub enum Bad {
    /// The input ends inside a character (iconv's EINVAL).
    Incomplete,
    /// The bytes are not a character in this encoding (EILSEQ).
    Invalid,
}

/// The encoding iconv calls `name`, ignoring case, `-` and `_`, and a
/// `//TRANSLIT` or `//IGNORE` suffix.  UTF-16, UCS-2 and UTF-32 without
/// an endianness are big-endian, as libiconv writes them.
pub fn parse(name: &str) -> Option<Enc> {
    let base = name.split("//").next().unwrap_or("");
    let n: String = base
        .chars()
        .filter(|c| !matches!(c, '-' | '_'))
        .collect::<String>()
        .to_ascii_uppercase();
    Some(match n.as_str() {
        "UTF8" => Enc::Utf8,
        "UTF16" | "UTF16BE" => Enc::Utf16 { le: false },
        "UTF16LE" => Enc::Utf16 { le: true },
        "UCS2" | "UCS2BE" | "UNICODEBIG" => Enc::Ucs2 { le: false },
        "UCS2LE" | "UNICODELITTLE" => Enc::Ucs2 { le: true },
        "UTF32" | "UTF32BE" | "UCS4" | "UCS4BE" => Enc::Utf32 { le: false },
        "UTF32LE" | "UCS4LE" => Enc::Utf32 { le: true },
        "ISO88591" | "LATIN1" | "L1" | "CP819" | "ISO885911987" => Enc::Latin1,
        "ASCII" | "USASCII" | "ANSIX3.41968" | "646" => Enc::Ascii,
        _ => return None,
    })
}

/// The first character of `b` and how many bytes it took.
pub fn decode(enc: Enc, b: &[u8]) -> Result<(char, usize), Bad> {
    let unit = |n: usize, le: bool| -> Result<u32, Bad> {
        let s = b.get(..n).ok_or(Bad::Incomplete)?;
        let v = s.iter().fold(0u32, |a, &x| (a << 8) | u32::from(x));
        Ok(if le {
            v.swap_bytes() >> (32 - 8 * n as u32)
        } else {
            v
        })
    };
    let ch = |v: u32| char::from_u32(v).ok_or(Bad::Invalid);
    match enc {
        Enc::Utf8 => utf8(b),
        Enc::Latin1 => Ok((char::from(*b.first().ok_or(Bad::Incomplete)?), 1)),
        Enc::Ascii => match b.first() {
            Some(&c) if c < 0x80 => Ok((char::from(c), 1)),
            Some(_) => Err(Bad::Invalid),
            None => Err(Bad::Incomplete),
        },
        Enc::Utf32 { le } => Ok((ch(unit(4, le)?)?, 4)),
        Enc::Ucs2 { le } => Ok((ch(unit(2, le)?)?, 2)),
        Enc::Utf16 { le } => {
            let hi = unit(2, le)?;
            if !(0xd800..0xdc00).contains(&hi) {
                return Ok((ch(hi)?, 2));
            }
            let lo = match b.get(2..4) {
                Some(_) => unit(4, le).map(|v| if le { v >> 16 } else { v & 0xffff })?,
                None => return Err(Bad::Incomplete),
            };
            if !(0xdc00..0xe000).contains(&lo) {
                return Err(Bad::Invalid);
            }
            Ok((ch(0x10000 + ((hi - 0xd800) << 10) + (lo - 0xdc00))?, 4))
        }
    }
}

fn utf8(b: &[u8]) -> Result<(char, usize), Bad> {
    let first = *b.first().ok_or(Bad::Incomplete)?;
    let n = match first {
        0x00..=0x7f => 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return Err(Bad::Invalid),
    };
    let s = b.get(..n).ok_or_else(|| {
        // an incomplete tail is only Incomplete if what is there is valid
        let ok = b[1..].iter().all(|&c| c & 0xc0 == 0x80);
        if ok { Bad::Incomplete } else { Bad::Invalid }
    })?;
    let c = std::str::from_utf8(s).map_err(|_| Bad::Invalid)?;
    Ok((c.chars().next().ok_or(Bad::Invalid)?, n))
}

/// `c` in `enc`, or `None` when the encoding has no such character.
pub fn encode(enc: Enc, c: char) -> Option<Vec<u8>> {
    let order = |v: Vec<u8>, le: bool| if le { v.into_iter().rev().collect() } else { v };
    let u = u32::from(c);
    match enc {
        Enc::Utf8 => Some(c.to_string().into_bytes()),
        Enc::Latin1 => (u < 0x100).then(|| vec![u as u8]),
        Enc::Ascii => (u < 0x80).then(|| vec![u as u8]),
        Enc::Utf32 { le } => Some(order(u.to_be_bytes().to_vec(), le)),
        Enc::Ucs2 { le } => (u < 0x10000).then(|| order((u as u16).to_be_bytes().to_vec(), le)),
        Enc::Utf16 { le } => {
            let mut units = [0u16; 2];
            Some(
                c.encode_utf16(&mut units)
                    .iter()
                    .flat_map(|w| order(w.to_be_bytes().to_vec(), le))
                    .collect(),
            )
        }
    }
}
