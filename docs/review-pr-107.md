# Independent review: restricted counterspell targets

PR: https://github.com/altf4/rustmage/pull/107
Implementation reviewed: 4bb97eed8cfedcfe09082e813e2f816b38c10747.
Base: main 9ce09fe6e0c456336cac1378141457a1953be484.

Two independent read-only subagents reviewed the exact implementation revision.
Neither edited files, mutated Git or posted externally. Both found no actionable
issues before merge.

The rules reviewer checked all seven complete bodies, color/type unions, printed
mana value and hybrid payment, target identity/revalidation, Spectral payment and
Greatshark's mandatory trigger/source departure. Oracle bodies and pinned Java
hashes match. Independently passed 713 Rust tests, 129 compiler/data tests,
catalogue consistency and all three new public Python methods.

The library reviewer checked the shared stack matcher, source/incarnation checks,
resolution revalidation, tax continuations, snapshots/privacy, all seven admissions
and workload assertions. Provenance matches. Independently passed 713 Rust tests,
129 compiler/data tests, 90 release Python tests and consistency. Twenty Standard
games exercised all seven cards and both tax outcomes, with 3,540 replay boundaries.
An unpinned benchmark measured roughly 396k decisions/s; initial logical state
remains 19,648 bytes. These timings establish no speed change.

Author verification passes the full suites above, rustfmt and Clippy with warnings
denied. One hundred legal Standard games complete: 66,082 decisions, 2,395
independent public checks, 16,556 clone/replay boundaries and 20,468 maximum
initialized state bytes. All seven bodies and both tax outcomes are sampled.
[counter-spell-filters.md](counter-spell-filters.md) documents rules, pinned
provenance, workload policy and performance limitations.

Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 1,040/5,164 Standard identities, six regression definitions, 48 token
factories and one emblem factory. Observation 41, ABI 1 and control protocol 2
remain unchanged. X costs, copies, changing stack characteristics and residual
unsupported dependencies remain gated. Full Standard and broader native RL/search
acceptance remain incomplete, with 4,124 unsupported identities; the full goal
remains active.
