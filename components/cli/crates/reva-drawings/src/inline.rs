//! Images inline in the terminal (iTerm2's protocol, also WezTerm's).

/// Whether the terminal shows images inline: iTerm2 or WezTerm, not
/// inside tmux or screen (which do not pass the escape on by default).
pub fn images_inline() -> bool {
    let env = |n: &str| std::env::var(n).unwrap_or_default();
    let term = env("TERM_PROGRAM");
    let known = matches!(term.as_str(), "iTerm.app" | "WezTerm") || env("LC_TERMINAL") == "iTerm2";
    known && env("TMUX").is_empty() && !env("TERM").starts_with("screen")
}

/// The escape that shows image file `name` (`bytes`) inline, half the
/// terminal's width, then a new line.
pub fn inline_image(name: &str, bytes: &[u8]) -> String {
    format!(
        "\x1b]1337;File=name={};size={};width=50%;inline=1:{}\x07\n",
        base64(name.as_bytes()),
        bytes.len(),
        base64(bytes)
    )
}

/// Standard base64, padded.
pub fn base64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = c
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
        for i in 0..4 {
            let ch = if i <= c.len() {
                A[(n >> (18 - 6 * i) & 63) as usize]
            } else {
                b'='
            };
            s.push(ch as char);
        }
    }
    s
}
