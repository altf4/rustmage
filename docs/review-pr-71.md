# Independent review: once-per-turn trigger limits

PR: https://github.com/altf4/rustmage/pull/71
Implementation reviewed: 3c66afd.

Two independent read-only subagents reviewed this milestone:

- Rules: trigger-time consumption, independent ability/incarnation keys,
  simultaneous death LKI, APNAP, turn resets/replay, event unwrapping and captured
  context suppression, typed entry observers, all 16 complete admissions and
  compiler gates.
- Library: limit ownership/reset, original source identities, event unwrapping,
  suppressed-queue context handling, snapshots/accounting, schema 31 privacy and
  all 16 complete admissions.

Both reported no actionable findings. Both independently passed 451 Rust tests
(443 unit and eight integration), 79 compiler tests and catalogue consistency.
The library reviewer also passed 49 release Python tests and 20 native games,
with 51 consumed limits and 1,495 replayed boundaries. Neither reviewer edited
files, mutated Git or posted externally.

Author validation passed those suites, rustfmt and Clippy with warnings denied.
One hundred supported Standard games completed naturally: 52,855 decisions,
335 consumed trigger limits, 1,537 turn transitions and 8,126 cloned/replayed
boundaries. Maximum initialized state was 13,232 bytes, excluding allocator
metadata, spare capacity, external snapshots and static definitions. The failed
announcement fixture is explicitly synthetic; printed-card paths and residual
compiler gates are listed in limited-triggers.md.

Coverage is 816/5,164 Standard identities, six regression cards and 40 token
factories. Remaining card semantics, replacement/prevention/layer machinery and
native RL/search integration remain incomplete. These reviews do not certify
full Standard conformance or establish an XMage speed comparison.
