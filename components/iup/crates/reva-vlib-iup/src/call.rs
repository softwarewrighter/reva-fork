//! `(call)` of an iup function: its arguments, read in place (a call that
//! waits for the host is retried), and what it does to the model.

use reva_core::Next;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::x86::ESI;
use reva_ui_base::keys::{from_name, name};
use reva_ui_base::kind::Kind;

use reva_core_ffi::args::CallArgs;

use crate::data::IupData;
use crate::names::{Args, Op, function};
use crate::run::IupRun;

type R<T = ()> = Result<T, Fault>;

/// The VM's memory as reva-cd reads it (strings, bitmaps).
struct View<'a>(&'a reva_core::mem::Memory);

impl reva_cd::calls::Memory for View<'_> {
    fn bytes(&self, a: u32, n: u32) -> Option<Vec<u8>> {
        self.0.slice(a, n).ok().map(<[u8]>::to_vec)
    }
}

reva_macros::machine_trait! {
    /// The virtual library `iup`.
    pub trait IupLib: IupRun {
        /// `(call)` of the function at address `f` with `n` cells: `None` if
        /// `f` is not one of iup's.
        fn iup_call(&mut self, host: &mut dyn Host, f: u32, n: u32) -> Option<R<Next>> {
            let (_, op) = function(f)?;
            Some(self.iup_run(host, op, n))
        }

        /// Do `op` with the `n` cells of its `(call)`.
        fn iup_run(&mut self, host: &mut dyn Host, op: Op, n: u32) -> R<Next> {
            let a = self.iup_args(n)?;
            let v = match op {
                Op::Show | Op::ShowXY | Op::Hide | Op::Popup | Op::MainLoop | Op::LoopStep => return self.iup_show(host, op, &a, n),
                Op::Zero => 0,
                Op::Default => -2i32 as u32,
                Op::Value(v) => v as u32,
                Op::Cd(f) => {
                    let done = self.ui.cd.call(f, &a, &View(&self.mem));
                    for (p, bytes) in done.writes {
                        self.mem.write_bytes(p, &bytes)?;
                    }
                    done.value
                }
                Op::New(kind, args) => self.iup_new(kind, args, &a)?,
                Op::SetAttr | Op::SetAttrId | Op::GetAttr | Op::GetAttrId => self.iup_attr(op, &a)?,
                _ => self.iup_other(op, &a)?,
            };
            self.ffi_finish(n, v)?;
            Ok(Next::Ret)
        }

        /// The `n` cells of a `(call)`, first C argument first, not popped.
        fn iup_args(&self, n: u32) -> R<Vec<u32>> {
            let esi = self.cpu.r[ESI as usize];
            (0..n).map(|i| self.mem.read_u32(esi + 4 * (n - i))).collect()
        }

        /// A new element of `kind`, given what `args` says of `a`.
        fn iup_new(&mut self, kind: Kind, args: Args, a: &[u32]) -> R<u32> {
            let h = self.ui.create(kind);
            let first = a.first().copied().unwrap_or(0);
            if matches!(args, Args::Title | Args::TitleChild)
                && let Some(t) = self.iup_zstr(first)?
            {
                self.ui.set(h, "TITLE", Some(&t));
            }
            let kids: Vec<u32> = match args {
                Args::Children => a.to_vec(),
                Args::TitleChild => a.get(1).copied().into_iter().collect(),
                Args::Array => (0..256).map(|i| self.mem.read_u32(first + 4 * i)).take_while(|c| !matches!(c, Ok(0))).collect::<R<_>>()?,
                _ => vec![],
            };
            for c in kids.into_iter().filter(|c| *c != 0) {
                self.ui.tree.append(h, c);
            }
            Ok(h)
        }

        /// Setting and getting attributes (element 0, IUP's global
        /// attributes, is not kept).
        fn iup_attr(&mut self, op: Op, a: &[u32]) -> R<u32> {
            let arg = |i: usize| a.get(i).copied().unwrap_or(0);
            let (ih, mut name) = (arg(0), self.iup_zstr(arg(1))?.unwrap_or_default());
            if matches!(op, Op::SetAttrId | Op::GetAttrId) {
                name = format!("{name}{}", arg(2) as i32);
            }
            match op {
                Op::SetAttr | Op::SetAttrId => {
                    let value = self.iup_zstr(arg(if op == Op::SetAttr { 2 } else { 3 }))?;
                    self.ui.set(ih, &name, value.as_deref());
                    Ok(0)
                }
                _ => {
                    let value = self.ui.get(ih, &name);
                    self.iup_out(ih, &name, value)
                }
            }
        }

        /// The other functions: structure, callbacks, keys, the version.
        fn iup_other(&mut self, op: Op, a: &[u32]) -> R<u32> {
            let arg = |i: usize| a.get(i).copied().unwrap_or(0);
            let (ih, t) = (arg(0), &self.ui.tree);
            let some = |h: Option<u32>| h.unwrap_or(0);
            Ok(match op {
                Op::Append => if self.ui.tree.append(ih, arg(1)) { ih } else { 0 },
                Op::Destroy => { self.ui.destroy(ih); 0 }
                Op::SetCallback => {
                    let name = self.iup_zstr(arg(1))?.unwrap_or_default();
                    some(self.ui.tree.set_callback(ih, &name, Some(arg(2)).filter(|f| *f != 0)))
                }
                Op::NextChild => some(t.next_child(ih, Some(arg(1)).filter(|c| *c != 0))),
                Op::DialogChild => {
                    let name = self.iup_zstr(arg(1))?.unwrap_or_default();
                    let t = &self.ui.tree;
                    some(t.dialog_path(ih).last().and_then(|d| t.named(*d, &name)))
                }
                Op::Parent => some(t.node(ih).and_then(|n| n.parent)),
                Op::Dialog => some(t.dialog_path(ih).last().copied().filter(|d| t.node(*d).is_some_and(|n| n.kind == Kind::Dialog))),
                Op::KeyName => self.iup_out(0, "KEYNAME", name(ih))?,
                Op::KeyCode => some(self.iup_zstr(ih)?.and_then(|n| from_name(&n))),
                Op::Version => self.iup_out(0, "VERSION", Some("3.0 (reva-fork)".into()))?,
                Op::ExitLoop => {
                    self.ui.closed = !self.ui.loops.is_empty();
                    0
                }
                _ => 0,
            })
        }
    }
}
