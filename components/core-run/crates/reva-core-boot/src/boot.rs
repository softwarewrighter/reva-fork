//! New machines, arguments, the registers at `cold`.

use crate::tables::{Tables, unwired};
use reva_core::canvas::Canvas;
use reva_core::heap::Heap;
use reva_core::layout::*;
use reva_core::mem::Memory;
use reva_core::x86::{Cpu, ESI, ESP};
use reva_core::{Gate, Machine, st};
use std::collections::{HashMap, VecDeque};

reva_macros::machine_trait! {
    /// Making machines.
    pub trait Boot {
        /// A fresh machine.  `args` become `argc`/`(argv)`; as in Reva's core,
        /// `args[1]` (if present) is the file `appstart` includes before the
        /// interactive loop.  `reva_core::embedded::BOOT_PATH` boots `reva.f`.
        /// Its dispatch is unwired: reva-vm installs reva-words'.
        fn new(args: &[&str]) -> Machine {
            let mut m = Machine::with_memory(MEM_SIZE);
            m.init_statics();
            m.build_dictionary();
            m.set_args(args);
            m.reset_cpu();
            m
        }

        /// A machine with `mem_size` bytes of zeroed memory and nothing in it
        /// (the gateway region reads as `hlt`).
        fn with_memory(mem_size: u32) -> Machine {
            let mut mem = Memory::new(mem_size);
            // Gateway ROM reads as `hlt`, so a disassembler shows something sane.
            mem.fill(GATE_BASE, GATE_END - GATE_BASE, 0xf4)
                .expect("gateway region");
            Machine {
                dispatch: unwired,
                cpu: Cpu::default(),
                mem,
                cache: Default::default(),
                heap: Heap::new(HEAP_BASE, mem_size),
                input: VecDeque::new(),
                input_closed: false,
                halted: None,
                restart: false,
                nested: 0,
                translator: None,
                steps: 0,
                trace: false,
                last_pad: (0, 0),
                dirs: Vec::new(),
                interrupt: false,
                missing_lib: None,
                reported_libs: Vec::new(),
                canvas: Canvas::new(),
                pending_call: None,
                host_reply: None,
                sql_bufs: HashMap::new(),
                sql_ptr: 0,
                iconv_descs: Vec::new(),
            }
        }

        /// The registers as `cold` expects them.
        fn reset_cpu(&mut self) {
            self.cpu = Cpu {
                esp_floor: RSTACK_LO,
                ..Cpu::default()
            };
            self.cpu.r[ESI as usize] = S0;
            self.cpu.r[ESP as usize] = RP0;
            self.cpu.eip = Gate::Cold.addr();
        }

        /// Write `argc`/`(argv)`: a pointer array followed by the strings.
        fn set_args(&mut self, args: &[&str]) {
            let mut strings = ARGV_BASE + 4 * (args.len() as u32 + 1);
            for (i, a) in args.iter().enumerate() {
                let bytes = a.as_bytes();
                if strings + bytes.len() as u32 + 1 > ARGV_END {
                    break;
                }
                let m = &mut self.mem;
                m.write_u32(ARGV_BASE + 4 * i as u32, strings).expect("argv");
                m.write_bytes(strings, bytes).expect("argv");
                m.write_u8(strings + bytes.len() as u32, 0).expect("argv");
                strings += bytes.len() as u32 + 1;
            }
            self.mem
                .write_u32(ARGV_BASE + 4 * args.len() as u32, 0)
                .expect("argv");
            self.mem.write_u32(st::ARGC, args.len() as u32).expect("argc");
            self.mem.write_u32(st::ARGV, ARGV_BASE).expect("argv");
        }
    }
}
