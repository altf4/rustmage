# Independent review: committed hybrid spell costs

PR: https://github.com/altf4/rustmage/pull/73
Initial implementation: 1418b57. Final implementation reviewed: eacec40.

Two independent read-only subagents reviewed this milestone:

- Rules: hybrid equivalents and CR601.2b declaration order, exact funding/payment,
  prefunded shortcut allocations, cast timing, printed mana value, all 18 complete
  admissions and compiler gates.
- Library: owned commitment/continuations, funding caches, duplicate suppression,
  immutable metadata, schema 32 privacy, snapshots and all 18 complete admissions.

The rules reviewer found a P2 in shared modal-hybrid declaration order. Commit
39d6cd3 changed it to mode, hybrid equivalent, targets, then mana/payment. Both
follow-up reviewers then found a P2 in payment observation during the earlier
mode stage: an uncommitted hybrid cost still reached the fixed funding projector.
Commit eacec40 centralizes the commitment predicate for routing and observation.
Both reviewers rechecked that commit and reported no remaining findings.
Neither issue affected a current complete printed admission; both shared paths
were fixed before merge. Synthetic declaration fixtures and their limits are
explicit in hybrid-spells.md and engine_hybrid_tests.rs.

Both initial independent audits passed 468 Rust tests, 82 compiler tests and
catalogue consistency. The library reviewer also passed 51 release Python tests
and 20 native games, with 162 staged hybrid payments, 53 prefunded shortcuts and
943 replayed boundaries. Final re-reviews passed all 18 hybrid fixtures; the
library reviewer additionally reran all 51 release Python tests. Neither reviewer
edited files, mutated Git or posted externally.

Final author validation passed 469 Rust tests (461 unit and eight integration),
82 compiler tests, 51 rebuilt release Python tests, consistency, rustfmt and Clippy
with warnings denied. One hundred supported Standard games completed naturally:
45,437 decisions, 801 committed choices, 801 staged payments, 300 prefunded
shortcuts and 4,674 cloned/replayed boundaries. Maximum initialized state was
13,144 bytes, excluding allocator metadata, spare capacity, external snapshots
and static definitions.

Coverage is 834/5,164 Standard identities, six regression cards and 40 token
factories. Remaining card semantics, other cost mechanics and native RL/search
integration remain incomplete. These reviews do not certify full Standard
conformance or establish an XMage speed comparison.
