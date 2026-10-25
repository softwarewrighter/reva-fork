// Run www/reva-worker.js in a Node worker thread, as a browser would run it
// in a Web Worker: Node's worker_threads speak through `parentPort` rather
// than the global `onmessage`/`postMessage`, so map one onto the other
// (holding messages that arrive before the worker script has loaded).
import { parentPort } from "node:worker_threads";
globalThis.postMessage = (m, transfer) => parentPort.postMessage(m, transfer);
const early = [];
parentPort.on("message", (data) => (globalThis.onmessage ? globalThis.onmessage({ data }) : early.push(data)));
await import("../www/reva-worker.js");
for (const data of early) globalThis.onmessage({ data });
