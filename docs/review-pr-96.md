# Independent review: planeswalker combat

PR: https://github.com/altf4/rustmage/pull/96
Implementation reviewed: 42b5317bff4370c75609020c945bd62fd38319f7.
Base: main 98012ec8d5aed8b0c605be96a886f7f925d88771.

Two independent read-only subagents reviewed the implementation before merge.
Neither edited files, mutated Git or posted externally.

The rules reviewer checked defender legality/privacy, incarnation binding and
departure, retained combat membership, first/double strike, ordinary trample
routing, damage continuations and loyalty-counter last-known information.
No actionable findings; behavior agrees with the pinned combat rules within the
documented scope. Independently passed 620 Rust tests, catalogue consistency
and the public Python attack-declaration/replay regression.

The library reviewer checked defender validation, serialization/privacy, damage
routing, departure/reentry, snapshots, actual C ABI replay, memory accounting and
loyalty last-known information. No actionable findings. Independently passed 620
Rust tests, all 74 release Python tests and 20 native games with 672 public checks
and 6,284 clone/replay boundaries. A local release benchmark sample produced
396,011 decisions/second, clone 89.25 ns, restore 86.44 ns and 19,552 initial logical
bytes. It did not reproduce the author's slower snapshot sample. Neither sample
proves a stable performance change; the source documentation retains that caveat.

Author validation passed 620 Rust tests, 74 release Python tests, catalogue
consistency, rustfmt and Clippy with warnings denied. One hundred supported
Standard games completed naturally with 145,800 decisions, 3,746 independent public
checks, 35,870 clone/replay boundaries and 21,516 maximum initialized state bytes.
[planeswalker-combat.md](planeswalker-combat.md) records rules, exact scope, privacy,
owned state, isolated future-walker fixtures and memory/performance limitations.

Only documentation and progress bookkeeping follow the reviewed source.
Coverage remains 986/5,164 Standard identities, six regression definitions and
43 token factories. No walker body is admitted. Issue #93 remains open for legal
loyalty activations and whole bodies; card-specific restrictions, control/type
changes, phasing, battles and trample over planeswalkers retain their unsupported
gates. Full Standard and broader RL/search acceptance remain incomplete.
