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
audits and records 80 dependency/proof groups, including duplicate group aliases.
Shared engine implementation may now begin; all groups remain open. Do not begin card authorship until all shared dependencies have
focused rules/contract evidence and independent reviews.

The first prepared repair covers existing fixed Endure: use the exact permanent's
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
have been produced yet. The [Endure evidence](tdm-endure-evidence.json) records
six passing groups after the independent-review placing-player repair, with fresh
committed-source reviews pending. The [counter-world evidence](tdm-counter-world-evidence.json)
records seven passing owner scenarios and the integrated seven-group root pass.
The counter rule scope received two independent nonauthor source reviews (root
and the entry-mode owner). The complete shared checkpoint and set acceptance
remain pending. The user goal remains active.
