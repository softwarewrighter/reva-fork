//! Raw mode on Windows: `SetConsoleMode` with virtual terminal input.

use std::ffi::c_void;

type Handle = *mut c_void;
const STD_INPUT_HANDLE: u32 = -10i32 as u32;
const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
const ENABLE_PROCESSED_INPUT: u32 = 0x1;
const ENABLE_LINE_INPUT: u32 = 0x2;
const ENABLE_ECHO_INPUT: u32 = 0x4;
const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x200;
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x4;

#[repr(C)]
#[derive(Default)]
struct BufferInfo {
    size: [i16; 2],
    cursor: [i16; 2],
    attributes: u16,
    window: [i16; 4], // left, top, right, bottom
    max: [i16; 2],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetStdHandle(n: u32) -> Handle;
    fn GetConsoleMode(h: Handle, mode: *mut u32) -> i32;
    fn SetConsoleMode(h: Handle, mode: u32) -> i32;
    fn GetConsoleScreenBufferInfo(h: Handle, info: *mut BufferInfo) -> i32;
}

/// Raw (VT input) mode on the console until dropped.
pub struct Raw {
    input: u32,
    output: u32,
}

impl Raw {
    pub fn enter() -> Option<Raw> {
        // SAFETY: plain console API calls on the process's handles.
        unsafe {
            let (i, o) = (
                GetStdHandle(STD_INPUT_HANDLE),
                GetStdHandle(STD_OUTPUT_HANDLE),
            );
            let (mut input, mut output) = (0, 0);
            if GetConsoleMode(i, &mut input) == 0 || GetConsoleMode(o, &mut output) == 0 {
                return None;
            }
            let raw_in = (input
                & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT))
                | ENABLE_VIRTUAL_TERMINAL_INPUT;
            if SetConsoleMode(i, raw_in) == 0 {
                return None;
            }
            if SetConsoleMode(o, output | ENABLE_VIRTUAL_TERMINAL_PROCESSING) == 0 {
                SetConsoleMode(i, input);
                return None;
            }
            Some(Raw { input, output })
        }
    }
}

impl Drop for Raw {
    fn drop(&mut self) {
        // SAFETY: restores the modes read in `enter`.
        unsafe {
            SetConsoleMode(GetStdHandle(STD_INPUT_HANDLE), self.input);
            SetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), self.output);
        }
    }
}

/// Console columns.
pub fn width() -> Option<usize> {
    let mut info = BufferInfo::default();
    // SAFETY: fills the struct it is given.
    let ok = unsafe { GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut info) } != 0;
    let w = (info.window[2] - info.window[0] + 1) as usize;
    (ok && w > 0).then_some(w)
}
