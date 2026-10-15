# Independent review: connive continuations

PR: https://github.com/altf4/rustmage/pull/69
Implementation reviewed: 9dd574c.

Two independent read-only subagents reviewed this milestone:

- Rules: controller and last-known information, original incarnation counters,
  fixed amounts and short hands, sequential APNAP choices, private discards,
  deferred triggers, target groups, subtype-entry/named bindings, all 16 complete
  admissions and compiler gates.
- Library: sequence ownership/order, controller and incarnation safety, private
  discards, continuation replay/accounting, target groups, schema 30 and all 16
  complete admissions.

Both reported no actionable findings. Both independently passed 433 Rust tests
(425 unit and eight integration), 76 compiler tests and catalogue consistency.
The library reviewer also passed 47 release Python tests and 20 native games,
with 148 connive operations and 1,551 replayed boundaries. Neither reviewer
edited files, mutated Git or posted externally.

Author validation passed those suites, rustfmt and Clippy with warnings denied.
One hundred supported Standard games completed naturally: 50,218 decisions,
732 visible connive requests/discard choices, 657 nonland discards, 646 observed
immediate counter increases and 8,091 cloned/replayed boundaries. Maximum
initialized state was 12,364 bytes, excluding allocator metadata, spare capacity,
external snapshots and static definitions. Fixed-N and multiple-conniver fixtures
are distinguished from admitted printed-card paths in connive.md.

Coverage is 800/5,164 Standard identities, six regression cards and 40 token
factories. Remaining card semantics, replacement/prevention/layer machinery and
native RL/search integration remain incomplete. These reviews do not certify
full Standard conformance or establish an XMage speed comparison.
