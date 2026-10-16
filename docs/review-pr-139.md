# PR #139 independent reviews

PR: https://github.com/altf4/rustmage/pull/139
Issue: https://github.com/altf4/rustmage/issues/138
Base main: `e76600523a7c4c3129c000432542de39955acd1a`
Initial source: `3f881915b2cd2ef02ee3f76d87d653a93088a83a`
Corrected and cleared source: `b523a7778c8fe67bf2384c3ddfad00bceb69a29e`

Two independent read-only subagents audited the initial source and re-reviewed
its correction. Neither edited files or made Git mutations.

## Finding and correction

The rules reviewer found a P2: synthetic exhaust loyalty abilities reused a bit
that persisted across turns, causing the loyalty mask to lock ordinary loyalty
abilities forever. No admitted card was affected. The compiler now rejects exhaust
with any loyalty cost pending separate tracking. Positive, zero and negative
loyalty regressions reject the composition; ordinary controls still compile.
Generated definitions and coverage change only in their fingerprint.

## Rules review

Cleared the corrected source with no remaining actionable findings. Independently
passed the three exhaust compiler methods, catalogue and diff checks. Initial audit
passed 869 Rust and 163 compiler/data tests, both new release Python methods and
all fourteen unique pinned Java hashes. Audited full bodies, per-slot use limits,
turn persistence, new object identity resets and activation timing against the
official Aetherdrift release notes and comprehensive rules.

## Library review

Cleared the corrected source and independent positive/zero/negative loyalty
reproductions with no remaining findings. Independently passed all 164 corrected
compiler/data tests, catalogue and diff checks. Initial audit passed 869 Rust and
119 fresh release Python tests and all 23 provenance entries across fourteen Java
files. Audited authoritative direct/staged/immediate mana activation, funding
bounds, payment atomicity, private continuations, snapshots, stale revisions,
schema 49, memory accounting and whole-body compiler gates.

Independently reproduced 100 native games: 58,634 decisions, 245,381 mask/body
checks, 904 exhaust activations, 4,401 persisted turn boundaries, 40 private discards,
83 reset returns, 26,959 clone/restore replay boundaries and 22,324 maximum logical
initialized bytes. The author also reran all 869 Rust, 164 compiler/data and 119
fresh release Python tests on the corrected fingerprint, plus fmt, Clippy,
catalogue and diff checks.

Only this record and the progress delivery row follow the cleared source.
Full Standard and broader native RL/search acceptance remain incomplete.
