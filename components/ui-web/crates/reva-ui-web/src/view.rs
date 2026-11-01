//! The element: the grid as rows of spans, a class per colour role (the
//! page's stylesheet colours them with its theme), the cursor a span of
//! its own; keys and clicks go back to the VM as event lines.

use std::rc::Rc;

use reva_ui_grid::grid::{Grid, Role};
use reva_ui_paint::paint::{click, paint};
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::keys::{clipboard_key, key_code, pasted};
use crate::state::{send, with};

/// The stylesheet class of a role.
fn class(role: Role) -> &'static str {
    match role {
        Role::Screen => "tui-screen",
        Role::Dialog => "tui-dialog",
        Role::Border => "tui-border",
        Role::Title => "tui-title",
        Role::Text => "tui-text",
        Role::Button => "tui-button",
        Role::Focus => "tui-focus",
        Role::Active => "tui-active",
        Role::Field => "tui-field",
        Role::Selection => "tui-selection",
        Role::Disabled => "tui-disabled",
    }
}

/// Row `r` of `g` as spans: runs of one role, the cursor apart.
fn row(g: &Grid, r: u16) -> Html {
    let mut spans = Vec::new();
    let (mut text, mut role, mut cursor) = (String::new(), None, false);
    for c in 0..g.cols {
        let cell = g.cell(c, r).copied().expect("on the grid");
        let here = g.cursor == Some((c, r));
        if role != Some(cell.role) || here || cursor {
            if let Some(ro) = role.filter(|_| !text.is_empty()) {
                spans.push(html! { <span class={classes!(class(ro), cursor.then_some("tui-cursor"))}>{ std::mem::take(&mut text) }</span> });
            }
            (role, cursor) = (Some(cell.role), here);
        }
        text.push(cell.ch);
    }
    if let Some(ro) = role {
        spans.push(html! { <span class={classes!(class(ro), cursor.then_some("tui-cursor"))}>{ text }</span> });
    }
    html! { <div class="tui-row">{ for spans }</div> }
}

/// A key: its IUP code to the VM, the browser's own use of it prevented.
fn on_key(e: KeyboardEvent) {
    if clipboard_key(&e.key(), e.ctrl_key() || e.meta_key()) {
        return;
    }
    if let Some(code) = key_code(
        &e.key(),
        e.shift_key(),
        e.ctrl_key() || e.meta_key(),
        e.alt_key(),
    ) {
        e.prevent_default();
        send(vec![format!("key {code}")]);
    }
}

/// A paste: the clipboard's text typed into the focused element.
fn on_paste(e: Event) {
    let Some(text) = e
        .dyn_ref::<web_sys::ClipboardEvent>()
        .and_then(|c| c.clipboard_data())
        .and_then(|d| d.get_data("text/plain").ok())
    else {
        return;
    };
    e.prevent_default();
    send(pasted(&text));
}

/// A click: the cell under it, from the element's size in cells.
fn on_click(e: MouseEvent) {
    let Some(el) = e
        .current_target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let r = el.get_bounding_client_rect();
    let lines = with(|s| {
        let size = s.size;
        let col = ((f64::from(e.client_x()) - r.left()) * f64::from(size.cols) / r.width().max(1.0))
            as u16;
        let row = ((f64::from(e.client_y()) - r.top()) * f64::from(size.rows) / r.height().max(1.0))
            as u16;
        click(&s.mirror, size, col, row)
    });
    send(lines);
}

/// The TUI element.
#[function_component]
pub fn Tui() -> Html {
    let update = use_force_update();
    use_effect_with((), move |_| {
        with(|s| s.redraw = Some(Rc::new(move || update.force_update())));
    });
    let g = with(|s| paint(&s.mirror, s.size));
    html! {
        <div class="tui" tabindex="0" onkeydown={Callback::from(on_key)} onmousedown={Callback::from(on_click)} onpaste={Callback::from(on_paste)}>
            { for (0..g.rows).map(|r| row(&g, r)) }
        </div>
    }
}
