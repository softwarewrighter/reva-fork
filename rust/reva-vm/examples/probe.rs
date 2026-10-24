use reva_vm::{BufferHost, Machine};
fn main() {
    let mut vm = Machine::new(&["reva", reva_vm::embedded::BOOT_PATH]);
    let mut host = BufferHost::new();
    let s = vm.run_until_blocked(&mut host);
    println!(
        "boot: {:?} out={:?} steps={}",
        s,
        host.take_output(),
        vm.steps
    );
    for line in std::env::args().skip(1) {
        if let Some(w) = line.strip_prefix("see ") {
            let xt = vm.find(w).expect("word");
            let mut a = xt;
            for _ in 0..40 {
                let b = reva_vm::translate::decode_block(vm.memory(), a, 1).unwrap();
                if b.insns.is_empty() {
                    println!("  {a:#x}: -> {}", vm.describe_addr(a));
                    break;
                }
                let (at, i) = b.insns[0];
                let extra = match i.op {
                    reva_vm::x86::Op::Call(t) | reva_vm::x86::Op::Jmp(t) => vm.describe_addr(t),
                    _ => String::new(),
                };
                println!("  {at:#x}: {:?} {extra}", i.op);
                a = b.end;
                if matches!(i.op, reva_vm::x86::Op::Ret(_)) {
                    break;
                }
            }
            continue;
        }
        vm.set_trace(std::env::var_os("TRACE").is_some());
        vm.feed(line.as_bytes());
        vm.feed(b"\n");
        let s = vm.run_until_blocked(&mut host);
        println!(
            "{line:30} => {:?} {:?} stack={:?}",
            host.take_output(),
            s,
            vm.data_stack()
        );
    }
}
