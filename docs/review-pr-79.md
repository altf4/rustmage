# Independent review: transactional pre-entry life payments

PR: https://github.com/altf4/rustmage/pull/79
Implementation reviewed: 1d2590b.
Regression/documentation delta reviewed: 9c46aa7.

Two independent read-only subagents reviewed this milestone:

- Rules: life-payment legality, pre-entry replacements, mixed/APNAP transactions,
  zero-life/SBA timing, entry paths and continuations, knowledge projection,
  all 20 complete admissions and compiler gates.
- Library: owned entry batches and resume/shuffle order, action validation,
  visibility/schema 33, state accounting, entry paths, the ordinary-land direct
  path and all 20 admissions.

Both reported no actionable findings. Both independently passed 510 Rust tests
(502 unit and eight integration), 90 compiler tests and catalogue consistency at
1d2590b. The library reviewer also passed 57 release Python tests and 20 native
games: 169 paid entries, 14 declines and 1,548 replayed boundaries.

Both examined a possible interaction with existing low-life lands. The official
[Duskmourn release notes](https://magic.wizards.com/en/news/feature/duskmourn-house-of-horror-release-notes)
explicitly require pre-payment life totals for simultaneous Lakeside Shack and
Breeding Pool entry. The existing frozen predicate is correct. The author added
that official case as a regression, covering both payment branches, both movement
orders and life 13/14. Both reviewers independently inspected the delta, verified
HEAD 9c46aa7, reran focused tests and again reported no findings. Neither reviewer
edited files, mutated Git or posted externally.

Final author validation: 511 Rust tests (503 unit and eight integration), 90
compiler tests, 57 rebuilt release Python tests, consistency, rustfmt and Clippy
with warnings denied. One hundred supported Standard games: 42,623 decisions,
847 paid entries, 86 declines and 7,842 cloned/replayed boundaries. Maximum
initialized state was 12,540 bytes, excluding allocator metadata, spare capacity,
external snapshots and immutable definitions. See entry-life.md for explicit
synthetic continuation and type-only structural fixture boundaries.

Coverage is 878/5,164 Standard identities, six regression definitions and 41 token
factories. Full Standard semantics, general replacement/layer handling, native
RL/search integration and an XMage speed comparison remain incomplete. These
reviews do not certify full-format conformance.
