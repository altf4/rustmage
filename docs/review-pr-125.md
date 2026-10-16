# Independent review: hybrid activated mana costs

PR: https://github.com/altf4/rustmage/pull/125
Implementation reviewed: f21937d58938ea270ebf8867214820a4ebe94375.
Base: main ae5f2f8b684385d5a53f2ce897c69ee330179cbc.

Two independent read-only subagents reviewed the exact implementation. Both
report no actionable findings. Neither edited files, mutated Git or posted
externally.

The rules reviewer checked hybrid/two-brid declaration and commitment, physical
allocation deduplication, colored/generic costs, funding/source restrictions,
one printed ability identity, all five activated bodies across the three full
admissions and compiler gates. All three complete Oracle bodies and pinned Java
hashes match. Independently passed 810 Rust, 150 compiler/data, the five-case
release Python activation replay test and catalogue consistency.

The library reviewer checked immutable alternatives, deduplication/determinism,
virtual mana-source obligations and recursion exclusion, one-index use masks,
staged cost/target/nonmana payment, snapshots/privacy/schema 45 and whole-body
gates. Independently passed 810 Rust, 150 compiler/data and 107 release Python
tests, catalogue/diff checks and all three source hashes. The hundred-game native
workload exactly reproduced 61,512 decisions, 1,063 activation payments/body
checks, 29,480 clone/restore/replay boundaries and 21,016 maximum initialized
bytes. All three entries, all five ability bodies, seven hybrid payment routes
and two fixed Trostani routes were exercised.

Author checks also pass formatting and Clippy with warnings denied. See
[hybrid-activation-costs.md](hybrid-activation-costs.md) for provenance, pinned
rules, validation, workload policy, future-cost IR boundaries and performance
limitations. Only this review record and progress bookkeeping follow the
reviewed source. Coverage is 1,105/5,164 Standard identities, with 4,059
unsupported. Observation 45 adds activation-cost choices; ABI 1, protocol 2 and
initial logical state 19,752 bytes remain unchanged. Full Standard and native
RL/search acceptance remain incomplete and active.
