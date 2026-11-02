# Aetherdrift implementation

The frozen DFT pool contains 276 Oracle identities (553 cosmetic printings),
including basic lands and reprints. `data/sets/dft.json` authenticates membership
against the pinned XMage table and Oracle facts. The starting branch admits 119
members and gates 157; admission is not full rules acceptance.

The implementation uses one set branch and one PR. Engine changes must be
completed, tested and independently reviewed before card authorship. Subsequent
card definitions, provenance and scenario tests belong in separate commits.
No set-completion claim is made until every printed obligation, complete set-only
games, deterministic replay and owned search behavior pass acceptance.

## Engine checkpoint

The branch starts at the merged engine/per-set/test build split (#565). It uses
`crates/engine`, generated per-set crates and separate integration test targets.
The direct-source dependency audit covers all 276 members, including the already
admitted cards; reusable helper review limits are explicit in its ledgers. Auditors partition monochrome W/U, monochrome B/R, green plus
multicolor, and colorless/lands; they do not author cards during this phase.

The audit must identify costs, timing, targets, modes, entry choices, triggers,
continuous effects, replacements, token/copy behavior, source departure and
controller changes. All identified dependencies were recorded in issue #566 before engine work began.
Focused assertions and affected shared contracts must pass before the engine
baseline is frozen and the authoring cache warmed.

## Acceptance still required

- Resolve every audited engine dependency and obtain independent reviews.
- Implement all 157 gated identities and repair any audited admitted behavior.
- Authenticate per-card Oracle and complete XMage/reusable-source provenance.
- Prove each printed obligation with independent outcomes and shared contracts.
- Regenerate linkage; run the complete release validator with reusable outputs.
- Exercise complete DFT-only legal games across all colors and mechanics, with
  deterministic replay and independent raw/index branches and restore.
- Obtain two independent nonauthor reviews of the final published PR head.
- Update the README tracker with verified final counts and acceptance evidence.

All Cargo compilation uses `scripts/resource_guard.py`. Reports and compiler
outputs use a persistent cache outside the checkout. Sources remain fixed while
a consuming process runs, and required libraries remain at their recorded paths.

The first engine checkpoint was accepted at `819fa96ec`. Exact card wiring then
identified seven missed typed-API details; the engine phase is reopened, ordinary
card authoring is parked, and all corrections are being assembled for one shared
build. The historical checkpoint remains preserved below.

At that checkpoint, All 3329 cases pass: 294 focused
engine scenarios, 3030 ordinary legacy scenarios, four complete-game cases and the
strict promoted trace baseline. Both formerly slow BIG pairings now finish below
the unchanged 45-second limit. Two independent nonauthor reviews approve this
baseline. Exact inputs, reports and workload timings are recorded in
`docs/aetherdrift-engine-freeze.json` and the historical progress ledger.

The final engine/set/test build took 152 seconds. After the audited schema-199
trace promotion, linkage regeneration and strict trace verification, a 0.365-second
warm Cargo check reused the engine and every verified test binary. Card authoring
must retain this target, profile and feature set, with card definitions/provenance
and card scenarios in separate commits. A missed shared dependency reopens the
engine phase before dependent cards continue.

No new DFT card was admitted during engine work. The 157 gated identities and full
rules verification of the inherited 119 now form the card phase; full DFT-only
games, release acceptance and the single completed-set PR remain outstanding.
