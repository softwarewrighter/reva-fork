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
| `.deb` and `.rpm` | built; the `.deb` installed and passed `components/desktop/smoke.sh` |
| `reva.exe` (cross-built, `x86_64-pc-windows-gnu`) | run under Wine 9: REPL, files, CoSy's `demo paste-tidy.txt`, exit status |
| `reva-desktop.exe` (cross-built) | builds and links; does not start under Wine (no WebView2) |
| macOS `.app`/`.dmg`, Windows installers | not built: they need macOS and Windows |

## 1. What you can redistribute

| Artifact | Build | Ship with it |
|---|---|---|
| `reva` CLI | `cargo build --release` in `components/cli/` (per target, below) | `LICENSE`, `LICENSE-MIT`, `LICENSING.md` |
| web page | `scripts/build-web.sh` | the same three files |
| desktop installers | `components/desktop/build.sh --bundle` | added to the bundle automatically (see 4) |
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
  `scripts/license-check.sh` (with `cargo-deny` installed) must pass
  before a release.

## 2. The `reva` binary

`reva.f` and `lib/` are compiled into the binary, so one file is the whole
install. `needs` reads `lib/` from `$REVA_HOME` (or from the checkout the
binary was built in, if it is still there), and otherwise uses the built-in
copy. On another machine, the built-in copy is what it uses.

