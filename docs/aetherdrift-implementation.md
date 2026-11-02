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

Focused engine evidence and build timings are retained in
`docs/aetherdrift-engine-progress.json`. The first32 selected cases passed on
148572fe; subsequent shared changes remain subject to revalidation and review.
