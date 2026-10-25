//! API-level behaviour that the CLI regression suite cannot observe:
//! resumability, budgets, images in memory, and the translator hooks.

use std::cell::RefCell;
use std::rc::Rc;

use reva_vm::translate::{decode_block, Block, NativeHandle, Translator};
use reva_vm::x86::{Alu, MulDiv, Op};
use reva_vm::{embedded, BufferHost, Machine, Status};

fn booted() -> (Machine, BufferHost) {
    let mut vm = Machine::new(&["reva", embedded::BOOT_PATH]);
    let mut host = BufferHost::new();
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    let out = host.take_output();
    assert_eq!(
        embedded::split_boot_output(&out),
        ("123645789ABCDE", "Reva 2011.2 Linux\n\nok> "),
        "boot printed something unexpected"
    );
    vm.quiet_prompt();
    (vm, host)
}

fn eval(vm: &mut Machine, host: &mut BufferHost, src: &str) -> String {
    vm.feed(src.as_bytes());
    vm.feed(b"\n");
    assert_eq!(vm.run_until_blocked(host), Status::NeedInput);
    host.take_output()
}

#[test]
fn input_can_arrive_in_pieces() {
    let (mut vm, mut host) = booted();
    vm.feed(b"2 3 ");
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    assert_eq!(
        host.take_output(),
        "",
        "nothing runs before the line is complete"
    );
    vm.feed(b"+ .\n");
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    assert_eq!(host.take_output(), "5 ");
}

#[test]
fn key_pauses_inside_a_word_and_resumes() {
    let (mut vm, mut host) = booted();
    assert_eq!(eval(&mut vm, &mut host, ": k2 key key + . ;"), "");
    vm.feed(b"k2\n");
    // The line "k2" is consumed; key now waits for more input.
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    vm.feed(b"\x01");
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    vm.feed(b"\x02");
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    assert_eq!(host.take_output(), "3 ");
}

#[test]
fn budgets_yield_and_infinite_loops_stay_interruptible() {
    let (mut vm, mut host) = booted();
    vm.feed(b": forever forever ; forever\n");
    // `forever` compiles to `jmp forever`: it never needs input or halts.
    for _ in 0..3 {
        assert_eq!(vm.run(&mut host, 10_000), Status::Yielded);
    }
}

#[test]
fn images_round_trip_in_memory() {
    let (mut vm, mut host) = booted();
    eval(
        &mut vm,
        &mut host,
        ": answer 6 7 * ; variable keep 99 keep !",
    );
    let img = vm.save_image();
    let mut vm2 = Machine::load_image(&img, &["reva"]).expect("load");
    let mut host2 = BufferHost::new();
    assert_eq!(vm2.run_until_blocked(&mut host2), Status::NeedInput);
    // An image starts like Reva's saved executable: banner, then the
    // interpreter (the prompt stays as the saved system had it: quiet).
    assert_eq!(host2.take_output(), "Reva 2011.2 Linux\n");
    assert_eq!(eval(&mut vm2, &mut host2, "answer . keep @ ."), "42 99 ");
    assert!(Machine::load_image(&img[..20], &[]).is_err());
}

