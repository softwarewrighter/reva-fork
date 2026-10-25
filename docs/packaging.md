# Packaging and Windows

How to hand Reva to someone else: the `reva` binary, the static web page,
and the desktop app's installers, with the license notices each must carry.
Then Windows 10 and later: how to build for it, and a test plan. The
repository has no CI, so everything here runs by hand; a proposed workflow
is at the end.

What was checked, and where (2026-10-25, Linux x86-64 cloud machine):

| Thing | Checked |
|---|---|
| CLI, glibc and static musl builds | built and run |
| static web zip | built, unzipped elsewhere, served, run in Chromium |
| `.deb` and `.rpm` | built; the `.deb` installed and passed `desktop/smoke.sh` |
| `reva.exe` (cross-built, `x86_64-pc-windows-gnu`) | run under Wine 9: REPL, files, CoSy's `demo paste-tidy.txt`, exit status |
| `reva-desktop.exe` (cross-built) | builds and links; does not start under Wine (no WebView2) |
| macOS `.app`/`.dmg`, Windows installers | not built: they need macOS and Windows |

## 1. What you can redistribute

| Artifact | Build | Ship with it |
|---|---|---|
| `reva` CLI | `cargo build --release` in `rust/` (per target, below) | `LICENSE`, `LICENSE-MIT`, `LICENSING.md` |
| web page | `rust/scripts/build-web.sh` | the same three files |
| desktop installers | `desktop/build.sh --bundle` | added to the bundle automatically (see 4) |
| a package (for example CoSy) | the package's own build | the package's own terms |

### License notices

- **`LICENSE`** must always ship: Reva is public domain with a fall-back
  grant, and the FirmWorks assembler inside `reva.f` is under an MIT-style
  notice that asks to be kept with copies.
- **`LICENSE-MIT`** covers the portable core, the web page and the desktop
  app. **`LICENSING.md`** maps each part to its terms.
- **The CLI and the wasm module have no crate dependencies**, so those
  three files are all they need.
