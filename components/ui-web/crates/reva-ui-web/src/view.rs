//! The element: the grid as rows of spans, a class per colour role (the
//! page's stylesheet colours them with its theme), the cursor a span of
//! its own; keys and clicks go back to the VM as event lines.

use std::rc::Rc;

use reva_ui_grid::grid::{Grid, Role};
use reva_ui_layout::geom::Rect;
use reva_ui_paint::paint::{click, paint, room};
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::keys::{on_key, on_paste};
use crate::state::{redraw, send, with};

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

/// A click: the cell under it, from the grid element's size in cells.
/// The grid is measured itself: Yew delegates events, so the event's
/// current target is the mount root, a little larger than the grid, and
/// scaling by it put clicks a cell off (the close box missed).
fn on_click(grid: &NodeRef, e: &MouseEvent) {
    let Some(el) = grid.cast::<web_sys::Element>() else {
        return;
    };
    let r = el.get_bounding_client_rect();
    let lines = with(|s| {
        let size = room(&s.mirror, s.size);
        let col = ((f64::from(e.client_x()) - r.left()) * f64::from(size.cols) / r.width().max(1.0))
            as u16;
        let row = ((f64::from(e.client_y()) - r.top()) * f64::from(size.rows) / r.height().max(1.0))
            as u16;
        click(&s.mirror, size, col, row)
    });
    send(lines);
}

/// An HTML canvas over an IUP canvas's cells (inside its frame), placed in
/// percent of the grid so it follows the element's size.
fn overlay(g: &Grid, h: u32, r: Rect) -> Html {
    let pct = |a: u16, of: u16| f64::from(a) * 100.0 / f64::from(of.max(1));
    let (cols, rows) = (
        r.cols.saturating_sub(2).max(1),
        r.rows.saturating_sub(2).max(1),
    );
    let style = format!(
        "left:{}%;top:{}%;width:{}%;height:{}%",
        pct(r.col + 1, g.cols),
        pct(r.row + 1, g.rows),
        pct(cols, g.cols),
        pct(rows, g.rows)
    );
    html! { <canvas class="tui-canvas" data-h={h.to_string()} style={style} /> }
}

/// Paint every IUP canvas's DRAW list; a canvas whose size changed is
/// reported to the VM (`size H W H`: RESIZE_CB, then its redraw).
fn paint_canvases() {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(list) = doc.query_selector_all(".tui-canvas") else {
        return;
    };
    let mut events = Vec::new();
    for i in 0..list.length() {
        let Some(el) = list
            .item(i)
            .and_then(|n| n.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        else {
            continue;
        };
        let Some(h) = el
            .get_attribute("data-h")
            .and_then(|v| v.parse::<u32>().ok())
        else {
            continue;
        };
        let draw = with(|s| s.mirror.tree.attr(h, "DRAW")).unwrap_or_default();
        let size = reva_ui_canvas2d::paint::paint(&el, &draw);
        if with(|s| s.canvas_sizes.insert(h, size)) != Some(size) {
            events.push(format!("size {h} {} {}", size.0, size.1));
        }
    }
    send(events);
}

/// The TUI element.
#[function_component]
pub fn Tui() -> Html {
    let update = use_force_update();
    use_effect_with((), move |_| {
        with(|s| s.redraw = Some(Rc::new(move || update.force_update())));
        // changes that came after the first render but before this effect
        // found no redraw to call: draw them now
        redraw();
    });
    use_effect(paint_canvases);
    let g = with(|s| paint(&s.mirror, room(&s.mirror, s.size)));
    let grid = use_node_ref();
    let onmousedown = {
        let grid = grid.clone();
        Callback::from(move |e: MouseEvent| on_click(&grid, &e))
    };
    html! {
        <div class="tui" ref={grid} tabindex="0" onkeydown={Callback::from(on_key)} {onmousedown} onpaste={Callback::from(on_paste)}>
            { for (0..g.rows).map(|r| row(&g, r)) }
            { for g.canvases.iter().map(|(h, r)| overlay(&g, *h, *r)) }
        </div>
    }
}
