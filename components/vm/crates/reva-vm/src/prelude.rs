//! The traits that give a machine (and its canvas) their methods, for
//! `use reva_vm::prelude::*`: Rust allows `impl Machine` only in reva-core,
//! so the run loop, the dictionary, images and the rest are extension
//! traits in the crates above it (docs/design.md 3.13).

pub use reva_canvas_out::ops::CanvasOps;
pub use reva_canvas_out::svg::CanvasSvg;
pub use reva_core_boot::boot::Boot;
pub use reva_core_boot::image::Images;
pub use reva_core_ffi::args::CallArgs;
pub use reva_core_ffi::regs::MachineRegs;
pub use reva_core_inspect::config::Settings;
pub use reva_core_inspect::view::View;
pub use reva_core_inspect::words::Dictionary;
pub use reva_core_run::calls::HostCalls;
pub use reva_core_run::input::Input;
pub use reva_core_run::nested::Nested;
pub use reva_core_run::run::Run;
