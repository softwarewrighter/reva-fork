# Transcript: step 080-local-installer-tests

- session: `8113cfdd-33bb-487c-a5d4-c49ff7277cda`
- log lines: 5522–6473 (952 lines)
- log sha256: `2daef662dff8d6aad79b561e225194d5103e13e04844156176e61c25caa12619` (the log's first 6473 lines)
- commits: ccb38706c25909c37f982dde2e0635dcb3dfab23
- tokens: 194 input, 63355 output, 22275756 cache read, 1419345 cache write

Prompts and replies only. Tool calls, tool results, thinking and metadata are
omitted; the full log is kept outside the repository (`agentrail sessions path`).

---

**Assistant** · 2026-11-26T18:28:22.489Z

Step 79 is finished: Windows is now a required CI check, alongside Linux and macOS. It's in **PR #120**: https://github.com/softwarewrighter/reva-fork/pull/120

CI run 45783196599 is green on all three systems:
- **The gate:** all 70 goldens pass on Windows, as on macOS and Linux, and Windows now blocks merges.
- **Installers:** on each system CI builds the demo package's installer, installs it and checks the installed app (6 of 6 checks):
  - Linux: the `.deb`, installed with apt;
  - macOS: the app bundle, ad-hoc signed;
  - Windows: the `setup.exe`, installed silently and per user, with no admin rights needed.

**Bugs found and fixed along the way:**
- **Windows paths:** the CLI treated `D:/...` and `C:\...` as relative, which broke `include` of a full path. CoSy users on Windows would have hit this.
- **Shell exit codes:** `exit 3` read as 0 on Windows. The code is now the same on every system.
- **Missing `sqlite3.dll`:** the Windows installer left it out, so the installed app wouldn't have started on a user's machine. CI hid this, because the build's copy is on its PATH; the check now runs without it.

**What this means for the CoSy lane:** once #120 merges, `just installer <package>` makes a tested installer on all three systems. A Windows `setup.exe` has to be built on Windows: in Git Bash with Visual Studio's build tools.

**Next:**
1. **Step 80, installer tests:** upgrading over an older version and uninstalling keep the user's folder, and the steps for opening an unsigned app the first time, tried on each system.
2. **Step 81, examples triage:** the 16 examples that fault or miss a word.

**Your call:** a release (2026.1126.0) would put CI, the installers and these Windows fixes on `main` and the live site; `main` is still 2026.1121.0. I can do it right after #120 merges, before step 80.

After #120 merges, say "go" for step 80, or "release first".

---

**User** · 2026-11-26T18:41:17.233Z

go and the release

go and then release

---

**Assistant** · 2026-11-26T18:41:36.302Z

#120 isn't merged yet, so I'll stack step 80 on its branch, the same way #119 sat on #118. After step 80, I'll do the release as its own step.

---

**User** · 2026-11-26T19:32:21.644Z

pr merged

---

**Assistant** · 2026-11-26T21:32:11.627Z

Now the PowerShell version, for checking by hand on Windows without Git Bash. CI runs it on the upgraded install, so it's tested too.

Step 80's work is written. The gate is running here, and then CI on all three systems will test it for real, because the installer lifecycle can't run on the shared Mac.

**What's in it:**
- **`scripts/installer-lifecycle.sh`**, run by CI on Linux, macOS and Windows. It takes the demo package through what a user does:
  1. build the installer at 1.0.0 and 1.0.1;
  2. install 1.0.0 the user's way (`.dmg` mounted and copied out, `.deb` with apt, `setup.exe` silently per user);
  3. check it, and take a screenshot with an IUP dialog and turtle drawing open;
  4. add the user's own files, install 1.0.1 over it, and check that the new version runs and the files are kept;
  5. uninstall, and check that the app is gone and the user folder is still there.

  The installers and screenshots are kept as CI artifacts.
- **`tests/check-installed.sh`** checks any installed package app, so CoSy's `check-release.sh --installed` can call it. It checks:
  - the install and `sqlite3.dll`;
  - the signature;
  - `--about` and `version`;
  - the user folder and seed files;
  - user copies read over the package's;
  - writes going to the user folder;
  - `save` and a restart from the image;
  - turtle drawing.

  **`tests/check-installed.ps1`** does the core of this on Windows without Git Bash.
- **A small app change:** the app reads `REVA_DOCUMENTS` in place of `Documents`, so a check never touches a user's real folder.

**Not automated:**
- **IUP and drawing pixels in the installed app:** the browser pixel test covers the same page code, and the screenshots show the installed app.
- **First start of an unsigned download:** the macOS and Windows warnings only appear for files downloaded by a browser. `docs/packaging.md` lists this as a by-hand check before a release.

**Then the release:** when step 80 is green and its PR is merged, I'll bump to 2026.1126.x (N depends on what has already reached `main` today), run every release check, open the PR, and after you merge it, merge `dev` into `main` and publish, as last time.

---

**User** · 2026-11-26T21:34:57.366Z

continue as planned when you are able

---

**Assistant** · 2026-11-26T21:34:59.042Z

I'll continue as soon as the gate finishes. Then I'll commit, push and run CI on all three systems, and open step 80's PR if it's green.

---

**User** · 2026-11-26T21:44:32.512Z

<task-notification>
<task-id>b4us4lhea</task-id>
<tool-use-id>toolu_01FhiTMQF7BTak1ykeKiSEsH</tool-use-id>
<output-file>/private/tmp/claude-501/-Users-mike-github-softwarewrighter-reva-fork/8113cfdd-33bb-487c-a5d4-c49ff7277cda/tasks/b4us4lhea.output</output-file>
<status>completed</status>
<summary>Background command "Syntax check, CHANGES line, run the gate" completed (exit code 0)</summary>
</task-notification>

---

**Assistant** · 2026-11-26T21:46:37.017Z

The gate passed here, and step 80's work is pushed (f6076e3). CI run 45900216484 is now running the installer lifecycle on all three systems, about 45–60 minutes.

When it finishes:
- **Green:** I finish step 80, open its PR, and move on to the release once you merge it.
- **Something fails:** I fix it from the log and push again. Since this is the lifecycle's first run anywhere, the Windows uninstall and the screenshots are the likeliest problems.

---

**User** · 2026-11-26T22:18:36.125Z

<task-notification>
<task-id>bde4u2358</task-id>
<tool-use-id>toolu_01WwWdnCSc6RXxUyXthH3fK5</tool-use-id>
<output-file>/private/tmp/claude-501/-Users-mike-github-softwarewrighter-reva-fork/8113cfdd-33bb-487c-a5d4-c49ff7277cda/tasks/bde4u2358.output</output-file>
<status>completed</status>
<summary>Background command "Wait for CI" completed (exit code 0)</summary>
</task-notification>

---

**Assistant** · 2026-11-27T01:53:34.321Z

Now the macOS diagnosis, with no window: build the demo app and run its `--about` with `REVA_DOCUMENTS` set.

Step 80's first CI run failed on all three systems, though Windows nearly passed. The fixes are made and the gate is running here.

**What the first run showed:**
- **Windows:** nearly everything passed:
  - silent install;
  - the installed-app checks (seeds, user copies read first, writes, `save` and a restart from the image, turtle drawing);
  - an upgrade from 1.0.0 to 1.0.1 that kept the user's files;
  - uninstall with the user folder kept;
  - the PowerShell checks.

  One check failed, there and everywhere: `--about` never showed the package's version, only reva-fork's.
- **Linux:** nothing was installed, because my lifecycle script gave apt a relative path, which it read as a package name.
- **macOS:** install and signature were fine, but the check didn't find the user folder. The same app works here, so the CI log should show why next time.
- **Screenshots:** they work on all three systems, but the turtle window covered the main window, the IUP dialog line had a typo, and on Windows the runner's console was on top.

**Fixes in this round:**
- **`--about` names the package:** `package: Reva Demo 1.0.0`, its install folder marked `(installed)`, and `user folder: …`. This meets the step's requirement, and the checks now read the user folder from it rather than searching for it.
- **Linux:** apt gets an absolute path.
- **Screenshots:** the drawing goes in the main window, the IUP line is fixed, and the Windows console is minimized first.

**Next:** when the gate passes, I push for a second CI run (about 50 minutes).

---

**User** · 2026-11-27T04:28:54.485Z

<task-notification>
<task-id>b1ui9ievw</task-id>
<tool-use-id>toolu_01XD4kdx7ZYdFYHSaLAgpQBa</tool-use-id>
<output-file>/private/tmp/claude-501/-Users-mike-github-softwarewrighter-reva-fork/8113cfdd-33bb-487c-a5d4-c49ff7277cda/tasks/b1ui9ievw.output</output-file>
<status>completed</status>
<summary>Background command "Guard, docs, CHANGES line, run the gate" completed (exit code 0)</summary>
</task-notification>

---

**Assistant** · 2026-11-27T15:27:01.255Z

Second round pushed (512d634): the gate passed here, and CI run 45940194961 is running the installer lifecycle on all three systems, about 50 minutes.

When it finishes:
- **Green:** I finish step 80 and open its PR. Once you merge it, I start the release.
- **Something fails:** I fix it from the log and push again. If macOS still can't find the user folder, `--about` will now show where the app put it.

---

**User** · 2026-11-27T15:27:01.500Z

<task-notification>
<task-id>bag0hpr3l</task-id>
<tool-use-id>toolu_015c911DZjRZLBhUgoQACmfg</tool-use-id>
<output-file>/private/tmp/claude-501/-Users-mike-github-softwarewrighter-reva-fork/8113cfdd-33bb-487c-a5d4-c49ff7277cda/tasks/bag0hpr3l.output</output-file>
<status>completed</status>
<summary>Background command "Wait for CI" completed (exit code 0)</summary>
</task-notification>

---

**Assistant** · 2026-11-27T16:17:09.293Z

Now the Windows failure, from the same run:

Step 80's second CI run: **Linux passed the whole installer lifecycle**. Install, the checks, the upgrade that keeps the user's files, and the uninstall that keeps the user folder all work there. macOS and Windows each had one problem, both now fixed. The gate is running here before the third CI run.

**macOS** (a real bug, which a user could also hit):
- **What happened:** the installed app didn't find its package when started through a path with a symlink in it. On GitHub's runners the temp folder is under `/var`, which is a symlink to `/private/var`, and Tauri refuses such paths for security.
- **Confirmed here:** the same app found its package through a normal path and lost it through a symlink.
- **Fix:** the app now finds its own `Resources` by its real path when Tauri refuses, with a test, and the lifecycle script uses real paths.

**Windows:** everything the bash checks cover passed: install, upgrade, uninstall, user files kept. Only the new PowerShell script failed: Windows PowerShell 5.1 treated WebView2's harmless log line on stderr as fatal. Fixed.

When the gate passes, I push for the third run (about 50 minutes). If it's green, step 80 is done and I open its PR, then the release after you merge it.

---

**User** · 2026-11-27T16:37:54.566Z

continue? status?

---

**Assistant** · 2026-11-27T16:38:04.776Z

Still working. The gate started at 08:34 and is on the desktop component now; it usually takes about 20 minutes, so it should finish around 08:55. Nothing new is committed since my last report.

**Step 80 so far:**
- **Linux:** the whole installer lifecycle passes in CI: install, checks, upgrade with the user's files kept, uninstall with the user folder kept.
- **macOS:** fixed a real bug. An app started through a path with a symlink in it couldn't find its package.
- **Windows:** all the bash checks passed. Fixed the PowerShell checker, which treated WebView2's log line on stderr as fatal.

**Next, in order:**
1. If the gate passes, commit and push for the third CI run (about 50 minutes, so roughly 09:50).
2. If all three systems are green, finish step 80 and open its PR.
3. After you merge it, the release: bump `VERSION` to 2026.1127.0, run the release checks and open the release PR. When you merge that, I merge `dev` into `main` and publish.

---

**User** · 2026-11-27T16:42:05.928Z

continue when the gate passes

---

**Assistant** · 2026-11-27T16:45:31.008Z

The gate failed. Looking at why:

The split is clean: fmt, clippy and tests pass. A quick sw-checklist check, then the full gate again:

The gate failed on one rule: after this round's macOS fix, `reva-desktop-user/src/user.rs` had 8 functions, and the code-structure limit is 7. Every test, golden and release check passed.

Following your "move out, never combine" rule, I split it in two:
- `user.rs`: the user folder (where it is, copying the starting files);
- `install.rs`: finding the shipped package, and what `--about` says about it.

Nothing else changed; fmt, clippy and the desktop tests pass. The full gate is running again, about 20 minutes. When it passes, I commit and push for the third CI run on all three systems (about 50 minutes). If that's green, I finish step 80 and open its PR, then start the release once you merge it.

---

**User** · 2026-11-27T17:10:05.697Z

<task-notification>
<task-id>bm13rnczx</task-id>
<tool-use-id>toolu_01Vcqn9Njbi7VrkhrDgSuC9E</tool-use-id>
<output-file>/private/tmp/claude-501/-Users-mike-github-softwarewrighter-reva-fork/8113cfdd-33bb-487c-a5d4-c49ff7277cda/tasks/bm13rnczx.output</output-file>
<status>completed</status>
<summary>Background command "Run sw-checklist then the full gate" completed (exit code 0)</summary>
</task-notification>

---

**Assistant** · 2026-11-27T17:31:53.300Z

The gate passed here, with 0 sw-checklist failures. The fixes are pushed (c3e5fe3), and the third CI run (46041359074) is running the installer lifecycle on all three systems, about 50 minutes.

If it's green, I finish step 80 and open its PR. Once you merge it, I start the release. If it fails, I fix it from the log and push again.

---

**User** · 2026-11-27T18:03:45.976Z

<task-notification>
<task-id>bb05u84qe</task-id>
<tool-use-id>toolu_01GqumYuGzyvqNTtnmw3k4FQ</tool-use-id>
<output-file>/private/tmp/claude-501/-Users-mike-github-softwarewrighter-reva-fork/8113cfdd-33bb-487c-a5d4-c49ff7277cda/tasks/bb05u84qe.output</output-file>
<status>completed</status>
<summary>Background command "Wait for the third CI run" completed (exit code 0)</summary>
</task-notification>

---

**Assistant** · 2026-11-27T18:17:05.602Z

Linux passed the whole lifecycle again. Windows passed everything except two checks, both in the PowerShell script; the bash checks passed. macOS was cancelled. Checking why:

The `.ps1` isn't run by the gate, but the rules require the full gate for any non-docs commit, so running it (about 20 minutes) before pushing:

Step 80 still needs one more CI run: the third one passed on Linux, nearly passed on Windows, and never ran on macOS.

- **Linux:** the whole installer lifecycle passed again: install, checks, upgrade from 1.0.0 to 1.0.1 with the user's files kept, uninstall with the user folder kept.
- **Windows:** the same lifecycle passed in full. Only two checks in the PowerShell script failed: writing a file, and `save` then a restart. Windows PowerShell 5.1 strips the double quotes inside arguments it passes to a program, so Forth strings like `" /check-written.txt"` arrived broken. The script now escapes them.
- **macOS:** cancelled before starting, because GitHub never assigned a runner (the same problem as before, not our code). The next push runs it again.

The gate is running here, about 20 minutes. Then I push for the fourth CI run (about 50 minutes). If all three systems are green, I finish step 80 and open its PR, then the release after you merge it.

---

**User** · 2026-11-27T18:34:52.550Z

<task-notification>
<task-id>bwfal7w42</task-id>
<tool-use-id>toolu_01KkrGHtGxRrkSJmTAJbnGds</tool-use-id>
<output-file>/private/tmp/claude-501/-Users-mike-github-softwarewrighter-reva-fork/8113cfdd-33bb-487c-a5d4-c49ff7277cda/tasks/bwfal7w42.output</output-file>
<status>completed</status>
<summary>Background command "CHANGES line and run the gate" completed (exit code 0)</summary>
</task-notification>

---

**Assistant** · 2026-11-27T18:48:41.245Z

The gate passed here and the fix is pushed (8299d29). The fourth CI run (46052033087) is running on all three systems, about 50 minutes.

When it's done:
- **All green:** I finish step 80 and open its PR, then start the release once you merge it.
- **macOS gets no runner again:** I re-run just that job.
- **Anything else fails:** I fix it from the log and push again.

---

**User** · 2026-11-27T19:31:41.041Z

<task-notification>
<task-id>be61869nq</task-id>
<tool-use-id>toolu_01LCaLkmBUZbywRXTTWzTu5M</tool-use-id>
<output-file>/private/tmp/claude-501/-Users-mike-github-softwarewrighter-reva-fork/8113cfdd-33bb-487c-a5d4-c49ff7277cda/tasks/be61869nq.output</output-file>
<status>completed</status>
<summary>Background command "Wait for the fourth CI run" completed (exit code 0)</summary>
</task-notification>

---

**Assistant** · 2026-11-27T20:54:00.410Z

All three systems are green, with no failures and no missing screenshots. Looking at the screenshots before I describe them:
