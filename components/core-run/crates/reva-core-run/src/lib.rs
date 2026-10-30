//! # reva-core-run — running the machine
//!
//! The machine executes vx86 instructions one at a time; when EIP lands in
//! the gateway region it runs the gate's Rust routine through the
//! machine's dispatch ([`run::Run`]).  Rust routines that call back into
//! Forth either return a continuation gate (the VM can then stop after any
//! instruction, as a browser needs) or, for short callbacks, make a nested
//! call ([`nested::Nested::call_nested`]).  Faults are reported and the
//! interpreter restarted ([`nested::Nested::recover`]); hosts feed input
//! ([`input`]) and answer host calls ([`calls`]).

pub mod calls;
pub mod input;
pub mod nested;
pub mod run;
