# Independent review: milling and exact-batch recovery

PR: https://github.com/altf4/rustmage/pull/57
Implementation reviewed: 7f280e6 against main.

Two independent read-only subagents reviewed this milestone:

- Rules: mill batching, short/empty libraries, optional mill legality,
  exact-incarnation recovery and compiler scope binding, public hand knowledge,
  combined entry/death triggers and all 20 complete admissions.
- Library: owned mill/recovery state, visitor validation and privacy,
  snapshots/accounting, program continuations, compiler gates and all 20
  complete admissions.

Both reported no actionable findings. Both independently passed 325 Rust tests
(317 unit and eight integration), 58 compiler tests and catalogue consistency.
The library reviewer also passed all 35 release Python tests and 20 native games
with 39 recoveries, 22 declines and 432 replayed boundaries. Neither reviewer
edited files, mutated Git or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, consistency and
the complete Rust/compiler/release Python suites. One hundred supported Standard
games completed naturally: 64,792 decisions, 5,173 newly public milled graveyard
objects, 208 recoveries, 109 declines, 79 life-based mill casts and 2,254
cloned/replayed boundaries. Maximum initialized state was 11,628 bytes for this
workload, excluding allocator metadata, spare capacity, snapshots and static
definitions. Synthetic composition tests are distinguished from complete-card
admissions in milling.md.

Coverage is 690/5,164 Standard identities, six regression cards and 36 token
factories. Wider graveyard events, replacements, recovery filters and remaining
card semantics and native RL/search integration are still incomplete. These
reviews do not certify full Standard conformance or establish an XMage
performance comparison.
