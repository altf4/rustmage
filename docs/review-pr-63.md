# Independent review: hand-zone cycling and typecycling

PR: https://github.com/altf4/rustmage/pull/63
Implementation reviewed: 56b0adb.

Two independent read-only subagents reviewed this milestone:

- Rules: hand activation/discard costs, exact source incarnation/funding,
  priority and mana-window restrictions, qualified typecycling searches,
  reveal/fail-to-find/shuffle and all 31 complete admissions/compiler gates.
- Library: source-zone legality, payment source obligations, stack independence,
  search ownership, privacy, snapshots, schema 27 and all 31 admissions.

Both reported no actionable findings. Both independently passed 380 Rust tests
(372 unit and eight integration), 67 compiler tests and catalogue consistency.
The library reviewer also passed 41 release Python tests and 20 native games,
including 699 cycling activations and 5,999 cloned/replayed boundaries. Neither
reviewer edited files, mutated Git or posted externally.

Author validation passed those suites, rustfmt and Clippy with warnings denied.
One hundred supported Standard games completed naturally: 124,297 decisions,
3,499 cycling activations, 2,715 searches, 1,642 successful searches, 1,073
failed-to-find searches and 29,377 cloned/replayed boundaries. Maximum initialized
state was 12,436 bytes, excluding allocator metadata, spare capacity, snapshots
and static definitions. Synthetic source-reincarnation interruption and the
current lack of a wholly admitted Halfling search candidate are documented in
cycling.md; no partial card is enabled to fill these fixtures.

Coverage is 759/5,164 Standard identities, six regression cards and 39 token
factories. Cycling/discard triggers, variable costs and cost modifiers, remaining
card semantics and native RL/search integration remain incomplete. These reviews
do not certify full Standard conformance or establish an XMage speed comparison.
