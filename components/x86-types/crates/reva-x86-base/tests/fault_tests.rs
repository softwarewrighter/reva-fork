use reva_x86_base::class::insn_class;
use reva_x86_base::fault::Fault;

#[test]
fn instruction_classes() {
    assert_eq!(insn_class(&[0xd9, 0xe8]), "x87 floating point"); // fld1
    assert_eq!(insn_class(&[0x0f, 0x6f, 0xc1]), "MMX"); // movq mm0, mm1
    assert_eq!(insn_class(&[0x66, 0x0f, 0x6f, 0xc1]), "SSE2"); // movdqa
    assert_eq!(insn_class(&[0x0f, 0x28, 0xc1]), "SSE"); // movaps
    assert_eq!(insn_class(&[0xe4, 0x60]), "I/O port (privileged)"); // in al, 0x60
    assert_eq!(insn_class(&[0xfa]), "privileged"); // cli
    assert_eq!(insn_class(&[0x64, 0x8b, 0x00]), "segment override"); // mov eax, fs:[eax]
    assert_eq!(
        insn_class(&[0x0f, 0xa2]),
        "processor identification (cpuid/rdtsc)"
    );
    assert_eq!(insn_class(&[0x0f, 0x0b]), "undefined instruction (ud2)");
    assert_eq!(insn_class(&[0xd6]), "x86");
}

#[test]
fn bad_opcode_message_names_the_class_and_the_doc() {
    let f = Fault::BadOpcode {
        eip: 0x2000,
        bytes: vec![0xd9, 0xe8],
    };
    assert_eq!(
        f.to_string(),
        "unsupported x87 floating point instruction at 0x00002000: d9 e8 \
         (see docs/portability.md)"
    );
}

#[test]
fn messages() {
    assert_eq!(Fault::StackUnderflow.to_string(), "stack underflow");
    assert_eq!(
        Fault::BadAddress { addr: 4, len: 2 }.to_string(),
        "bad memory access: 2 byte(s) at 0x00000004"
    );
    assert_eq!(
        Fault::NotImplemented("x").to_string(),
        "'x' is not implemented in this port yet"
    );
}
