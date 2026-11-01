# Licensing

This repository combines the original Reva Forth with new work. Each part
keeps its own terms.

| Part | Terms | File |
|---|---|---|
| **Original Reva Forth** by Ron Aaron and the Reva development team: `src/` (including `src/reva.f`), `lib/`, `examples/`, `tools/`, `www/` (the old wiki), `legacy/` (the deprecated x86 core, binaries and scratch files), and everything else not listed below | Public domain, with an unconditional fall-back permission grant where public-domain dedication is not recognised | [LICENSE](LICENSE) |
| Embedded x86 assembler in `src/reva.f` (adapted from FirmWorks) | As noted in LICENSE | [LICENSE](LICENSE) |
| IUP libraries in `legacy/bin/` | IUP license | [LICENSE.IUP](LICENSE.IUP) |
| Other third-party binaries and notices | Each listed with its license | [docs/licenses.md](docs/licenses.md) |
| **New portions**: the portable core under `components/`, `scripts/`, `README.md`, `LICENSING.md`, `DEPRECATED.md`, `legacy/README.md`, `docs/`, `www/README.md` | MIT, Copyright (c) 2026 Michael A. Wright | [LICENSE-MIT](LICENSE-MIT), [components/LICENSE](components/LICENSE) |

Notes:

- The repository contains no GPL, LGPL or AGPL code, and
  `scripts/license-check.sh` keeps it that way (see
  [docs/licenses.md](docs/licenses.md) for the policy, the reviewed binaries
  and what was removed).
- The portable core *embeds and runs* `src/reva.f`. That file stays in the
  public domain; the MIT terms cover only the new code around it.
- The original README is preserved unchanged as [orig-README](orig-README).
- Reva's author asks for credit where Reva is useful; this project credits
  Ron Aaron (ron@ronware.org) as the original author.