| Target | Command (in `components/cli/`) | Notes |
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
just web-zip            # or: scripts/make-web-zip.sh  → target/web/reva-web-VERSION.zip (~350 KB)
```

Unzip it anywhere and serve it with any static server (`python3 -m
http.server -d reva-web`). It needs no special headers (no
SharedArrayBuffer) and no particular MIME type for `.wasm` (it loads with
`fetch` plus `WebAssembly.instantiate`). The page must be served over HTTP:
a worker cannot start from `file://`. Saved images (`?image=app.img`) and
other files can sit next to `index.html`. `--package DIR` bundles a
package (CoSy's zip: `just cosy-zip` in ba-cosy-zip-fork). Options, the
split-view default and how to test under a subpath: [web.md](web.md).
The same tree as one local binary per OS: [serve.md](serve.md)
(`scripts/build-serve.sh mac|linux|windows`).

## 4. Desktop installers (Tauri 2)

```bash
cargo install tauri-cli --version "^2" --locked     # once, ~8 min
components/desktop/build.sh --bundle                           # plain Reva's installers in target/release/bundle/
just installer [PACKAGE_DIR]                         # a package's (below), in target/installer/
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

### Installers for a package (`just installer`)

```bash
just installer tests/installer-demo        # or scripts/make-installer.sh [--native] [PACKAGE_DIR]
```

This bundles reva-desktop with a package folder into installers in
`target/installer/`. It makes a universal `.dmg` on macOS (`--native` builds
only this Mac's architecture, which is quicker) and a `.deb` and `.rpm` on
Linux. With no folder it bundles plain Reva. Each file name carries the
package's version and reva-fork's, for example
`Reva-Demo_1.0.0_reva-2026.1121.0_universal.dmg`. The About dialog and the
footer show both versions too ("Reva Demo 1.0.0 on reva-fork ...").

**The installed app.** The package ships read-only inside the install, as
Tauri resources in `package/`:

- macOS: `Reva Demo.app/Contents/Resources/package`;
- Linux: `/usr/lib/<name>/package`.

Started without `--package`, the app finds the package there. The user's
files live in **Documents/&lt;userFolder&gt;**:

- macOS: `~/Documents`;
- Linux: the XDG documents folder, else `~/Documents`;
- Windows: the user's Documents folder.

The VM sees the package's files with the user's copies over them. Reads
look in the user's folder first, and every write goes there. The `seed`
files are copied into the folder only when they are missing, so an
upgrade never overwrites a user's file. Upgrading or uninstalling leaves
the folder alone. A file the user deletes that the package also ships
shows the package's copy again on the next start. `--user DIR` gives the
same layering to an uninstalled package (`--package DIR --user DIR2`), for
testing.

**Manifest fields the installer uses** (`reva-package.json`; the app's other
fields: `Manifest` in `reva-desktop-pkg`):

| field | use |
| --- | --- |
| `name` | required: the app's name (`productName`), the `.app`, the Linux binary (`reva-demo`: lower case, `-` for other characters), the default user folder |
| `about.version` | the installer's version (made semver: `2026.0105.0` becomes `2026.105.0`), else reva-fork's |
| `identifier` | the bundle id; default `com.softwarewrighter.reva.<binary>` |
| `icon` | a square PNG in the package: the bundles' icon set (`cargo tauri icon`); default Reva's |
| `files` | if present, ships only the manifest and its `start`, `files`, `seed`, `icon` and `image` files; if absent, the whole folder except dot files |
| `userFolder` | the folder under Documents (one plain name; default `name`) |
| `seed` | package files copied into the user's folder when missing (start data, such as a `.csy`) |

**Unsigned, first start** (the user, 2026-11-22: no paid signing).

- macOS: the app is ad-hoc signed, so Apple Silicon runs it, but it isn't
  notarized. Open it once, then go to System Settings › Privacy &
  Security and choose Open Anyway. Or run `xattr -dr
  com.apple.quarantine "/Applications/Reva Demo.app"`.
- Windows: SmartScreen warns. Choose More info, then Run anyway, or
  Unblock it in the file's Properties.
- Linux packages aren't signed: `sudo apt install ./NAME.deb` or `sudo
  dnf install ./NAME.rpm`.

The signing settings above still work if a certificate is given:
`APPLE_SIGNING_IDENTITY` replaces the ad-hoc identity. For Windows,
SignPath Foundation's free signing for open source could fill
`bundle.windows.signCommand` later.

**Checked** (step local-installer-tests). `tests/check-installed.sh APP
[PACKAGE_VERSION]` checks any installed package app, by hand or from
another project's release script (CoSy's `check-release.sh --installed`).
`tests/check-installed.ps1 -App ...` does the core of it in PowerShell, on
Windows without Git Bash. The user's folder goes under a throwaway
`REVA_DOCUMENTS`, which the app reads in place of Documents, so a check
never touches a user's real folder. It checks:

- the package is installed (and on Windows `sqlite3.dll` next to the app);
- macOS: the signature (ad-hoc) verifies;
- `--about` shows reva-fork's version and, for a package, its name and
  version, where it is installed and the user's folder (`package:`,
  `package dir:`, `user folder:`);
- the app starts and evaluates, and `version` shows reva-fork's commit;
- the user's folder is made under Documents, and on a fresh folder the
  seed files are copied from the package;
- the user's copy of a file is read over the package's;
- writes land in the user's folder, never in the install;
- `save` writes an image there, and the app restarts from it;
- turtle graphics draw in their own window (`[canvas: 768 lines]`).

`scripts/installer-lifecycle.sh` takes the demo package
(`tests/installer-demo`) through a user's life on the OS it runs on. CI
runs it on all three:

1. build the installer at 1.0.0 and at 1.0.1;
2. install 1.0.0 as a user does: macOS, the `.dmg` mounted and the app
   copied out; Linux, the `.deb` with apt; Windows, `setup.exe /S` (per
   user, no admin rights), with the build's `sqlite3.dll` off PATH;
3. run the checks, and take a screenshot with an IUP dialog and turtle
   drawing open;
4. add the user's own files, install 1.0.1 over it, run the checks again
   (on Windows the `.ps1` too): the new version runs, the user's files
   are kept;
5. uninstall: the app is gone, the user's folder is still there.

**Results** (CI run 46052033087, 2026-11-27, reva-fork 2026.1121.0):

| OS | install | checks (1.0.0, then 1.0.1 over it) | upgrade keeps the user's files | uninstall keeps the user's folder | screenshot |
|---|---|---|---|---|---|
| Linux (ubuntu-latest) | `.deb` with apt | all pass | yes | yes | black: Xvfb's capture misses what WebKitGTK draws (to look into) |
| macOS (macos-latest, arm64) | `.dmg` mounted, app copied | all pass, ad-hoc signature verifies | yes | yes | the app, the IUP dialog, the turtle drawing; footer "Reva Demo 1.0.0 on reva-fork 2026.1121.0" |
| Windows (windows-latest) | `setup.exe /S`, per user | all pass, the `.ps1` too, without the build's `sqlite3.dll` on PATH | yes | yes | the app, the IUP dialog, the turtle drawing |

Found and fixed on the way: an app started through a symlinked path on
macOS lost its package (Tauri refuses such a path; it now finds its
bundle by the real path), and `--about` did not name the package.

The installers and screenshots are the CI run's artifacts
(`installers-<OS>`). Not checked automatically in the installed app:
IUP and CD pixels (the browser page's pixel test, `cd-pixels.mjs`,
covers the same page code; the screenshots show the installed app), and
the first start of an unsigned download (Gatekeeper and SmartScreen
only act on files downloaded by a browser: a by-hand check, below).

**Windows** (`setup.exe`, NSIS, `installMode` currentUser, so no admin
rights are needed, with the WebView2 bootstrapper) is set in
`tauri.conf.json`; `just installer` makes it in Git Bash with Visual
Studio's build tools, and ships `sqlite3.dll` next to the app.

**By hand, before a release** (a download from a browser, on each OS):
open the installer as a user would, accept the warning once (above), start
the app from the Start menu, Launchpad or the desktop menu, and run
`tests/check-installed.sh` (or the `.ps1`) on it.

**CoSy's ask 26** (ba-cosy-zip-fork `docs/reva-fork-asks.md`), answered:

- `just installer` makes the macOS, Linux and Windows installers, and CI
  installs, upgrades, uninstalls and checks each.
- The installed app follows CoSy's choices: the user folder is
  Documents/CoSy (`"userFolder": "CoSy"`), reads look there first, writes
  go there, and only `CoSy.csy` is copied on first start (`"seed":
  ["CoSy.csy"]`). `CoSy.orig.csy` stays readable in the install.
- No AppImage: it would bundle WebKitGTK, which is LGPL.

## 5. Windows 10 and later

### Status

| Part | State |
|---|---|
| CI | since step local-ci (2026-11-23): `.github/workflows/ci.yml` runs the whole gate (`scripts/check.sh`) and the desktop smoke test on `windows-latest` (x64, MSVC, Git Bash) as on macOS and Linux, then builds the demo package's `setup.exe`, installs it silently and checks the installed app. Required since step local-windows-goldens (2026-11-25): a PR is ready when Windows is green too |
| `reva.exe` | builds, passes every component's tests, release-check and all 70 goldens on real Windows (CI), the same as on macOS and Linux. Step local-windows-goldens fixed the 9 that differed: drive paths (`D:/a/x.f`, `C:\a\x.f`) were taken as relative (gap 2), a shell command's status was not a wait status (`shell_code @ 256 /` gave 0 for `exit 3`), `--port-check` printed `.\x.f`, and two tests were Unix-only (the turtle test saved to `/tmp`; line editing ran at a Unix pty, now at a ConPTY through pywinpty) |
| browser page | the web tests, the CD pixel test among them, pass in Chrome on Windows (CI) |
| `reva-desktop.exe` | builds and passes its tests and release-check (`--about`) on Windows; a Windows (GUI) program, so no console window opens (gap 1 below, fixed); smoke test in CI: all 11 checks pass (the turtle window once hung: a synchronous command building a window deadlocks on Windows; `open_window` is `async` now) |
| SQLite | the official `sqlite3.dll` (3.54.1), linked, not compiled: `scripts/sqlite-dll.sh` ([sqlite.md](sqlite.md#how-it-works)); it ships next to the `.exe` |
| line ends | `.gitattributes` (`* -text`): files are checked out as stored, so Git for Windows does not add CR (it broke `VERSION` and the scripts) |
| scripts (`build.sh`, `smoke.sh`, gates, `make-subset.sh`) | bash or POSIX sh; they run in Git Bash (it includes perl) |

Known Windows gaps, to fix after a first run on real Windows:

1. **A console window opened with the desktop app** (fixed, step local-ci).
   It is a Windows (GUI) program now (`windows_subsystem = "windows"`), so
   none opens; started from cmd or PowerShell it attaches to that console,
   so `--about` and `--print-output` still print there
   (`reva-desktop/src/console.rs`), and redirected output (Git Bash,
   `smoke.sh`) works as before. Checked in CI through redirection; the
   console case wants a by-hand look on Windows.
2. **Absolute paths in Forth** (fixed, step local-windows-goldens). The
   VM's paths are `/`-rooted, and a host path `C:\dir` is `/C:/dir` to
   the VM. A program may also write `C:\dir\file` or `C:/dir/file`: on
   Windows a drive path is absolute and `\` separates like `/`
   (`reva-hostpath`'s `user_path`).
3. **End of input at the console is Ctrl-Z then Enter**, not Ctrl-D.

### How to build for Windows

| Way | Good for | How |
|---|---|---|
| **Cross-build on Linux** (checked) | quick checks; the CLI for release | `rustup target add x86_64-pc-windows-gnu`, `sudo apt install mingw-w64`, then `cargo build --release --target x86_64-pc-windows-gnu ` in `components/cli/`. Run it with `wine` (`sudo apt install wine64`): pipe input works, and there is no interactive console. The desktop app also builds with this target (`cargo build --release --target x86_64-pc-windows-gnu` in `components/desktop/`), but a GNU build loads `WebView2Loader.dll`, which must ship next to it. An NSIS installer can be cross-built with `cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc` (needs `cargo-xwin`, `nsis` and `lld`); this was not tried |
| **Native on Windows** | release installers, and the real test | Install [rustup](https://rustup.rs) with the MSVC toolchain and the Visual Studio Build Tools ("Desktop development with C++"), plus [Git for Windows](https://gitforwindows.org) for Git Bash. Then, in Git Bash: `scripts/sqlite-dll.sh` and set `SQLITE3_LIB_DIR` and `PATH` as it says, `rustup target add wasm32-unknown-unknown`, `cargo install tauri-cli --version "^2" --locked`, `just test`, and `components/desktop/build.sh --bundle`. The MSVC build links the WebView2 loader statically, so there is no extra DLL |
| **CI** (since step local-ci) | every push and PR | `.github/workflows/ci.yml`: the `windows-latest` job runs `scripts/sqlite-dll.sh x64`, then `scripts/check.sh` and `components/desktop/smoke.sh` in Git Bash |

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
   components/desktop/smoke.sh` (five "ok").
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
  `scripts/doc-check.sh docs/*.md`, the reg-rs goldens and
  `license-check.sh` (with `reg-rs`, `sed-rs` and `cargo-deny` from
  `cargo install`).
- **cli** (matrix of the targets in section 2): build, then tar or zip
  with the license files.
- **web**: `build-web.sh`, then the zip from section 3.
- **desktop** (ubuntu with `libwebkit2gtk-4.1-dev`, macos, windows):
  `components/desktop/build.sh --bundle`, then `components/desktop/smoke.sh` (under `xvfb-run` on
  Linux). Upload the installers.
- **on a tag**: attach everything to a GitHub release. Signing secrets
  (Apple and Authenticode) are added when certificates exist.

This would also give the repository the CI it lacks: every gate in the
README's Branches section, run on all three systems on every PR.
