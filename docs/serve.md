# reva-serve: the browser REPL as one local binary

`reva-serve` is one file per OS that holds the static site (the same tree
as the [web zip](web.md)) and a tiny web server. Run it, and the site opens
in your browser from `http://127.0.0.1:8421/`. Ctrl-C stops it. A package
can be built in, and CoSy's build is `cosy-serve` (ba-cosy-zip-fork).

```text
just serve                       # build for this machine, run it
scripts/build-serve.sh      # the same, without just: → target/serve/reva-serve-VERSION-TARGET
reva-serve-VERSION-TARGET        # serves, opens the browser; Ctrl-C to stop
```

## Running it

```text
usage: reva-serve [options]
  --port N     use port N instead of 8421
  --no-open    do not open the browser
  --list       list the embedded files and exit
  -V, --version
```

- **Fixed port, on purpose.** The page keeps your files in the browser's
  localStorage, and localStorage belongs to the address, port included. A
  fixed port means your files are there the next time you run it.
- **Already running?** If the port is held by a copy of the same app (any
  version), the second launch just opens the browser on it and exits.
- **Port taken by something else?** It takes the next free port (up to 20
  more) and says so: files saved at the usual port are not visible there.
- **Browser:** `open` on macOS, `xdg-open` on Linux, `explorer` on Windows;
  `$BROWSER` overrides. If none can be started, the URL is printed.
- **Ctrl-C** ends the process. There is nothing to save on the way out,
  because the browser holds your files.

## Building release binaries

`scripts/build-serve.sh [options] [TARGET ...]` (POSIX sh) builds the
site with `make-web-zip.sh` (or takes `--site DIR`), then compiles
`reva-serve` with it embedded, one binary per target, each with a `.sha256`:

| TARGET | Binary | Where to build |
|---|---|---|
| `mac` | universal (arm64 + x86_64, joined with `lipo`) | macOS (Xcode command line tools) |
| `linux` | static musl binary for this CPU (x86_64 or aarch64) | Linux, or any host with the musl target |
| `windows` | `x86_64-pc-windows-gnu` `.exe` | Windows, or Linux with `mingw-w64` (`apt install gcc-mingw-w64-x86-64`) |
| a triple | that target | wherever rustup and a linker for it are |

The default target is `mac` on macOS and `linux` on Linux. Targets are added
with `rustup target add` as needed. Options: `--package DIR` and `--name
NAME` (passed to make-web-zip.sh; the binary is `NAME-serve`), `--port N`
(the built-in port), `--site DIR`, `--out DIR`, `--no-build`.

```text
# on a Mac: the universal binary
scripts/build-serve.sh mac
# on Linux: static Linux binary plus a Windows .exe
scripts/build-serve.sh linux windows
```

The build inputs reach `build.rs` as environment variables:
`REVA_SERVE_SITE` (the directory to embed; default `components/web/crates/reva-web/www`, so
a plain `cargo build -p reva-serve` serves the source tree),
`REVA_SERVE_NAME` and `REVA_SERVE_PORT`.

## Design

```text
 build time                                   run time
 make-web-zip.sh ─▶ site tree ─▶ build.rs ─▶  reva-serve ──bind 127.0.0.1:PORT──▶ browser
 (index.html, js, wasm, pkg/)   include_bytes!   │  └─ taken? probe /.reva-serve: ours → open it
                                                 │                               other → PORT+1..+20
                                                 └─ GET/HEAD only, Host must be 127.0.0.1/localhost:PORT
```

* **No dependencies.** The server is std only (`TcpListener`, one thread
  per connection, at most 32 at a time, `Connection: close`). A page loads
  about a dozen files, once. So the binary stays small (about 1.7 MB with
  the 1 MB wasm, 3 MB with CoSy), cross-compiles with nothing but
  `rustup target add`, and adds no crates to the license review
  (`scripts/license-check.sh`).
* **One tree, two deliveries.** The binary embeds exactly the tree the
  uploadable zip holds, so what you test locally is what you upload.
* **Loopback only, read only.** It binds `127.0.0.1`, never a public
  interface. It serves only `GET` and `HEAD` of embedded files, and refuses
  a `Host` that is not its own loopback address (421). That stops a page
  from another site from reading it through DNS rebinding. Responses carry
  `Cache-Control: no-cache`, `X-Content-Type-Options: nosniff` and
  `Referrer-Policy: no-referrer`.
* **Correct media types.** `.wasm` is served as `application/wasm` and Forth
  sources as `text/plain; charset=utf-8`.
* **Recognising itself.** `GET /.reva-serve` answers `reva-serve NAME
  VERSION`, so a second launch can tell its own server from another
  program on the port.
* **Limits:** request heads up to 16 KiB, a 10 s timeout per connection.
  Everything is held in memory, since it is the binary itself.
* **Not done:** HTTPS (not needed on loopback), keep-alive, compression,
  and macOS code signing. An unsigned binary downloaded from the web needs
  `xattr -d com.apple.quarantine FILE` or right-click › Open the first time.
  One built locally does not.
