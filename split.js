// split.js — the split view (`?canvas=split`, or `"canvas": "split"` in a
// package manifest): the REPL on the left, the graphics pane on the right,
// both full height.
//
//   ┌──────── main ────────┬─┬──────── #side ───────┐
//   │ header               │ │ Graphics        [⟩]  │
//   │ output …             │▐│ #panes: canvas.html  │
//   │ ok> _                │ │   (iframe)           │
//   └──────────────────────┴─┴──────────────────────┘
//                        #divider
//
// The divider is a WAI-ARIA window splitter: drag it (pointer capture, so
// the iframe never swallows the drag), or focus it and use ←/→ (Shift: 10%
// steps), Home/End (the limits) and Enter (collapse/restore).  Double-click
// puts it back at 50%.  The REPL's share is clamped to MIN..MAX percent.
// The right pane collapses to a strip with a button that restores it.
// Position and collapsed state are kept in localStorage under `key`, per
// origin, so a page served from another host or port starts at the
// default; nothing breaks where storage is blocked.
//
// The page lays the split out with CSS (body.split, --split: the REPL's
// width); this module only moves #panes into the right pane and keeps the
// state.  The canvas page redraws itself when its pane changes size
// (canvas.html, ResizeObserver).

const MIN = 20, MAX = 80, HOME = 50;

const load = (key) => {
  try {
    const v = JSON.parse(localStorage.getItem(key) ?? "null");
    return v && typeof v.pos === "number" ? v : null;
  } catch {
    return null;
  }
};
const save = (key, v) => {
  try {
    localStorage.setItem(key, JSON.stringify(v));
  } catch {
    // storage blocked: the split still works, it is just not remembered
  }
};
const clamp = (p) => Math.min(MAX, Math.max(MIN, Math.round(p * 10) / 10));

/**
 * Turn the page into the split view.  `main` is the REPL column, `panes`
 * the element windows open into as panes (moved to the right side).
 * Returns `{ collapsed, collapse(on), restore(), pos }`.
 */
export function splitView(main, panes, { key = "reva:ui:split", title = "Graphics" } = {}) {
  const saved = load(key);
  const state = { pos: clamp(saved?.pos ?? HOME), collapsed: !!saved?.collapsed };
  const body = document.body;
  body.classList.add("split");

  const divider = document.createElement("div");
  divider.id = "divider";
  divider.tabIndex = 0;
  divider.setAttribute("role", "separator");
  divider.setAttribute("aria-orientation", "vertical");
  divider.setAttribute("aria-controls", "repl");
  divider.setAttribute("aria-label", "Resize the REPL and graphics panes");
  divider.setAttribute("aria-valuemin", String(MIN));
  divider.setAttribute("aria-valuemax", String(MAX));
  divider.title = "Drag to resize · double-click: 50/50 · Enter: collapse";

  const side = document.createElement("aside");
  side.id = "side";
  side.setAttribute("aria-label", title);
  const bar = document.createElement("div");
  bar.className = "bar";
  const name = document.createElement("span");
  name.textContent = title;
  const hide = button("⟩", `Collapse the ${title.toLowerCase()} pane`);
  bar.append(name, hide);
  const strip = button(`⟨ ${title}`, `Show the ${title.toLowerCase()} pane`);
  strip.className = "strip";
  side.append(bar, panes, strip);
  main.id ||= "repl";
  main.after(divider, side);

  const apply = (keep = true) => {
    body.style.setProperty("--split", `${state.pos}%`);
    body.classList.toggle("collapsed", state.collapsed);
    divider.setAttribute("aria-valuenow", String(state.collapsed ? 100 : state.pos));
    divider.setAttribute("aria-valuetext",
      state.collapsed ? `${title} collapsed` : `REPL ${Math.round(state.pos)}%`);
    if (keep) save(key, state);
  };
  const set = (pos) => { state.pos = clamp(pos); state.collapsed = false; apply(); };
  const collapse = (on) => {
    state.collapsed = on;
    apply();
    (on ? strip : divider).focus({ preventScroll: true });
  };

  divider.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    e.preventDefault();
    divider.setPointerCapture(e.pointerId);
    body.classList.add("dragging");
    const move = (ev) => {
      const w = body.clientWidth;
      if (w > 0) { state.pos = clamp((ev.clientX / w) * 100); state.collapsed = false; apply(false); }
    };
    const up = () => {
      divider.removeEventListener("pointermove", move);
      body.classList.remove("dragging");
      apply();
    };
    divider.addEventListener("pointermove", move);
    divider.addEventListener("pointerup", up, { once: true });
    divider.addEventListener("pointercancel", up, { once: true });
  });
  divider.addEventListener("dblclick", () => set(HOME));
  divider.addEventListener("keydown", (e) => {
    const step = e.shiftKey ? 10 : 2;
    const k = {
      ArrowLeft: () => set(state.pos - step),
      ArrowRight: () => set(state.pos + step),
      Home: () => set(MIN),
      End: () => set(MAX),
      Enter: () => collapse(!state.collapsed),
      " ": () => collapse(!state.collapsed),
    }[e.key];
    if (k) { e.preventDefault(); k(); }
  });
  hide.addEventListener("click", () => collapse(true));
  strip.addEventListener("click", () => collapse(false));
  apply(false);

  return {
    get collapsed() { return state.collapsed; },
    get pos() { return state.pos; },
    collapse: () => collapse(true),
    restore: () => collapse(false),
  };
}

function button(text, label) {
  const b = document.createElement("button");
  b.type = "button";
  b.textContent = text;
  b.title = label;
  b.setAttribute("aria-label", label);
  return b;
}
