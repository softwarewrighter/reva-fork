# Turtle graphics

Logo-style turtle graphics: Forth words move a turtle that draws lines on a
2D canvas in a **separate graphics window**. It works the same in the
[desktop app](desktop.md) (a second native window), in the browser (a popup
window, a pane below the REPL, or the [split view](web.md#the-split-view)
beside it), and on the CLI. The CLI has no window, so
there you save the drawing as SVG instead.

```text
needs turtle/fractals
fractals                 | lists the demos
5 koch-demo              | the graphics window opens and draws
" koch.svg" svg-save .   | the drawing as an SVG file (0 = ok)
```

## Running it locally

From a reva-fork checkout on `dev`:

```text
# desktop app (Linux, macOS): build once, then
desktop/build.sh --install
reva-desktop -e 'needs turtle/fractals' -e '5 koch-demo'

# browser
rust/scripts/build-web.sh
python3 -m http.server -d rust/reva-web/www 8000
#   open http://localhost:8000/   then type:  needs turtle/fractals 12 dragon-demo
#   http://localhost:8000/?canvas=pane keeps the drawing inside the page
#   http://localhost:8000/?canvas=split puts it beside the REPL (see web.md)

# CLI: no window; write SVG files
cd rust && cargo build --release
target/release/reva -e 'needs turtle/fractals 6 hilbert-demo " hilbert.svg" svg-save . bye'
```

The graphics window has a **Save PNG** button. After you close it, the REPL's
**Graphics** button brings it back, and the next drawing that starts with
`clear` reopens it.

## The vocabulary: `needs turtle/core`

The words live in the context `~turtle`, which `needs` adds to the search
order.

The world is 800 × 800 pixels with the origin in the centre and *y* pointing
up. Heading 0 is north, and `right` turns clockwise, as in Logo. Lengths are
pixels, angles are whole degrees, and colours are `$RRGGBB`. Every argument
is an integer.

| Word | Stack | Does |
|---|---|---|
| `forward` `fd` / `back` `bk` | `( n -- )` | move, drawing if the pen is down |
| `right` `rt` / `left` `lt` | `( deg -- )` | turn |
| `setxy` / `setheading` `seth` / `home` | `( x y -- )` / `( deg -- )` / `( -- )` | go to a point or heading; home = centre, north |
| `xy@` / `heading@` | `( -- x y )` / `( -- deg )` | where the turtle is (rounded) |
| `penup` `pu` / `pendown` `pd` | `( -- )` | stop / start drawing |
| `color` / `width` / `bg` | `( rgb -- )` / `( n -- )` / `( rgb -- )` | pen colour, pen width, background |
| `step` / `step!` / `scale` | `( -- )` / `( n -- )` / `( num den -- )` | move by the *step length*; set it; multiply it by num/den |
| `tpush` / `tpop` | `( -- )` | save / restore the whole turtle (position, heading, pen, colour, width, step) |
| `path` | `( a n -- )` | bulk: for each of the `n` cells at `a`, face that heading and take a step |
| `clear` `cs` / `clean` | `( -- )` | erase and reset the turtle / erase only |
| `show` / `hide` | `( -- )` | the turtle marker in the window |
| `segments` | `( -- n )` | lines in the drawing |
| `svg-save` | `( a n -- ior )` | write the drawing as SVG; 0 or -errno |

Colour constants: `black white red green blue gold brown purple orange`.

```forth
needs turtle/core
: square 4 0do 100 forward 90 right loop ;
clear square xy@ . . heading@ . segments .
```
```output
0 0 0 4
```

Loops (`0do ... loop`) only work inside a definition, as everywhere in Reva.

**Why a step length.** Fractals shrink their segments at every level of
recursion. With integer lengths, `100 3 /` loses a third of a pixel per
level, and the figure drifts. The turtle instead keeps its own step length
as a float: `1 3 scale` divides it exactly, and `tpush`/`tpop` restore it.

```forth
needs turtle/core
: tri 3 0do step 120 left loop ;
clear 90 setheading 100 step! 1 3 scale 1 3 scale tpush tri tpop
xy@ . . segments .
```
```output
0 0 3
```

`path` draws a whole curve in one call. This suits array languages, where
the headings come from a running sum of turns. It is the entry point the
CoSy demo uses.

```forth
needs turtle/core
create hs 0 , 90 , 180 , 270 ,
clear 50 step! hs 4 path xy@ . . segments .
```
```output
0 0 4
```

## The fractals: `needs turtle/fractals`

Each `-demo` word takes a depth, clears the canvas, places the turtle, and
sizes the step so the figure fills the window.

| Demo | Try | Max depth | Lines at max | Technique |
|---|---|---|---|---|
| `koch-demo` | `5 koch-demo` | 7 | 49,152 | snowflake: each edge becomes 4 edges ⅓ as long |
| `sierpinski-demo` | `6 sierpinski-demo` | 8 | 19,683 | three half-size triangles, `tpush`/`tpop` |
| `dragon-demo` | `12 dragon-demo` | 16 | 65,536 | paper folding; `( depth sign )` recursion, all 90° turns |
| `hilbert-demo` | `6 hilbert-demo` | 8 | 65,535 | space-filling curve, `( depth angle )` recursion |
| `tree-demo` | `10 tree-demo` | 12 | 4,095 | branches ⅔ as long, thinner and greener |
| `plant-demo` | `5 plant-demo` | 6 | 6,048 | an **L-system** (below) |

A larger depth is capped with a note, so a typo cannot ask for billions of
lines. Independently, the turtle stops drawing at 1,000,000 lines per
drawing and says so once; `clear` starts over. The words are also there
on their own (`koch`, `snowflake`, `sierpinski`, `dragon`, `hilbert`, `tree`,
`lsys`) for your own figures.

```forth
needs turtle/fractals
5 koch-demo segments .  12 dragon-demo segments .  99 hilbert-demo segments .
```
```output
3072 4096 (depth capped at 8 ) 65535
```

### L-systems

`lsys ( a n depth -- )` expands a string by its rules, recursively, without
building the expanded string. `F` and `G` draw a step, `f` moves one step
without drawing, `+` and `-` turn by `angle`, and `[` and `]` save and
restore the turtle. Other characters only get expanded. Define rules with
`rule ( c a n -- )`:

```forth
needs turtle/fractals
: koch-rules  rules-clear 60 angle !  'F " F+F--F+F" rule ;
: flake  clear koch-rules  penup -150 90 setxy 90 setheading pendown
  10 step!  " F--F--F" 2 lsys ;
flake segments . xy@ . .
```
```output
48 90 -150
```

`plant-demo` uses rules for `X` (which only expands) and `F`:
`X → F+[[X]-X]-F[-FX]+X`, `F → FF`, at 25°.

```forth
needs turtle/fractals
5 plant-demo segments .
```
```output
1488
```

## How it works

```text
 Forth words (lib/turtle/core)  ──(call)──▶  virtual library "turtle"  (reva-vm/src/turtle.rs)
                                                     │
                                       Canvas: turtle (f64) + display list (canvas.rs)
                                                     │ take_ops(), once per ~8 ms slice
              ┌──────────────────────────────────────┼──────────────────────────┐
     browser / desktop page                                                   CLI / any host
 worker ─ui{channel:"canvas", ops}─▶ page (ui.js) ─▶ canvas.html             svg-save via the VFS
 (Float32Array, transferred)          popup, pane, or 2nd Tauri window
```

* **Turtle math is on the host.** Position and heading are `f64` in Rust,
  so Forth needs no floats. The portable core has no x87 yet, and integer
  arguments lose nothing to rounding as a figure recurses. Multiples of 90°
  are exact, so right-angle curves land on whole pixels.
* **The display list is the drawing.** The VM keeps every line until `clear`.
  A window that opens (or reopens, or reloads) says "ready", and the page
  asks for the whole drawing again (`resend`). The page itself keeps
  nothing, and a late or closed window always catches up.
* **Batched, never per line.** After each time slice the worker takes the
  new ops: records of 8 `f32` (`CLEAR bg`, `LINE x1 y1 x2 y2 rgb width`,
  `TURTLE x y heading visible`). A batch holds at most 16,384 records
  (512 KiB) and is sent as one transferable `Float32Array`. The window
  queues batches and draws at most 60,000 lines per animation frame into a
  canvas sized to the window (times the device pixel ratio, 256-4096 px).
  When the window or pane is resized, it resizes the canvas and sends
  `redraw`, and the VM sends the whole drawing again. A
  65,536-line dragon takes about 150 ms of VM time and appears within a
  few frames.
* **One channel of the UI service.** Messages use the envelope
  `{ type: "ui", channel, ... }`, and windows open by name through
  `openWindow(host, name, { where })` in `ui.js`. The desktop's `host.js`
  supplies native windows (`open_window` command, `reva-ui-down`/`-up`
  events); the browser uses `window.open`, falling back to an in-page pane
  when popups are blocked. After `restart`, or a worker restarted by Stop,
  the window gets `reset` and clears.
* **Limits:** depth caps per demo, 1,000,000 lines per drawing, a turtle
  stack of 4,096 (`tpush`), pen width 0-100.

See also [DESIGN.md](../rust/DESIGN.md) for the VM and
[portability.md](portability.md) for virtual libraries.
