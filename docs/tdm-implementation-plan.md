# Tarkir: Dragonstorm implementation

Tracking issue: [#570](https://github.com/altf4/rustmage/issues/570).
Implementation branch: `codex/tdm-complete-set`.
Baseline: `c87b55079c540edde1c49cc725a1e1475bebf68b`.
Pinned XMage: `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

The frozen denominator is **276 identities**, deduplicating 432 printings.
`data/sets/tdm.json` retains 275 pinned Oracle identities and explicitly records
Cori-Steel Cutter as unresolved. The Standard snapshot excludes this banned card;
complete-set work must reconcile its facts and behavior without changing Standard
legality. The initial flags report 137 admitted candidates and 138 gated identities.
None of those flags independently certifies all printed obligations.

## Current phase

Complete whole-set source and dependency audit before changing behavior. Three
disjoint audit partitions cover all 275 resolved identities; the separate
[Cori-Steel Cutter audit](tdm-cori-steel-audit.json) covers the final identity.
The [complete dependency ledger](tdm-engine-audit.json) reconciles all 276 source
audits and records 81 dependency/proof groups, including duplicate group aliases.
Shared engine implementation is in progress. **49 of the81 audit/proof groups**, including aliases, have scoped implementation, focused evidence and two independent nonauthor reviews. Another22 groups have owner tests passing with reviews or review repairs pending. Counts are not weighted by complexity. Independent casting review identified an81st cross-cutting dependency: zero/free/reduced final totals must not open a mana-payment window or invent payment witnesses. Its primary-rule audit is in `tdm-mana-window-audit.json`; it is being fixed before card authorship. Full integrated engine acceptance remains open; card authorship stays gated until all dependencies pass and are reviewed.

The reviewed fixed Endure repair covers: use the exact permanent's
current controller, retain its last battlefield controller after departure, and
create a Spirit when counter placement is prohibited. Existing departure packets
already preserve that controller in `StackSourceView.facts.controller`; no new
public state is needed for this repair. Independent expected outcomes are:

| Risk | Required observable result |
| --- | --- |
| Global or attached counter prohibition | Create the printed Spirit; no impossible counter choice |
| Control changes after a trigger | The current controller chooses and owns the created Spirit |
| Departure after a control change | The last battlefield controller creates the Spirit |
| Departure followed by reentry | Never put counters on the new incarnation |
| Suspended choice | Only its actor receives the choice; illegal/wrong-actor actions roll back |
| Owned choice state | Raw/index children and restored snapshots resolve independently after parent disposal |

## Acceptance and resource contract

Use `crates/engine/` for shared rules, paired files under
`src/card_definitions/<set>/` for behavior, and separate integration test targets.
Generate linkage with `python3 scripts/link_catalogue.py`; never hand-edit outputs.
All Cargo commands run through `python3 scripts/resource_guard.py -- cargo ...`.
Use one stable mutable cache outside the checkout; retain logs and immutable
published libraries under the persistent cache. Never modify a consuming job's
inputs until that process actually terminates.

After the complete engine checkpoint, regenerate/warm and freeze its profile and
features. Author all remaining whole cards, validate their independent outcomes,
then exercise set-only legal complete games across all colors, mechanics and
interactions with replay and owned search checks. Update README only with final
verified counts. Freeze final executable inputs and run
`python3 scripts/validate.py --release --python smoke` before opening the single
completed-set PR; obtain two independent nonauthor final-head reviews.

## Measured status

Initial source-pin verification: 5,488 definitions / 2,512 source blobs verified;
generated-link consistency and authenticated frozen membership checks pass.
No new complete cards, full acceptance runs or simulation throughput measurements
have been produced. The latest validated assembled checkpoint passes376 selected
rules scenarios. Compilation took155.727s and bounded scenario execution1.152s;
these are rules-workload measurements, not simulation throughput. Newer cost,
combat and event checkpoints have separate passing owner reports and still need
integration acceptance.

Reviewed scoped checkpoints include fixed and dynamic Endure, sampled copiable
token bases and source LKI, optional attachment, named entry modes, counter-world
queries, entry counter payloads/history, colored protection and independent
Decayed instances, granted Harmonize/delve, copy/Suspend lifecycle, and resolving
outcomes/private-library continuations. Reviews reproduced and repaired Decayed
multiplicity, wide Harmonize/Bargain funding, sampled source-copy LKI and private
top-card pacing defects. Combat review identified a further CR111.5 nonpermanent
token-copy edge; its owner repair passes93 scenarios and awaits integration/review.

The dependency ledger links exact source hashes, reports and scoped reviews.
Remaining work includes Behold/free-origin casting, target/activation cost review,
combat/event integration and reviews, static restrictions, search/color sequences
and the other audited groups. Global observation-schema/trace migration, full
shared-engine acceptance, every card's complete behavior, set-only games and the
single final PR's acceptance/reviews remain open. The user goal stays active.