- **The desktop app** links about 320 crates. Each permissive license (MIT,
  Apache-2.0, BSD...) asks that its copyright notice ship with binaries,
  and the five MPL-2.0 Servo CSS crates ask that recipients be told where
  their source is. [licenses.md](licenses.md) names those crates and where
  their source is, and it ships in the bundle. Before a public release,
  also generate a full third-party notice file (proposed:
  [`cargo-about`](https://github.com/EmbarkStudios/cargo-about), MIT/Apache)
  and add it to `bundle.resources`. This step is not set up yet.
- **Nothing copyleft ships.** On Linux the app uses the system's WebKitGTK
  (LGPL) and never carries it. On Windows it uses the system's WebView2.
  `rust/scripts/license-check.sh` (with `cargo-deny` installed) must pass
  before a release.

## 2. The `reva` binary

`reva.f` and `lib/` are compiled into the binary, so one file is the whole
install. `needs` reads `lib/` from `$REVA_HOME` (or from the checkout the
binary was built in, if it is still there), and otherwise uses the built-in
copy. On another machine, the built-in copy is what it uses.

| Target | Command (in `rust/`) | Notes |
|---|---|---|
| Linux x86-64, any distribution | `rustup target add x86_64-unknown-linux-musl` then `cargo build --release --target x86_64-unknown-linux-musl -p reva-cli` | fully static. **Prefer this for redistribution**: a plain build links the build machine's glibc (2.39 on Ubuntu 24.04) and will not start on older systems |
| Linux ARM64 | `aarch64-unknown-linux-musl` | build on an ARM64 machine, or install a cross linker |
| macOS (Apple Silicon and Intel) | on a Mac: build `aarch64-apple-darwin` and `x86_64-apple-darwin`, then `lipo -create -output reva <the two>` | one universal file |
| Windows x86-64 | `x86_64-pc-windows-msvc` on Windows, or `x86_64-pc-windows-gnu` from Linux (section 5) | `reva.exe` uses only system DLLs |
| Windows ARM64 | `aarch64-pc-windows-msvc` on Windows | x64 builds also run under emulation |

Ship it as a `.tar.gz` (Unix) or `.zip` (Windows) with the three license
files. **Signing:** on macOS, a downloaded unsigned binary is quarantined.
Sign it with `codesign --options runtime --timestamp -s "Developer ID
Application: ..." reva`, then notarize the zip with `xcrun notarytool submit
reva.zip --keychain-profile <profile> --wait`. A bare binary cannot be
stapled, but Gatekeeper checks notarization online. Unsigned, users can run
`xattr -d com.apple.quarantine reva`. On Windows, an unsigned `.exe` gets a
SmartScreen warning until it has a reputation. Authenticode signing (with
`signtool` on Windows, or `osslsigncode` on Linux) needs a code-signing
certificate.

## 3. The static web page

```bash
just web-zip            # or: rust/scripts/make-web-zip.sh  → rust/target/web/reva-web-VERSION.zip (~350 KB)
```

Unzip it anywhere and serve it with any static server (`python3 -m
http.server -d reva-web`). It needs no special headers (no
SharedArrayBuffer) and no particular MIME type for `.wasm` (it loads with
`fetch` plus `WebAssembly.instantiate`). The page must be served over HTTP:
a worker cannot start from `file://`. Saved images (`?image=app.img`) and
other files can sit next to `index.html`. `--package DIR` bundles a
package (CoSy's zip: `just cosy-zip` in ba-cosy-zip-fork). Options, the
split-view default and how to test under a subpath: [web.md](web.md).

## 4. Desktop installers (Tauri 2)

```bash
cargo install tauri-cli --version "^2" --locked     # once, ~8 min
desktop/build.sh --bundle                           # installers in desktop/target/release/bundle/
```

`tauri.conf.json` lists `deb`, `rpm`, `app`, `dmg`, `nsis` and `msi`, and
the bundler makes the ones that fit the machine it runs on. Every bundle
carries `LICENSE`, `LICENSE-MIT`, `LICENSING.md` and `licenses.md` as
resources (on Linux, in `/usr/lib/Reva/`).

**Linux: `.deb` and `.rpm`** (2 MB each). The `.deb` depends on
`libwebkit2gtk-4.1-0` and `libgtk-3-0` from the distribution. Install it with
`sudo apt install ./Reva_0.1.0_amd64.deb`, then run `reva-desktop` (it is
also in the desktop menu as Reva). **No AppImage:** an AppImage carries its
own WebKitGTK, which is LGPL, and the project bundles no copyleft code
([licenses.md](licenses.md)). Build on the oldest distribution you want to
support, because the binary links that distribution's glibc.

**macOS: `.app` and `.dmg`** (build on a Mac). Without signing, the app
opens only after right-click › Open, or after `xattr -dr
com.apple.quarantine Reva.app`. To sign and notarize, Tauri reads these
variables during `cargo tauri build`:

- signing: `APPLE_SIGNING_IDENTITY` (a "Developer ID Application"
  certificate in the keychain; in CI, `APPLE_CERTIFICATE` and
  `APPLE_CERTIFICATE_PASSWORD` import it);
- notarization: `APPLE_API_ISSUER`, `APPLE_API_KEY` and
  `APPLE_API_KEY_PATH` (an App Store Connect API key), or `APPLE_ID`,
  `APPLE_PASSWORD` (an app-specific password) and `APPLE_TEAM_ID`.

With those set, Tauri signs with the hardened runtime, notarizes and
staples. For a universal app, run `cargo tauri build --target
universal-apple-darwin` (after adding both Apple targets).

**Windows: NSIS `-setup.exe` and `.msi`** (build on Windows; see 5). The
app needs the WebView2 runtime. Windows 11 includes it, and most Windows 10
machines have it through Edge updates, but offline and LTSC installs may
not. Tauri's `bundle.windows.webviewInstallMode` chooses what the
installer does about it. The default is `downloadBootstrapper` (fetch it
if missing); `embedBootstrapper` adds about 2 MB; `offlineInstaller`
embeds the full runtime (about 130 MB) for machines with no network.
Installers are signed with `bundle.windows.certificateThumbprint` (or a
`signCommand`), the same Authenticode certificate as in section 2. The MSI
needs WiX, which the bundler downloads, and it only builds on Windows. NSIS
can also be cross-built (section 5).

## 5. Windows 10 and later

### Status

| Part | State |
|---|---|
| `reva.exe` | works under Wine since this change: file paths were broken on Windows (`include t.f` found nothing) and are fixed now (`reva-cli/src/fs.rs`, unit-tested on every host). Not yet run on real Windows |
| browser page | the same wasm on any browser; should work in Edge and Chrome on Windows; not yet checked there |
| `reva-desktop.exe` | builds (this change adds the `icon.ico` that a Windows build needs, and the `nsis`/`msi` targets); not run: it needs WebView2, which Wine lacks |
| scripts (`build.sh`, `smoke.sh`, gates, `make-subset.sh`) | bash or POSIX sh; run them in Git Bash (it includes perl) |

Known Windows gaps, to fix after a first run on real Windows:

1. **A console window opens with the desktop app.** It is built as a
   console program so that `--print-output` and `smoke.sh` work. The usual
   fix is `windows_subsystem = "windows"` plus attaching to the parent's
   console when `--print-output` is given.
2. **Absolute paths in Forth are written `/C:/dir/file`.** The VM's paths
   are `/`-rooted, and a host path `C:\dir` is `/C:/dir` to the VM.
   Relative paths and `/` separators work as usual. `C:\dir\file` written
   in Forth is treated as a relative name (checked under Wine).
3. **End of input at the console is Ctrl-Z then Enter**, not Ctrl-D.

### How to build for Windows

| Way | Good for | How |
|---|---|---|
| **Cross-build on Linux** (checked) | quick checks; the CLI for release | `rustup target add x86_64-pc-windows-gnu`, `sudo apt install mingw-w64`, then `cargo build --release --target x86_64-pc-windows-gnu -p reva-cli` in `rust/`. Run it with `wine` (`sudo apt install wine64`): pipe input works, and there is no interactive console. The desktop app also builds with this target (`cargo build --release --target x86_64-pc-windows-gnu` in `desktop/`), but a GNU build loads `WebView2Loader.dll`, which must ship next to it. An NSIS installer can be cross-built with `cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc` (needs `cargo-xwin`, `nsis` and `lld`); this was not tried |
| **Native on Windows** | release installers, and the real test | Install [rustup](https://rustup.rs) with the MSVC toolchain and the Visual Studio Build Tools ("Desktop development with C++"), plus [Git for Windows](https://gitforwindows.org) for Git Bash. Then, in Git Bash: `rustup target add wasm32-unknown-unknown`, `cargo install tauri-cli --version "^2" --locked`, `cd rust && cargo build --release && cargo test --release`, and `desktop/build.sh --bundle`. The MSVC build links the WebView2 loader statically, so there is no extra DLL |
| **CI** (proposed, not added) | repeatable release builds | a `windows-latest` GitHub Actions job running the same commands; see 6 |

A Mac can run Windows 11 ARM in a VM (UTM or Parallels). There, build
`aarch64-pc-windows-msvc` natively, or run the x64 builds under
emulation. Windows 10 x64 needs an x86-64 machine or a cloud VM.

### Test plan

Machines: Windows 10 22H2 x64 and Windows 11 x64 (and Windows 11 ARM64 if
available), each a clean VM without developer tools, for the installers.
Record each result as pass or fail, with the output, in the PR that makes
Windows supported.

CLI (`reva.exe`, in Windows Terminal and in the old console host, then in
PowerShell):

1. `reva.exe -V` prints the version report; `reva.exe` shows the banner and
   `ok>`.
2. The [demo](demo.md) steps 1 and 2: definitions, `needs string/soundex`,
   the error and fault lines, `restart`, `version`.
3. Files: `" t.txt" creat ...`, then `" t.txt" slurp type`; a file in a
   subfolder (`" sub/t.txt"`); an absolute
   `/C:/Users/<you>/t.txt`; a path with spaces; a file name differing only
   in case (`T.TXT`: the same file on NTFS).
4. Ctrl-C during `: forever repeat again ; forever` (expect the process to
   end, since the CLI has no interrupt handler yet; write down what
   happens), and Ctrl-Z Enter to end input.
5. Exit status: `echo 3 ~sys.(bye) | reva.exe` then `echo %ERRORLEVEL%`
   (cmd) or `$LASTEXITCODE` (PowerShell) gives 3.
6. Piping: `echo 2 3 + . | reva.exe` in cmd and in PowerShell (PowerShell
   may add a byte order mark or change line endings).
7. `save app.img` then `reva.exe --image app.img`; the same image file
   copied from Linux also starts (images are host-independent).
8. `--allow-shell` with `needs os/shell " dir" shell`.

Browser: serve the web zip from any machine and open it in Edge and
Chrome. Run [demo](demo.md) step 3, including reload persistence and Stop.

Desktop (`-setup.exe` and `.msi` on the clean VMs):

1. Install with WebView2 present; then on a Windows 10 VM without WebView2
   (to check `webviewInstallMode`), and with no network.
2. Start from the Start menu: the window opens and the REPL works.
   Files go to `%USERPROFILE%\Reva`. Note whether a console window
   appears (gap 1).
3. `reva-desktop.exe --package C:\path\to\pkg`: the start files run;
   a file Forth writes into a subfolder lands in the package folder; a
   path with spaces.
4. In Git Bash: `REVA_DESKTOP=/c/Program\ Files/Reva/reva-desktop.exe
   desktop/smoke.sh` (five "ok").
5. CoSy: `reva-fork-subset/make-subset.sh` in Git Bash, then
   `reva-desktop.exe --package <...>\build`, and `demo paste-tidy.txt`
   (see ba-cosy-zip-fork's `reva-fork-subset/DEMO.md`).
6. Uninstall: the app and its Start menu entry go away, and
   `%USERPROFILE%\Reva` is kept.
7. With signing set up: no SmartScreen "unknown publisher" warning.

## 6. Proposed CI (not added)

One GitHub Actions workflow, `release.yml`, on pull requests to `dev` and
`main` and on tags `v*`:

- **test** (ubuntu, macos, windows): `cargo test --release --workspace`,
  `scripts/doc-check.sh ../docs/*.md`, the reg-rs goldens and
  `license-check.sh` (with `reg-rs`, `sed-rs` and `cargo-deny` from
  `cargo install`).
- **cli** (matrix of the targets in section 2): build, then tar or zip
  with the license files.
- **web**: `build-web.sh`, then the zip from section 3.
- **desktop** (ubuntu with `libwebkit2gtk-4.1-dev`, macos, windows):
  `desktop/build.sh --bundle`, then `desktop/smoke.sh` (under `xvfb-run` on
  Linux). Upload the installers.
- **on a tag**: attach everything to a GitHub release. Signing secrets
  (Apple and Authenticode) are added when certificates exist.

This would also give the repository the CI it lacks: every gate in the
README's Branches section, run on all three systems on every PR.
