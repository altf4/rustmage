# Independent review: scaled token batches

PR: https://github.com/altf4/rustmage/pull/99
Implementation reviewed: 713ffef18315e29449638791454f0f54be665f33.
Base: main 1ba20310d93cba017c1d7d1cdb8947e2e8e999f2.

Two independent read-only subagents reviewed the implementation before merge.
Neither edited files, mutated Git or posted externally. Both found no actionable
issues at the exact implementation revision above.

The rules reviewer checked all eleven admissions, resolution-time quantities,
count filters, life-gain totals, simultaneous entry observers, token capacity and
identity handling, and compiler gates. Oracle bodies, Java hashes and regression
subtype references match. Independently passed 648 Rust tests, 114 compiler/data
tests, catalogue consistency and all three new public Python quantity/private-scry
regressions.

The library reviewer checked quantity evaluation, arena/free-slot preflight,
allocation errors, life-gain totals, simultaneous observers, snapshots/privacy,
all eleven admissions and subtype repairs. All fourteen recorded source hashes
match. Independently passed 648 Rust tests, 114 compiler/data tests, all 79 release
Python tests, catalogue consistency and twenty native games with 111 batches and
2,811 replay boundaries.

Author validation passed those full suites plus rustfmt and Clippy with warnings
denied. One hundred legal Standard games completed naturally: 56,332 decisions,
7,521 independently checked public facts, 549 token batches and 14,116 clone/replay
boundaries, with 36,808 maximum initialized state bytes. All eleven admissions had
their token effects sampled. The workload bounds optional policy choices; it does
not alter engine legality. [scaled-tokens.md](scaled-tokens.md) records provenance,
focused rules tests, exact numeric and snapshot contracts, memory costs and the
limits of the unpinned performance measurements.

Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 998/5,164 Standard identities, six regression definitions, 48 token
factories and one separate emblem factory. Observation schema is 41; ABI remains
1 and control protocol remains 2. Other incomplete bodies remain rejected.
Full Standard and broader RL/search acceptance remain incomplete, with 4,166
unsupported identities.
