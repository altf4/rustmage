# Independent review: creature-death history and morbid rules

PR: https://github.com/altf4/rustmage/pull/121
Implementation reviewed: 50f2aaf8442f7ddb4a029ca0b5b96f372e1b3774.
Base: main cab54e2864a5b31da80c98a68c0b9318f3929941.

Two independent read-only subagents reviewed the exact revision. Both report
no actionable findings. Neither edited files, mutated Git or posted externally.

The rules reviewer checked the shared actual-death transition boundary,
simultaneous batches/linked returns, token and reentry semantics, turn reset,
morbid entry/cost/intervening-if behavior, all five full bodies and residual
gates. All five card and two helper Java hashes match the pinned revision.
Independently passed 789 Rust tests, 145 compiler/data tests, both public Python
morbid payment/private connive replay regressions and catalogue consistency.

The library reviewer checked checked history accounting, atomic capacity
rollback, direct/staged entry visibility, snapshots and privacy, public schema
43, exact shared cost/guard semantics and whole-body gates. Independently
passed 789 Rust, 145 compiler/data and 103 release Python tests, catalogue/diff
checks and all seven source hashes. The hundred-game native workload exactly
reproduced 60,021 decisions, 180,063 public checks, 26,092 clone/restore/replay
boundaries and 20,416 maximum initialized bytes. All five entries, three guarded
step-trigger kinds and both Bat payment/Slasher entry branches were reached.

Author checks also pass rustfmt and Clippy with warnings denied. See
[creature-death-history.md](creature-death-history.md) for provenance, pinned
rules, validation, workload policy, state accounting and performance limits.
Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 1,099/5,164 Standard identities, with 4,065 unsupported. Initial
logical state remains 19,720 bytes on the measured platform; observation schema
43 adds public death history, ABI 1 and protocol 2 remain unchanged. Full
Standard and native RL/search acceptance remain incomplete and active.
