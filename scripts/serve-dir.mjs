// serve-dir.mjs: serve a directory over HTTP on 127.0.0.1, for `just web-dev`
// (the browser REPL from the source tree). No caching, so a rebuild shows at
// the next reload.
//
//   node scripts/serve-dir.mjs DIR [PORT]      (PORT: 8000)
import http from "node:http";
import fs from "node:fs";
import path from "node:path";

const [dir = ".", port = "8000"] = process.argv.slice(2);
const root = path.resolve(dir);
const TYPES = {
  ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript",
  ".wasm": "application/wasm", ".css": "text/css", ".json": "application/json",
  ".png": "image/png", ".ico": "image/x-icon", ".svg": "image/svg+xml", ".txt": "text/plain",
};

http.createServer((req, res) => {
  const url = decodeURIComponent(new URL(req.url, "http://x").pathname);
  let file = path.join(root, url);
  if (!file.startsWith(root)) { res.writeHead(403); res.end(); return; }
  if (url.endsWith("/")) file = path.join(file, "index.html");
  fs.readFile(file, (err, data) => {
    if (err) { res.writeHead(404); res.end("not found"); return; }
    res.writeHead(200, {
      "Content-Type": TYPES[path.extname(file)] ?? "application/octet-stream",
      "Cache-Control": "no-store",
    });
    res.end(data);
  });
}).listen(Number(port), "127.0.0.1", () => console.log(`serving ${root} at http://localhost:${port}/`));
