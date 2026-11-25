// The pixels CD draws in the page: examples/iup/cd-test.f runs in the real
// page (www/, as built) in headless Chrome, and the IUP canvas's pixels are
// read back with getImageData and checked at known points: the background,
// the dotted line, the diamond mark, the filled box and the 8x8 bitmap
// scaled to 32x32 (the DRAW list these come from is the golden
// iup-cd-test).  The desktop app shows the same page, so this checks its
// drawing too.  Prints a transcript and exits non-zero on any mismatch;
// exit 2 if no Chrome is found (CHROME=path to name one).
//
// Chrome runs headless (no window, no focus taken), with a scratch profile,
// driven over the DevTools protocol with Node's own WebSocket: no npm
// packages.
//
//   node components/web/crates/reva-web/test/cd-pixels.mjs
import { spawn, execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import http from "node:http";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const www = path.join(here, "..", "www");
const example = path.join(here, "../../../../..", "examples/iup/cd-test.f");

/** Chrome: $CHROME, the Mac's, Windows' usual places, or one on PATH. */
function chrome() {
  const mac = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
  const win = ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"]
    .map((v) => process.env[v] && path.join(process.env[v], "Google/Chrome/Application/chrome.exe"));
  const found = [process.env.CHROME, mac, ...win].find((p) => p && existsSync(p));
  if (found) return found;
  const which = process.platform === "win32" ? ["where", []] : ["sh", ["-c"]];
  for (const name of ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser", "chrome"]) {
    try {
      const arg = which[0] === "sh" ? `command -v ${name}` : name;
      return execFileSync(which[0], [...which[1], arg], { encoding: "utf8" }).split(/\r?\n/)[0].trim();
    } catch { /* next */ }
  }
  return null;
}

/** www/ over HTTP on a free port of 127.0.0.1. */
function serve() {
  const TYPES = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm",
    ".css": "text/css", ".json": "application/json", ".png": "image/png", ".ico": "image/x-icon" };
  const server = http.createServer((req, res) => {
    const url = decodeURIComponent(new URL(req.url, "http://x").pathname);
    const file = path.join(www, url.endsWith("/") ? url + "index.html" : url);
    if (!file.startsWith(www)) { res.writeHead(403); res.end(); return; }
    try {
      const data = readFileSync(file);
      res.writeHead(200, { "Content-Type": TYPES[path.extname(file)] ?? "application/octet-stream" });
      res.end(data);
    } catch { res.writeHead(404); res.end(); }
  });
  return new Promise((ok) => server.listen(0, "127.0.0.1", () => ok(server)));
}

/** Headless Chrome, and its DevTools WebSocket URL. */
function launch(bin, profile) {
  const proc = spawn(bin, [
    "--headless=new", "--remote-debugging-port=0", `--user-data-dir=${profile}`,
    "--no-first-run", "--no-default-browser-check", "--window-size=1024,768",
    "--force-device-scale-factor=1", "about:blank",
  ], { stdio: ["ignore", "ignore", "pipe"] });
  return new Promise((ok, fail) => {
    let err = "";
    proc.stderr.on("data", (d) => {
      err += d;
      const m = err.match(/DevTools listening on (ws:\/\/\S+)/);
      if (m) ok({ proc, ws: m[1] });
    });
    proc.on("exit", (code) => fail(new Error(`chrome exited ${code}: ${err}`)));
  });
}

/** A DevTools connection: send(method, params, session) → result. */
async function devtools(url) {
  const ws = new WebSocket(url);
  await new Promise((ok, fail) => { ws.onopen = ok; ws.onerror = fail; });
  let id = 0;
  const waiting = new Map();
  ws.onmessage = (e) => {
    const m = JSON.parse(e.data);
    const w = waiting.get(m.id);
    if (!w) return;
    waiting.delete(m.id);
    m.error ? w.fail(new Error(m.error.message)) : w.ok(m.result);
  };
  const send = (method, params = {}, sessionId) => new Promise((ok, fail) => {
    waiting.set(++id, { ok, fail });
    ws.send(JSON.stringify({ id, method, params, sessionId }));
  });
  return { send, close: () => ws.close() };
}

// In the page: make the IUP canvas 400x200 CSS pixels (as a window resize
// would; the page reports the size, the VM redraws), then the colours at CD
// points (x, y up from the bottom left, as CD draws) and along the dotted
// line, or null until it is painted at that size.
const SAMPLE = (points) => `(() => {
  const c = document.querySelector(".tui-canvas");
  if (!c) return null;
  if (!document.getElementById("cd-pixels")) {
    const s = document.createElement("style");
    s.id = "cd-pixels";
    s.textContent = ".tui-canvas { width: 400px !important; height: 200px !important }";
    document.head.append(s);
    return "resize";
  }
  if (c.width !== 400 || c.height !== 200) return null;
  const ctx = c.getContext("2d");
  const at = ([x, y]) => {
    const d = ctx.getImageData(x, c.height - 1 - y, 1, 1).data;
    return [d[0], d[1], d[2]].map((v) => v.toString(16).padStart(2, "0")).join("");
  };
  const all = ${JSON.stringify(points)}.map(at);
  if (at([398, 198]) === "000000") return null;
  const line = [];
  for (let i = 3; i <= 27; i++) line.push(at([i, i]));
  return JSON.stringify({ corner: at([398, 198]), all, png: c.toDataURL("image/png"),
    // a 1-pixel diagonal is anti-aliased: a dot is a pixel clearly bluer than grey
    dots: line.filter((p) => parseInt(p.slice(4), 16) - parseInt(p.slice(0, 2), 16) > 40).length,
    gaps: line.filter((p) => p === "c0c0c0").length });
})()`;