#[test]
fn compiled_code_is_the_ia32_reva_emitted() {
    let (mut vm, mut host) = booted();
    eval(&mut vm, &mut host, ": sq dup * ;");
    let xt = vm.find("sq").unwrap();
    let block = decode_block(vm.memory(), xt, 16).unwrap();
    let ops: Vec<Op> = block.insns.iter().map(|(_, i)| i.op).collect();
    // Both words are `;inline` in reva.f, so their bytes are copied in:
    // dup = lea esi,[esi-4]; mov [esi],eax   * = mul dword [esi]; add esi,4
    assert!(matches!(ops[0], Op::Lea { .. }), "{ops:?}");
    assert!(matches!(ops[1], Op::Mov { .. }));
    assert!(matches!(
        ops[2],
        Op::MulDiv {
            op: MulDiv::Mul,
            ..
        }
    ));
    assert!(matches!(ops[3], Op::Alu { op: Alu::Add, .. }));
    assert_eq!(ops[4], Op::Ret(0));
    assert!(block.ends_with_branch);

    // A word that calls a non-inline word ends in a tail jump instead.
    eval(&mut vm, &mut host, ": sq. sq . ;");
    let block = decode_block(vm.memory(), vm.find("sq.").unwrap(), 16).unwrap();
    assert!(matches!(block.insns[0].1.op, Op::Call(_)));
    let block = decode_block(vm.memory(), block.end, 16).unwrap();
    assert!(matches!(block.insns[0].1.op, Op::Jmp(_)), "{block:?}");
}

#[derive(Default)]
struct Recorder {
    invalidated: Vec<u32>,
}

struct Shared(Rc<RefCell<Recorder>>);

impl Translator for Shared {
    fn name(&self) -> &'static str {
        "recorder"
    }
    fn translate(&mut self, _block: &Block) -> Option<NativeHandle> {
        None
    }
    fn invalidate(&mut self, page_start: u32, _len: u32) {
        self.0.borrow_mut().invalidated.push(page_start);
    }
}

#[test]
fn writes_to_executed_code_reach_the_translator() {
    let (mut vm, mut host) = booted();
    let rec = Rc::new(RefCell::new(Recorder::default()));
    vm.set_translator(Box::new(Shared(rec.clone())));
    // Run a word, then patch its first byte from Forth: that page must be
    // reported, and the patched code must be what runs next time.
    eval(&mut vm, &mut host, ": five 5 ; five .");
    let xt = vm.find("five").unwrap();
    // `five` starts with `lea esi,[esi-4]`; overwrite the whole word with
    // `mov eax,7 ; ret` after pushing (dup) by hand.
    let patch = format!(
        "{xt} $8d swap c! {} $76 swap c! {} $fc swap c! {} $89 swap c! {} $06 swap c! \
         {} $b8 swap c! {} 7 swap c! {} 0 swap c! {} 0 swap c! {} 0 swap c! {} $c3 swap c! five .",
        xt + 1,
        xt + 2,
        xt + 3,
        xt + 4,
        xt + 5,
        xt + 6,
        xt + 7,
        xt + 8,
        xt + 9,
        xt + 10
    );
    // c! is ( c a -- ), so push the address first and swap.
    let out = eval(&mut vm, &mut host, &patch);
    assert_eq!(out, "7 ");
    let page = xt & !0xff;
    assert!(
        rec.borrow().invalidated.contains(&page),
        "{:?}",
        rec.borrow().invalidated
    );
}

#[test]
fn the_virtual_libc_reaches_the_host() {
    let (mut vm, mut host) = booted();
    host.env.insert("REVA_TEST".into(), "forty-two".into());
    assert_eq!(
        eval(&mut vm, &mut host, "\" REVA_TEST\" getenv type"),
        "forty-two"
    );
    eval(&mut vm, &mut host, "\" REVA_SET\" \" yes\" setenv");
    assert_eq!(host.env.get("REVA_SET").map(String::as_str), Some("yes"));
    // BufferHost has no clock: time&date reads as the Unix epoch, in UTC
    assert_eq!(
        eval(&mut vm, &mut host, "time&date . . . . . ."),
        "1970 1 1 0 0 0 "
    );
}

