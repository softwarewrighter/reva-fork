//! The renderer's state, shared by the element and the page's calls.

use std::cell::RefCell;
use std::rc::Rc;

use reva_ui_layout::geom::Size;
use reva_ui_mirror::mirror::Mirror;

/// The model as the VM's changes describe it, the screen size in cells,
/// where events go (the page's function), and how to draw again.
pub struct State {
    pub mirror: Mirror,
    pub size: Size,
    pub on_event: Option<js_sys::Function>,
    pub redraw: Option<Rc<dyn Fn()>>,
    /// The size (pixels) each IUP canvas was last reported at.
    pub canvas_sizes: std::collections::BTreeMap<u32, (i32, i32)>,
    /// The dialog the last Ctrl-C asked to close (keys.rs `quit`).
    pub quit: Option<u32>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        mirror: Mirror::default(),
        size: Size { cols: 80, rows: 24 },
        on_event: None,
        redraw: None,
        canvas_sizes: std::collections::BTreeMap::new(),
        quit: None,
    });
}

/// `f` of the state.
pub fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

/// Send event lines to the page, which answers the VM's wait with them.
pub fn send(lines: Vec<String>) {
    let Some(f) = with(|s| s.on_event.clone()) else {
        return;
    };
    for line in lines {
        let _ = f.call1(&wasm_bindgen::JsValue::NULL, &line.into());
    }
}

/// Draw the element again.
pub fn redraw() {
    if let Some(r) = with(|s| s.redraw.clone()) {
        r();
    }
}
