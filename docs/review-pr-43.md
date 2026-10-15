# Independent review: legend rule and state-based choices

PR: https://github.com/altf4/rustmage/pull/43
Implementation reviewed: 6592c5f against main.

Two independent read-only subagents reviewed this milestone:

- Rules: frozen simultaneous state-based events, APNAP legend choices, exact
  incarnations, linked-return fixed points, cleanup resumption, all ten new
  definitions and complete-text compiler gating.
- Library: legality and atomic rejection, continuation ownership, snapshot replay,
  storage accounting, schema 17 public observations and compiler admission.

Both reported no actionable findings. Both independently passed 216 Rust tests,
37 script tests and catalogue consistency. The library reviewer also passed all
24 release Python tests and 20 native Standard games with 28 cloned/replayed
legend choices. Neither reviewer edited files or posted externally.

Author validation passed rustfmt, Clippy with warnings denied, catalogue
consistency and the Rust/script/Python suites. One hundred supported Standard
self-play games completed naturally: 64,147 decisions and 124 legend choices,
cloned and restored/replayed at every such decision. Rust fixtures cover public
APNAP selections, own-group ordering, simultaneous deaths and losses, a dying
legend chosen to remain, indestructible, token cessation, counter cancellation,
linked-return repetition, landfall triggers and cleanup priority resumption.

A local 1,000-game limited-card smoke benchmark measured approximately 1,380
games/second and 788,123 decisions/second; clone/restore of its 8,208-byte initial
payload measured 170/163 ns. It includes smoke-policy observations, excludes
model/tensor/JSON costs, and is neither a full Standard workload nor an XMage
performance comparison. These numbers are one local measurement, not a guarantee.

Coverage is 547/5,164 Standard identities, six regression cards and 31 token
factories. Control/name/type changes, copies, legendary spell restrictions, other
state-based rules and the remaining Standard/RL semantics remain outstanding.
These reviews do not certify full Standard conformance.
