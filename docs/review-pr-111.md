# Independent review: locked generic spell reductions

PR: https://github.com/altf4/rustmage/pull/111
Final implementation reviewed: 49f1ca65a88b7ab2c9887282c20e8a448f39ec9e.
Base: main 37e9005a1fa9e172a7aaf54df574738c509e120d.

Two independent read-only subagents reviewed the implementation at 9d408b5,
then re-reviewed the final revision above. Neither edited files, mutated Git
or posted externally. Both report no remaining actionable findings.

The rules reviewer found one P2: the compiler accepted hand-count discounts,
although affordability queries run before the proposed spell leaves hand under
CR 601.2a. Those clauses now remain unsupported, with extraction and whole-card
admission regression coverage. None of the twenty admissions depends on hand
size. The first new synthetic test omitted required oracle_id metadata; that
fixture was corrected before final review. The reviewer verified the final
revision, three focused compiler tests, twenty-one native reduction tests and
catalogue consistency. Initial independent checks passed all 740 Rust tests,
133 compiler tests, three Python regressions and all pinned Java hashes.

The library reviewer checked cost locking, shared funding/validation, reducer
indexing, hybrid allocation deduplication, snapshots/privacy and admission gates.
Initial independent checks passed 740 Rust tests, 133 compiler/data tests, 95
fresh-release Python tests, catalogue consistency and all twenty Java hashes.
Twenty legal Standard games exercised all twenty cards and reduction rules on
paid spells: 13,891 decisions and 5,341 clone/replay boundaries. Final re-review
passed all 134 compiler/data tests, hand-count boundary rejection cases,
catalogue consistency and diff checks. Generated runtime definitions and
coverage remain unchanged except for the fingerprint after the gate fix.

Author verification passes 740 Rust tests, 134 compiler/data tests, 95 release
Python tests, consistency, rustfmt and Clippy with warnings denied. One hundred
legal Standard games complete naturally: 68,396 decisions, 2,025 staged payments,
15,493 independent public checks and 26,470 replay boundaries. Every new card,
reduction rule on a paid spell and new spell/trigger body category is sampled.
See [spell-cost-reductions.md](spell-cost-reductions.md) for complete provenance,
rules, workload policy and performance limits. The owned locked total adds 32
logical bytes; initial state is 19,680 bytes. Observation 41, ABI 1 and protocol
2 remain unchanged.

Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 1,063/5,164 Standard identities, six regression definitions, 48 token
factories and one emblem. Full Standard/native RL/search acceptance remains
incomplete, with 4,101 unsupported identities; the full goal remains active.
