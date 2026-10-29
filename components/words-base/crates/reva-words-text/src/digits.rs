//! Digits, signs and base prefixes of Reva's number syntax.

/// The bases of the prefixes `#` `$` `%` `&` (and `'`, a character).
const BASES: [u32; 5] = [10, 16, 2, 8, 255];
const DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// What follows a number's sign.
#[derive(Debug, PartialEq, Eq)]
pub enum Prefix<'a> {
    /// `'c`: the character's code.
    Char(u32),
    /// Digits in a base (a prefix's, or the current one).
    Digits(u32, &'a [u8]),
}

/// The value of digit `c` (`0-9`, `A-Z`, `a-z`) if it is below `base`.
pub fn digit_value(c: u32, base: u32) -> Option<u32> {
    let v = match c {
        0x30..=0x39 => c - 0x30,
        0x41..=0x5a => c - 0x41 + 10,
        0x61..=0x7a => c - 0x61 + 10,
        _ => return None,
    };
    (v < base).then_some(v)
}

/// A leading `-` and the rest; `None` for an empty number or a lone `-`.
pub fn sign_split(s: &[u8]) -> Option<(bool, &[u8])> {
    match s.split_first()? {
        (b'-', []) => None,
        (b'-', rest) => Some((true, rest)),
        _ => Some((false, s)),
    }
}

/// A base prefix (`#$%&`, and `'` when `with_char`), or the digits as
/// they are; `None` when a prefix has nothing after it.
pub fn number_prefix(s: &[u8], base: u32, with_char: bool) -> Option<Prefix<'_>> {
    let top = if with_char { b'\'' } else { b'&' };
    if !(b'#'..=top).contains(&s[0]) {
        return Some(Prefix::Digits(base, s));
    }
    let k = (s[0] - b'#') as usize;
    if k == 4 {
        return s.get(1).map(|&c| Prefix::Char(c as u32));
    }
    let rest = &s[1..];
    (!rest.is_empty()).then_some(Prefix::Digits(BASES[k], rest))
}

/// `PROC todigit` (`>digit`): the digit for `n`; `None` (a divide error)
/// when `n` is 16 or more and the base is 0.
pub fn to_digit(n: u32, base: u32) -> Option<u32> {
    if n < 16 {
        return Some(DIGITS[n as usize] as u32);
    }
    if base == 0 {
        return None;
    }
    let c = n % base + 0x30;
    Some(if c > 0x39 { c + 39 } else { c })
}