// What cd-test.f draws (CD coordinates) and the colour expected there.
const EXPECT = [
  ["diamond mark, centre", [40, 10], "0000ff"],
  ["filled box 60..90 x 0..20, inside", [75, 10], "0000ff"],
  ["filled box, just right of it", [93, 10], "c0c0c0"],
  ["bitmap at 70,70: a 1 cell (yellow)", [71, 71], "ffff00"],
  ["bitmap: a 0 cell (blue)", [76, 76], "0000ff"],
  ["bitmap: its middle row (yellow)", [85, 85], "ffff00"],
  ["bitmap: just past it", [104, 104], "c0c0c0"],
];

/** Ask `test` every 0.2 s until it is true (20 s at most). */
async function waitFor(test) {
  for (let t = 0; t < 100; t++) {
    if (await test()) return;
    await new Promise((ok) => setTimeout(ok, 200));
  }
  throw new Error("cd-pixels: the page did not load");
}

let failed = 0;
const check = (what, got, want) => {
  const ok = got === want;
  if (!ok) failed++;
  console.log(`${ok ? "ok  " : "FAIL"} ${what}: ${got}${ok ? "" : ` (want ${want})`}`);
};

const bin = chrome();
if (!bin) {
  console.error("cd-pixels: no Chrome found (set CHROME to its path)");
  process.exit(2);
}
const server = await serve();
const profile = mkdtempSync(path.join(tmpdir(), "reva-cd-pixels-"));
const { proc, ws } = await launch(bin, profile);
try {
  const dt = await devtools(ws);
  // The example goes into the page's files (as an upload would put it), and
  // the page is opened again to include it, as a user would run it.
  const page = `http://127.0.0.1:${server.address().port}/index.html`;
  const { targetId } = await dt.send("Target.createTarget", { url: page });
  const { sessionId } = await dt.send("Target.attachToTarget", { targetId, flatten: true });
  const put = `(async () => {
    const { openIdbStorage } = await import("./store.js");
    const { storage } = await openIdbStorage();
    storage.setItem("reva:fs:/cd-test.f", ${JSON.stringify(readFileSync(example, "latin1"))});
    await storage.flush();
    return "ok";
  })()`;
  await waitFor(() => dt.send("Runtime.evaluate",
    { expression: put, awaitPromise: true, returnByValue: true }, sessionId)
    .then((r) => r.result?.value === "ok", () => false));
  await dt.send("Page.navigate", { url: `${page}?include=/cd-test.f` }, sessionId);
  const expr = SAMPLE(EXPECT.map((e) => e[1]));
  // Painted at 400x200 and the same in two samples 0.5 s apart (the redraw
  // the new size asks for has landed).
  let got = null;
  let last = null;
  for (let t = 0; t < 60 && !got; t++) {
    const r = await dt.send("Runtime.evaluate", { expression: expr, returnByValue: true }, sessionId);
    const v = r.result?.value ?? null;
    if (v === "resize") {
      // a new viewport: the TUI lays out again and paints the canvas at its new size
      await dt.send("Emulation.setDeviceMetricsOverride",
        { width: 1100, height: 800, deviceScaleFactor: 1, mobile: false }, sessionId);
    }
    if (v && v !== "resize" && v === last) got = JSON.parse(v);
    last = v;
    if (!got) await new Promise((ok) => setTimeout(ok, 500));
  }
  if (!got) {
    console.log("FAIL the IUP canvas was not painted within 30 s");
    failed++;
    const page = `(() => { const c = document.querySelector(".tui-canvas");
      return JSON.stringify({ canvas: c && [c.width, c.height, c.clientWidth, c.clientHeight],
        text: document.body.innerText.trim().slice(-800) }); })()`;
    const r = await dt.send("Runtime.evaluate", { expression: page, returnByValue: true }, sessionId);
    console.log(r.result?.value);
  } else {
    if (process.env.CD_PIXELS_PNG) {
      // the canvas as painted, to look at
      writeFileSync(process.env.CD_PIXELS_PNG, Buffer.from(got.png.split(",")[1], "base64"));
    }
    check("background, top right corner", got.corner, "c0c0c0");
    check("dotted line 0,0-30,30: blue dots", got.dots > 4, true);
    check("dotted line: background between them", got.gaps > 4, true);
    EXPECT.forEach(([what, , want], i) => check(what, got.all[i], want));
  }
  dt.close();
} finally {
  const gone = new Promise((ok) => proc.on("exit", ok));
  proc.kill();
  await gone;
  server.close();
  // Chrome's helpers may still be writing to the profile after it exits:
  // retry, and a leftover scratch folder is not a failure of the test.
  try {
    rmSync(profile, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
  } catch (e) {
    console.error(`cd-pixels: left ${profile} (${e.code})`);
  }
}
process.exit(failed ? 1 : 0);
