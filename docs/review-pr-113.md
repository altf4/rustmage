# Independent review: count-scaled spell reductions and affinity

PR: https://github.com/altf4/rustmage/pull/113
Implementation reviewed: fd79160d7991c4d143186e2cd93b6a0f789b9e6e.
Base: main c0acde414d26ebfd07d8ecb4b5c41de8bd489006.

Two independent read-only subagents reviewed the exact revision. Neither edited
files, mutated Git or posted externally. Both report no actionable findings.

The rules reviewer checked every complete admission, affinity stacking,
ownership, zone/card/type/color quantities, locked costs, generic floors and
colored requirements, composed bodies and compiler gates. All fourteen Oracle
bodies and pinned Java hashes match. Independently passed 752 Rust tests, 136
compiler/data tests, catalogue consistency and both public Python affinity/private
surveil regressions.

The library reviewer checked shared count evaluation/funding/validation,
saturation, source grants/indexing, locked totals and additional costs,
snapshots/privacy and whole-body admission gates. Independently passed 752 Rust
tests, 136 compiler/data tests, 97 fresh-release Python tests, catalogue/diff
checks and all fourteen source hashes. Both twenty- and hundred-game legal
Standard workloads passed. The hundred-game run exactly reproduced author
results: 68,986 decisions, 1,560 staged payments, 10,262 independent public checks,
25,679 clone/replay boundaries and 21,504 maximum initialized state bytes. All
fourteen cards and cost rules on paid spells were sampled.

Author checks also pass rustfmt and Clippy with warnings denied. See
[scaled-spell-reductions.md](scaled-spell-reductions.md) for provenance, rules,
focused native/Python evidence, workload policy and performance limits.
No mutable state or public schema/ABI field was added; initial logical state
remains 19,680 bytes. Observation 41, ABI 1 and protocol 2 are unchanged.

Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 1,077/5,164 Standard identities, six regression definitions, 49 token
factories and one emblem. Full Standard/native RL/search acceptance remains
incomplete, with 4,087 unsupported identities; the full goal remains active.
