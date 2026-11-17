# Asks for agentrail

Feature requests from reva-fork to agentrail, the saga tool its sessions
follow (`.agentrail/`, CLAUDE.md). The first is about space.

## 1. Session logs outgrow the repository (answered)

**Answered (agentrail of 2026-11-15, adopted 2026-11-17).** Each ask
below became a command:

| Ask | agentrail now |
|-----|---------------|
| 1. Do not commit logs | kept in `~/.agentrail/sessions/<repo>/` (`agentrail sessions path`); `agentrail sessions migrate` moved this repository's out |
| 2. A shadow store | `agentrail shadow set ../reva-fork-agentrail`, `push`, `pull`, `status`; it replaces `scripts/agentrail-shadow.sh` (removed) |
| 3. A text digest per step | `steps/NNN-slug/transcript.md`: the prompts and replies, with the log's hash |
| 4. A size guard | `agentrail check-size`, and `complete` leaves out a file over the limit |
| 5. Survive history rewrites | `agentrail remap`: reva-fork's steps were remapped with the 2026-11-13 commit map; two steps had recorded commits amended away before the rewrite, matched by time (`--from`/`--to`) |
| 6. A privacy note | in the briefing (CLAUDE.md) |

What follows is the ask as filed.

**What happened (2026-11-13).** agentrail keeps each agent session's log
in `.agentrail/sessions/ID.jsonl`, and the saga-record commits commit it.
One local-lane session ran for 67 steps over 18 days. Its log reached
105 MB, and GitHub refused the push of PR #106 (*File ... is 100.31 MB;
this exceeds GitHub's file size limit of 100.00 MB*). Every saga record
had committed the whole log again. History held 67 versions of it, 3.16
GB uncompressed. Locally, `.git` was 992 MB.

**What we did.** Removed `.agentrail/sessions/` from all history with
`git filter-repo` and force-pushed. `.git` is now 27 MB with every
branch. The logs are now ignored in reva-fork.
`scripts/agentrail-shadow.sh` copies them, xz-compressed, to a private
repository, `softwarewrighter/reva-fork-agentrail`, after each saga record.
That repository also keeps a bundle of the history from before the rewrite.
It was needed because steps record commit hashes, and the rewrite changed
them all.

**A second problem: privacy.** The log quotes every file the agent
reads, every command's output, and its reasoning. reva-fork is public. The
log quoted CoSy files that reva-fork must not redistribute. A secret
read during a session would be published the same way.

### Where the space goes

| What | Size | Share |
|------|------|-------|
| All of `.agentrail/` in history, before the rewrite | 3.165 GB | 100% |
| The session log's 67 committed versions | 3.165 GB | 99.99% |
| Everything else: plan, saga and the 87 steps' `prompt.md`, `step.toml` and `summary.md` (239 files) | 0.12 MB | 0.004% |

The current log, 105 MB, broken down:

| Part of the log | MB |
|-----------------|----|
| Tool results: the files read and command output, each stored twice (`toolUseResult` and `message`) | 28 |
| Other user-side records: hooks, reminders, attachments | 20 |
| API request bookkeeping (`api-request`, `-blob`, `-shape`) | 20 |
| Assistant records' metadata, repeated on every record | 20 |
| Assistant thinking | 6.4 |
| Assistant tool calls | 3.8 |
| File-history snapshots | 3.7 |
| Mode, title, PR-link and other small records, about 2,200 each | 2 |
| **The conversation itself: the user's 256 typed prompts and the assistant's replies** | **0.67** |

xz compresses the log 7.6 times, to 13.9 MB.

### How much could be avoided

- **Not committing the log:** all 3.16 GB in history, and 105 MB in the
  working tree. The saga's own record stays at 0.12 MB.
- **Committing one log once:** the log only grows, so its latest version
  holds every earlier one. Committing it at the session's end instead of
  at every step would have stored 105 MB once instead of 3.16 GB, a 30th.
  Compressed, 13.9 MB.
- **Keeping only the conversation:** 0.67 MB, a 157th of the log,
  readable in a diff. The step summaries already say what each step did.

### Ask

1. **Do not commit session logs by default.** Keep them under
   `.agentrail/sessions/` with a `.gitignore` that agentrail writes. Or
   store them outside the repository, for example in
   `~/.agentrail/<repo>/sessions/`.
2. **A shadow store for large or private artifacts:**
   `agentrail shadow push|pull`. It copies the logs, compressed, to a
   configured private git repository or directory, as reva-fork's
   `scripts/agentrail-shadow.sh` does, and can restore them.
3. **A text digest of a completed step, committed in place of the log:**
   the user's prompts and the assistant's replies, with no tool results,
   thinking or metadata, plus the step's commits, a token count, and the
   hash of the full log in the shadow store. It's small enough to read and
   review, about 1% of the log.
4. **A size guard.** `agentrail complete` warns when a file under
   `.agentrail/` passes 10 MB. It refuses to stage one over 95 MiB, and
   names the file.
5. **Survive history rewrites.** Steps record full commit hashes, and
   any rewrite (filter-repo, a rebase before merge, a squash merge) leaves
   them dangling. `agentrail remap <commit-map>` would rewrite them from
   git filter-repo's `commit-map`. `audit` already falls back to commit
   times.
6. **Privacy note in the briefing:** session logs quote everything read.
   In a public repository they publish it.
