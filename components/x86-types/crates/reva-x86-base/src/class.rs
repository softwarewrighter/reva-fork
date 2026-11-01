//! What kind of x86 code an unsupported instruction is.

/// What kind of x86 code `bytes` (an instruction, prefixes first) is, for
/// fault messages: names the family the portable core leaves out, so a
/// reader knows at once whether it is floating point, a vector extension,
/// a privileged instruction or a plain gap in the decoder.
pub fn insn_class(bytes: &[u8]) -> &'static str {
    let (i, sse_prefix) = match skip_prefixes(bytes) {
        Ok(at) => at,
        Err(class) => return class,
    };
    let Some(&op) = bytes.get(i) else {
        return "x86";
    };
    match op {
        0xd8..=0xdf | 0x9b => "x87 floating point",
        0xe4..=0xe7 | 0xec..=0xef | 0x6c..=0x6f => "I/O port (privileged)",
        0xfa | 0xfb | 0xf4 | 0xcf => "privileged",
        0x8e | 0x8c | 0x06 | 0x07 | 0x0e | 0x16 | 0x17 | 0x1e | 0x1f => "segment register",
        0xcc..=0xce => "software interrupt",
        0x0f => class_0f(bytes.get(i + 1).copied().unwrap_or(0), sse_prefix),
        _ => "x86",
    }
}

/// Past the prefixes: (opcode index, an SSE-style prefix seen), or the
/// class a prefix alone decides.
fn skip_prefixes(bytes: &[u8]) -> Result<(usize, bool), &'static str> {
    let mut i = 0;
    let mut sse_prefix = false;
    while let Some(&b) = bytes.get(i) {
        match b {
            0x26 | 0x2e | 0x36 | 0x3e | 0x64 | 0x65 => return Err("segment override"),
            0xf0 => return Err("lock prefix"),
            0x66 | 0xf2 | 0xf3 => {
                sse_prefix = true;
                i += 1;
            }
            _ => break,
        }
    }
    Ok((i, sse_prefix))
}

/// The class of a `0F`-escaped opcode `b`.
fn class_0f(b: u8, sse_prefix: bool) -> &'static str {
    match b {
        0x00 | 0x01 | 0x06 | 0x08 | 0x09 | 0x20..=0x23 | 0x30 | 0x32 | 0x33 => "privileged",
        0x0b => "undefined instruction (ud2)",
        0xa0 | 0xa1 | 0xa8 | 0xa9 => "segment register",
        0xa2 | 0x31 => "processor identification (cpuid/rdtsc)",
        0x10..=0x17 | 0x28..=0x2f | 0x50..=0x5f | 0xc2 | 0xc6 => "SSE",
        0x60..=0x7f | 0xd0..=0xff if sse_prefix => "SSE2",
        0x60..=0x7f | 0xd0..=0xff => "MMX",
        _ => "x86",
    }
}
