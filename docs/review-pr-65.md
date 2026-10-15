# Independent review: discard/cycling events and cleanup

PR: https://github.com/altf4/rustmage/pull/65
Implementation reviewed: 1ab61bd.

Two independent read-only subagents reviewed this milestone:

- Rules: actual discard causes, batch grouping, self-discard/cycling source
  identity, captured counts/LKI, APNAP, private cleanup/repeated cleanup,
  creature-or-Vehicle targeting/ward, all eight admissions and compiler gates.
- Library: captured owned event facts, source identity/LKI, cost/effect paths,
  simultaneous private choices, cleanup/accounting, schema 28 and all eight
  admissions.

Both reported no actionable findings. Both independently passed 395 Rust tests
(387 unit and eight integration), 70 compiler tests and catalogue consistency.
The library reviewer also passed 43 release Python tests and 20 native games,
with 237 event-bound triggers and 804 cloned/replayed boundaries. Neither
reviewer edited files, mutated Git or posted externally.

Author validation passed those suites, rustfmt and Clippy with warnings denied.
One hundred supported Standard games completed naturally: 48,493 decisions,
1,088 event-bound announcements, 900 cycling-bound announcements, 31 multi-card
announcements, 14 cleanup discard choices and 3,610 cloned/replayed boundaries.
Maximum initialized state was 12,980 bytes, excluding allocator metadata, spare
capacity, snapshots and static definitions. Direct cause-hook/LKI and
metadata-only unsupported Vehicle fixtures are distinguished from printed-card
paths in discard-events.md; no partial Vehicle is admitted to submitted decks.

Coverage is 767/5,164 Standard identities, six regression cards and 39 token
factories. Broader discarded-card bindings, conditional/reflexive triggers,
variable costs, remaining card semantics and native RL/search integration remain
incomplete. These reviews do not certify full Standard conformance or establish
an XMage speed comparison.
