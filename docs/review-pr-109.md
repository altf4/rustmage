# Independent review: exact two-brid spell costs

PR: https://github.com/altf4/rustmage/pull/109
Implementation reviewed: 7234a75e4a5ba87f38b5f0cc82dc75070a624028.
Base: main 7e0e73c8ebaa17ec9256e137b8e89860466ddaca.

Two independent read-only subagents reviewed the exact source revision. Neither
edited files, mutated Git or posted externally. Both found no actionable issues.

The rules reviewer checked exact two-brid equivalents, printed mana value,
committed funding/payment, deduplication, all three complete bodies and compiler
gates. Oracle bodies and pinned Java hashes match. Independently passed 719 Rust
tests, 131 compiler/data tests, consistency, both new public Python methods and
an additional 484-combination cost/mana-value audit.

The library reviewer checked varying-generic immutable ManaCost alternatives,
funding/commitment, prefunded deduplication, canonical printed values, numeric
limits and existing fixed/hybrid paths. All three admissions and provenance match.
Independently passed 719 Rust tests, 131 compiler/data tests, 92 release Python
tests and consistency. Twenty legal Standard games exercised all three bodies,
every generic component and every payment total, with 5,447 replay boundaries.

Author verification passed the full suites above, rustfmt and Clippy with warnings
denied. One hundred legal Standard games completed: 86,710 decisions, 602 committed
choices, 4,022 independent public checks, 30,593 clone/replay boundaries and 20,484
maximum initialized state bytes. All three bodies and all four generic/payment
totals were sampled. [two-brid-spells.md](two-brid-spells.md) records provenance,
rules, workload policy and performance limitations. Initial logical state remains
19,648 bytes; no mutable state, observation or ABI field was added.

Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 1,043/5,164 Standard identities, six regression definitions, 48 token
factories and one emblem factory. Observation 41, ABI 1 and control protocol 2
remain unchanged. Other two-brid bodies and mana-spent/Phyrexian/snow/X dependencies
remain gated. Full Standard and broader native RL/search acceptance remain
incomplete, with 4,121 unsupported identities; the full goal remains active.
