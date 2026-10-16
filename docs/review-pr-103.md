# Independent review: distinct quantities and scaled effects

PR: https://github.com/altf4/rustmage/pull/103
Implementation reviewed: 2494741566112451a508be8d799aae743ba198c2.
Base: main e16ead3a6f19bc5c224a3e01e2b85a4d79217da9.

Two independent read-only subagents reviewed the exact implementation revision.
Neither edited files, mutated Git or posted externally. Both found no actionable
issues before merge.

The rules reviewer checked all eight admissions, color/card-type unions, live
versus frozen stats, drain semantics, draw/decking, Peer's private discard-before-
count sequence and compiler gates. All Oracle bodies and pinned Java hashes match.
Independently passed 679 Rust tests, 121 compiler/data tests, catalogue consistency
and both new public Python Vivid/Peer replay tests (three card fixtures).

The library reviewer checked distinct unions, scaled effects, numeric rollback,
private discard continuations, live/frozen stats, all eight admissions and public
workload checks. All provenance hashes match. Independently passed 679 Rust tests,
121 compiler/data tests, all 83 release Python tests, catalogue consistency and
twenty native games covering all eight bodies with 2,954 clone/replay boundaries.
An independent benchmark measured roughly 394k native decisions/s; initial logical
state remains 19,648 bytes. These unpinned timings do not establish a speed change.

Author verification passed the full suites above, rustfmt and Clippy with warnings
denied. One hundred legal Standard games completed naturally: 58,753 decisions,
1,388 independently checked public facts across 628 quantity effects and 14,728
clone/replay boundaries, with 21,820 maximum initialized state bytes. All eight
admissions' quantity behaviors were sampled. [distinct-amounts.md](distinct-amounts.md)
records primary rule references, pinned provenance, focused rules/binding tests,
numeric and snapshot contracts, workload policy and performance limitations.

Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 1,014/5,164 Standard identities, six regression definitions, 48 token
factories and one emblem factory. Observation 41, ABI 1 and control protocol 2
are unchanged. Other Vivid dependencies and incomplete bodies remain rejected.
Full Standard and broader native RL/search acceptance remain incomplete, with
4,150 unsupported identities; the full goal remains active.
