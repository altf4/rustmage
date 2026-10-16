# Independent review: conditional entry counters

PR: https://github.com/altf4/rustmage/pull/119
Final implementation reviewed: 1004239d1341bf31cc8ed2c6edcfb0335fc4b987.
Base: main 958a08f6454e5f79b56a0bcfc52395cd36e8b1b6.

Two independent read-only subagents reviewed the initial implementation and the
final consistency follow-up. Both report no remaining actionable findings.
Neither edited files, mutated Git or posted externally.

The rules reviewer checked all three full admissions, pre-entry timing,
controller cast/attack histories, simultaneous counter freezing, existing fixed
counters/loyalty and compiler residual gates. Oracle and all three pinned Java
hashes match. Independently passed 778 Rust tests at the initial revision, two
focused compiler regressions, release Python entry/replay cases and catalogue
consistency. Re-reviewed the final raw-batch delta and passed all ten matching
entry-counter-related tests, including both raw entry orders and rollback.

The library reviewer checked all production Battlefield entry paths, immutable
quantity/guard rules, checked arithmetic, snapshot ownership and schema/state
compatibility. Independently passed 778 Rust, 143 compiler/data and 101 release
Python tests, catalogue/diff checks and the 100-game native workload at the
initial revision. That run exactly reproduced 61,586 decisions, 286 checked
entries, 28,781 clone/restore/replay boundaries, six zero/positive entry branches
and 20,564 maximum initialized state bytes. All three source hashes verified.
At the final revision, independently passed all 779 Rust tests, all eight new
entry-counter regressions and diff checks.

The final follow-up freezes counter quantities in the lower-level raw batch
helper too. Current gameplay Battlefield batches already use staged entries;
the helper's sequential calculation appeared only in fixtures. Both reviewers
confirmed the consistency fix and its both-order snapshot/fork/replay regression.

Author validation also passes formatting and Clippy with warnings denied. See
[conditional-entry-counters.md](conditional-entry-counters.md) for provenance,
rules, workload policy and limitations. Only this review record and progress
bookkeeping follow the reviewed source. Coverage is 1,094/5,164 Standard
identities, with 4,070 unsupported. Observation 42, ABI 1, protocol 2 and initial
logical state 19,720 bytes remain unchanged. Full Standard and native RL/search
acceptance remain incomplete; the full goal remains active.
