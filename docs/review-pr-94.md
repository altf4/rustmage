# Independent review: simultaneous battlefield groups

PR: https://github.com/altf4/rustmage/pull/94
Implementation reviewed: 4649870cfa0f23728eb439d8d35f88dd8b1a061b.
Base: main c45d7d3b30d7626a66a601c045a613c083b73173.

Two independent read-only subagents reviewed the implementation before merge.
Neither edited files, mutated Git or posted externally.

The rules reviewer checked all 14 whole-card admissions, frozen affected sets,
simultaneous death/LKI handling, exact result counts across linked returns and
entry choices, source-aware damage batching, deferred SBAs/failed draws and exact
compiler gates. All recorded Oracle bodies and local XMage source hashes match.
No actionable findings. Independently passed 596 Rust tests, 104 compiler/data
tests, catalogue consistency and both public Python Fumigate/Tectonic regressions.

The library reviewer checked group membership, immutable rules, owned result
counts/suspended continuations, rollback/replay, public projections, source damage
history and all 14 admissions. Source hashes match the pinned revision. No
actionable findings. Independently passed 596 Rust tests, 104 compiler/data tests,
all 71 release Python tests, catalogue consistency and 20 supported Standard games
with 324 independent batch checks and 3,484 clone/replay boundaries.

Author validation passed the same full Rust/compiler/Python suites, catalogue
consistency, rustfmt and Clippy with warnings denied. One hundred supported
Standard games completed naturally: 85,507 decisions, 1,654 independently checked
public batch facts, 18,872 clone/replay boundaries, and 19,980 maximum initialized
state bytes. [battlefield-groups.md](battlefield-groups.md) records the complete
admissions, public/synthetic distinctions, source metadata, result-count contract,
bounded smoke measurement and remaining constraints.

Only this review record and progress bookkeeping follow the reviewed source.
Coverage is 981/5,164 pinned Standard identities, six regression definitions and
43 token factories. Planeswalker damage remains gated pending #93; no creature-
only approximation admits those bodies. Full Standard rules and broader RL/search
acceptance remain incomplete, with 4,183 unsupported identities.
