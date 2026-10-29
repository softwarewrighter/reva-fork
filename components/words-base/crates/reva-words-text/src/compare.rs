//! `compare`, `comparei` and `fnvhash` over byte strings.

/// The order of two lengths, as `compare` gives it for empty strings.
pub fn length_order(n1: u32, n2: u32) -> i32 {
    match (n1 as i32).cmp(&(n2 as i32)) {
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
        std::cmp::Ordering::Less => -1,
    }
}

/// The difference of the first bytes that differ (case-folded when
/// `fold`), or `None` when the strings agree.
pub fn first_difference(s1: &[u8], s2: &[u8], fold: bool) -> Option<i32> {
    let f = |c: u8| if fold { c.to_ascii_lowercase() } else { c };
    s1.iter()
        .zip(s2)
        .find(|(x, y)| f(**x) != f(**y))
        .map(|(x, y)| f(*x).wrapping_sub(f(*y)) as i8 as i32)
}

/// `PROC fnvhash`: 32-bit FNV-1a; the offset basis for an empty string.
pub fn fnv_hash(bytes: &[u8]) -> u32 {
    bytes.iter().fold(2_166_136_261, |h: u32, &b| {
        (h ^ b as u32).wrapping_mul(16_777_619)
    })
}
