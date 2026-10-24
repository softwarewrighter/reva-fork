# Licensing

This repository combines the original Reva Forth with new work. Each part
keeps its own terms.

| Part | Terms | File |
|---|---|---|
| **Original Reva Forth** by Ron Aaron and the Reva development team: `src/` (including `src/reva.f`), `lib/`, `examples/`, `bin/`, `tools/`, and everything else not listed below | Public domain, with an unconditional fall-back permission grant where public-domain dedication is not recognised | [LICENSE](LICENSE) |
| Embedded x86 assembler in `src/reva.f` (adapted from FirmWorks) | As noted in LICENSE | [LICENSE](LICENSE) |
| IUP libraries in `bin/` | IUP license | [LICENSE.IUP](LICENSE.IUP) |
| **New portions**: the portable core under `rust/`, `README.md`, `LICENSING.md`, `DEPRECATED.md` | MIT, portions copyright © 2026 Michael A Wright | [LICENSE-MIT](LICENSE-MIT), [rust/LICENSE](rust/LICENSE) |

Notes:

- The portable core *embeds and runs* `src/reva.f`. That file stays in the
  public domain; the MIT terms cover only the new code around it.
- The original README is preserved unchanged as [README.reva](README.reva).
- Reva's author asks for credit where Reva is useful; this project credits
  Ron Aaron (ron@ronware.org) as the original author.