#[test]
fn file_words_reach_the_host_file_system() {
    let (mut vm, mut host) = booted();
    let run = |vm: &mut Machine, host: &mut BufferHost, s: &str| eval(vm, host, s);
    // creat / write / close, then open/r / fsize / read
    assert_eq!(
        run(
            &mut vm,
            &mut host,
            "\" /t\" creat value h ioerr @ . \" abc\" h write ioerr @ . h close"
        ),
        "0 0 "
    );
    assert_eq!(host.fs.get("/t"), Some(&b"abc"[..]));
    assert_eq!(
        run(
            &mut vm,
            &mut host,
            "\" /t\" open/r value r r fsize . pad 10 r read . pad 3 type r close"
        ),
        "3 3 abc"
    );
    // seek / tell on a read-write handle
    assert_eq!(
        run(
            &mut vm,
            &mut host,
            "\" /t\" open/rw value w 1 w seek \" X\" w write w tell . w close"
        ),
        "2 "
    );
    assert_eq!(host.fs.get("/t"), Some(&b"aXc"[..]));
    // failure: ioerr is -errno (ENOENT), and the fd is the same value
    assert_eq!(
        run(&mut vm, &mut host, "\" /nope\" open/r . ioerr @ ."),
        "-2 -2 "
    );
    // stat (mode word), rename (reva.f's takes one extra cell), delete
    assert_eq!(
        run(
            &mut vm,
            &mut host,
            "\" /t\" stat $ffff and . \
                                 0 \" /t\" \" /u\" rename ioerr @ . \" /u\" delete ioerr @ ."
        ),
        "33188 0 0 "
    );
    assert_eq!(host.fs.get("/u"), None);
    // directories: mkdir, chdir, getcwd, relative paths, opendir
    assert_eq!(
        run(
            &mut vm,
            &mut host,
            "needs os/dir \" /d\" mkdir \" /d\" chdir getcwd type space \
                                 \" f\" creat close \" /d\" opendir closedir type ."
        ),
        "/d .0 "
    );
    assert_eq!(host.fs.get("/d/f"), Some(&b""[..]));
    // `include` reads the same file system
    host.fs.put("/d/g.f", b": g 6 7 * ;");
    assert_eq!(run(&mut vm, &mut host, "include g.f g ."), "42 ");
    // what changed, for a host that persists files
    let mut changed: Vec<String> = host.fs.take_changes().into_iter().map(|(p, _)| p).collect();
    changed.sort();
    assert_eq!(changed, ["/d/f", "/t", "/u"]);
    // the console descriptors
    assert_eq!(
        run(&mut vm, &mut host, "\" out\" 1 write ioerr @ ."),
        "out0 "
    );
}

#[test]
fn interrupt_stops_a_loop_and_keeps_the_dictionary() {
    let (mut vm, mut host) = booted();
    vm.feed(b": kept 42 ; : spin repeat again ; spin\n");
    assert_eq!(vm.run(&mut host, 100_000), Status::Yielded);
    vm.feed(b"this queued line is dropped\n");
    vm.interrupt();
    assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
    let out = host.take_output();
    assert!(out.contains("*** interrupted (in spin"), "{out}");
    assert!(!out.contains("dropped"), "{out}");
    assert_eq!(eval(&mut vm, &mut host, "kept ."), "42 ");
}

#[test]
fn a_missing_native_library_is_named_once() {
    let (mut vm, mut host) = booted();
    let out = eval(
        &mut vm,
        &mut host,
        "\" libnothere.so\" lib nothere 1 func: f1 1 func: f2 f1",
    );
    assert!(
        out.starts_with("note: native library libnothere.so is not available"),
        "{out}"
    );
    assert!(out.contains("could not load f1"), "{out}");
    let again = eval(&mut vm, &mut host, "f2");
    assert!(!again.contains("note:"), "{again}");
    assert!(again.contains("could not load f2"), "{again}");
}

#[test]
fn shell_needs_a_host_that_allows_it() {
    let (mut vm, mut host) = booted();
    let out = eval(&mut vm, &mut host, "needs os/shell \" ls\" shell 7 .");
    assert!(
        out.contains("shell (running an OS command) is not allowed here"),
        "{out}"
    );
    // the fault ends the line; the session goes on
    assert_eq!(eval(&mut vm, &mut host, "7 ."), "7 ");
}
