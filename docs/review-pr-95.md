# Independent review: loyalty counters and planeswalker damage

PR: https://github.com/altf4/rustmage/pull/95
Implementation reviewed: 8eff4db46e944bddc9ef158c26bb212c9080488e.
Base: main 33161d7a7a2fd935fc4dae938b6c4d265bb86156.

Two independent read-only subagents reviewed the implementation before merge.
Neither edited files, mutated Git or posted externally.

The rules reviewer checked loyalty entry/reset, damage and zero-loyalty SBAs,
lifelink/deathtouch/history, counter doubling/removal/funding, all five complete
admissions and compiler gates. Oracle bodies and pinned XMage hashes match.
No actionable findings. Independently passed 609 Rust tests, 106 compiler/data
tests, catalogue consistency and both public Python Dragonback/Broadside tests.

The library reviewer checked loyalty in costs, funding, numeric bounds, rollback,
snapshots, JSON/schema 38, damage/SBAs, metadata and all five admissions. All pinned
Java hashes match. No actionable findings. Independently passed 609 Rust tests,
106 compiler/data tests, all 73 release Python tests, catalogue consistency and
20 native games with 672 public checks and 6,284 clone/replay boundaries.

Author validation passed the same full Rust/compiler/Python suites, catalogue
consistency, rustfmt and Clippy with warnings denied. One hundred supported
Standard games completed naturally: 145,800 decisions, 3,746 independent public
checks, 35,870 clone/replay boundaries and 21,276 maximum initialized state bytes.
[loyalty-damage.md](loyalty-damage.md) records scope, synthetic future-walker versus
public fixtures, source metadata, numeric contracts and explicit memory cost.

Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 986/5,164 pinned Standard identities, six regression definitions and
43 token factories. Issue #93 remains open for loyalty activations, planeswalker
combat and whole walker bodies. Unsupported walkers remain rejected in production
decks. Full Standard and broader RL/search acceptance remain incomplete, with
4,178 unsupported identities.
