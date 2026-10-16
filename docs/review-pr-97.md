# Independent review: loyalty activations and persistent emblems

PR: https://github.com/altf4/rustmage/pull/97
Implementation reviewed: cdb258fdb43e8dd9a91badc89bf93258354af6f8.
Base: main e6ec47b45244e4ce3b3c33c75829d862dad675cc.

Two independent read-only subagents reviewed the implementation before merge.
Neither edited files, mutated Git or posted externally.

The rules reviewer checked fixed loyalty costs/timing/shared limits, payment and
source obligations, mana classification, Dellian's complete body, command-zone
emblem lifecycle, captured life events and compiler gates. Oracle text and both
pinned Java source hashes match. No actionable findings. Independently passed
632 Rust tests, 110 compiler/data tests, catalogue consistency and both public
Python ability/emblem lifecycle regressions.

The library reviewer checked signed costs, shared activation limits, funding/source
obligations, atomic rollback, emblem classification/lifetime, captured life events,
schema 40 and native/Python replay. Both pinned source hashes match. No actionable
findings. Independently passed 632 Rust tests, 110 compiler/data tests, all 76 release
Python tests, catalogue consistency and 20 legal Standard games exercising all four
abilities, 55 planeswalker attacks, nine emblems and 38 emblem triggers.

Author validation passed those full suites plus rustfmt and Clippy with warnings
denied. One hundred legal Standard games completed naturally: 97,979 decisions,
3,952 independent public checks, 24,532 clone/replay boundaries, 21,180 maximum
initialized state bytes, all four abilities, 317 walker attacks, 60 emblems and
184 emblem triggers. [loyalty-activations.md](loyalty-activations.md) records rule
references, admission provenance, full public lifecycle evidence, exact numeric
and snapshot contracts, memory costs and bounded performance limitations.

Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 987/5,164 Standard identities, six regression definitions, 43 token
factories and one separate emblem factory. Issue #93 stays open for variable
loyalty costs, modifiers and remaining walker dependencies; other incomplete
bodies remain rejected. Full Standard and broader RL/search acceptance remain
incomplete, with 4,177 unsupported identities.
