// reva-worker.js — the Reva VM in a Web Worker, so the page never waits
// for it.  Started by `start()` in reva.js; it can also be driven directly.
//
// Messages (all plain objects with a `type`):
//
//   page → worker   init   { wasm: URL or bytes, files: { path: bytes },
//                            image?: URL or bytes, local?: [service] }
//                                        local: host-call services this
//                                        worker answers itself ("sqlite":
//                                        sqlite.js) instead of the page
//                   input  { text }      a line (or any text) to read
//                   start  { text }      start input: read now and again
//                                        after each `restart`
//                   close  {}            end of input
//                   stop   {}            interrupt the running word
//                   ui     { channel, resend? }
//                                        a UI channel: "canvas" + resend
//                                        asks for the whole drawing again
//                   hostreply { reply }  the answer to a hostcall: { int },
//                                        { bytes } or { error }
//   worker → page   output { text }      text printed since the last one
//                   ui     { channel: "canvas", ops | reset }
//                                        drawing ops since the last slice
//                                        (Float32Array, transferred), or
//                                        reset after `restart`
//                   status { status, exitCode, boot?, build? }
//                                        "running", NEED_INPUT or HALTED;
//                                        after boot, the build facts
//                   files  { list }      [{ path, data: bytes | null }]
//                   hostcall { call }    the VM waits for this call's answer
//                                        ({ service, op, ints, bytes }; see
//                                        Reva.hostCall), sent back as
//                                        hostreply
//                   error  { message }
//
// The worker runs the VM in slices of about 8 ms and yields between them,
// so `input` and `stop` messages are seen within one slice.  While Forth
// waits for input the worker is idle: no polling, no CPU.

import { Reva, RUNNING, HALTED, HOST_CALL } from "./reva.js";

let reva = null;
let pumping = false;
// Services this worker answers itself (init's `local`): "sqlite" in the
// browser (sqlite.js); the desktop's page answers instead.
let local = new Set();
let sqlite = null;
async function answerLocally(call) {
  sqlite ??= new (await import("./sqlite.js")).SqliteHost();
  return sqlite.answer(call);
}
const post = (m, transfer) => globalThis.postMessage(m, transfer);
const postUi = (m) => post(m, m.ops ? [m.ops.buffer] : []);

async function pump() {
  if (pumping || !reva) return;
  pumping = true;
  post({ type: "status", status: RUNNING });
  let s;
  try {
    s = await reva.run((text) => post({ type: "output", text }), { onUi: postUi });
  } catch (e) {
    post({ type: "error", message: String(e?.message ?? e) });
    s = HALTED;
  }
  pumping = false;
  sendFiles();
  // A host call: the page answers (hostreply), and the VM runs on.
  if (s === HOST_CALL) {
    const call = reva.hostCall();
    if (!local.has(call.service)) return post({ type: "hostcall", call });
    reva.resume(await answerLocally(call));
    return pump();
  }
  post({ type: "status", status: s, exitCode: s === HALTED ? reva.exitCode : 0 });
}

function sendFiles() {
  const list = reva.changes();
  if (list.length) post({ type: "files", list }, list.flatMap((c) => (c.data ? [c.data.buffer] : [])));
}

// Messages are handled one at a time, in order, so input that arrives
// while `init` is still loading the module waits for it.
let queue = Promise.resolve();
globalThis.onmessage = ({ data: m }) => {
  queue = queue.then(() => handle(m));
};

async function handle(m) {
  switch (m.type) {
    case "init":
      local = new Set(m.local ?? []);
      try {
        reva = await Reva.load(m.wasm, { files: m.files ?? {}, image: m.image });
      } catch (e) {
        post({ type: "error", message: `could not start: ${e?.message ?? e}` });
        return;
      }
      reva.setClock();
      post({ type: "output", text: reva.takeOutput() });
      post({ type: "status", status: reva.status, exitCode: 0, boot: true, build: reva.build });
      break;
    case "input":
      reva.feed(m.text);
      pump();
      break;
    case "start":
      reva.feedStart(m.text);
      pump();
      break;
    case "close":
      reva.closeInput();
      pump();
      break;
    case "hostreply":
      reva.resume(m.reply);
      pump();
      break;
    case "stop":
      if (pumping) reva.interrupt();
      break;
    case "ui":
      reva?.ui(m);
      if (!pumping) reva?.flushUi(postUi);
      break;
  }
}
