# Magic: The Gathering | Teenage Mutant Ninja Turtles (TMT)

## Scope and baseline

Implement all TMT physical identities in the repository's frozen **2026-10-15
Standard tabletop/MTGO snapshot**. This work preserves that target; it does not
claim a live-format update. Implementation branch created 2026-10-23 04:41:28 UTC (2026-10-22 local).
Integration branch `codex/tmt-complete-set`, base `1a795dcf`.
Tracking issue: https://github.com/altf4/rustmage/issues/464.
No implementation PR: publish exactly one only after complete-set acceptance.

Pinned XMage revision: `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.
[Frozen membership](../../data/sets/tmt.json): **195 physical identities**,
320 XMage printings, zero unresolved identities. Independent Scryfall
`set:tmt`, `unique=cards`, `include_extras=true` reconciliation matches all 195
identities, including reprints; [evidence](tmt-membership-reconciliation.json).
No TMT member appears on the frozen target's banned list. Commander/bonus-set
cards outside TMT are not added to Standard. Official product/card gallery:
https://magic.wizards.com/en/products/teenage-mutant-ninja-turtles/card-image-gallery.
Oracle facts remain pinned in `data/standard/cards.json`.

Initial admission: **71/195**, **124 gated**. Those 71 are inherited admission
flags and still require full source/rules audits. The
[obligation ledger](tmt-obligations.json) accounts for every printed line and
canonical definition. It is not a duplicate membership source.

## Local batches and shared mechanics

First local batch owns seven physical card pairs: Foot Mystic, Insectoid
Exterminator, Michelangelo, Game Master, Lord Dregg, Insect Invader, Putrid Pals,
Featherbrained Filcher and Zoo Escapees. Supporting definitions append the
missing black 1/1 Ninja and black 1/1 flying Insect Warrior tokens without
renumbering existing identities. Existing Food and Mutagen token definitions
retain their historical typed-lift records and acquire source pins.

Shared mechanic: public per-player Disappear/Revolt departure history, recorded
from the actual committed departure's pre-move controller, including tokens,
lands, bounce and exile; nonbattlefield movements and control changes do not
qualify. History resets for both players at each turn start. The typed guard
serves intervening-if triggers and pre-entry counter replacements. History is
owned in snapshots/branches and exposed to both observations. The shared
observation schema version advances for the additional public player field.

Full Oracle bodies, complete XMage classes and relevant reusable helpers were
read; paired JSON records pin the read bytes. Putrid Pals uses the shared
pre-entry replacement/counter-replacement pipeline, rather than a trigger.

## Validation, reviews and measurements

Initial source/provenance suite: `python3 -m unittest scripts.test_direct_cards
scripts.test_set_status scripts.test_source_paths`: **14 tests passed**.
Resource-guard preflight passed with enforced systemd cgroup, 6 GiB shared cap,
one Cargo job and one test thread; no allocation/OOM probe.

First batch: **10 optimized native scenarios passed in 0.07 seconds**, covering
seven newly verified physical cards. Build/test resource report:
`<cache>/rustmage-build-reports/ed4718ecfa4446f48a28713983d9b7b0.json`
(243.59 seconds, 3,228,803,072-byte peak, zero OOM events).

Second batch adds Action News Crew, Armaggon, Future Shark and High-Flying Ace.
Third adds Mouser Foundry, Sewer-veillance Cam and Ragamuffin Raptor. These six
remain candidates until the combined focused suite passes. Current admission
flags: **84/195**, comprising 71 inherited plus 13 local definitions; **111 gated**.
Cam uses a typed resolution-time tap/untap instruction through the normal
tapping and untapping replacement paths, with an optional decline.

The combined 20-scenario run passed 19 scenarios; the last Cam scenario tried
to activate while the opponent held priority after a Flash spell. The test now
passes priority through a legal action before activation. The full release
validator will verify that repair alongside all existing rules/game suites.
Implementation code did not require a repair for that failure.

Full release validation with Python smoke checks is pending on the local
integration revision; keep all inputs fixed until its process terminates.
No new whole cards count as verified until those scenarios pass. No local or
published-head independent reviews yet. No TMT full-game acceptance workload or
TMT simulation throughput measurement yet; inherited ECL/Freeform measurements
do not establish TMT performance. Implementation throughput pending completion
of the first verified batch.

## Resume here

Current branch `codex/tmt-complete-set`; local uncommitted work comprises the
manifest/reconciliation/obligation ledger, first batch, supporting token metadata,
shared departure history, schema version and `src/engine_tmt_disappear_tests.rs`.
Do not discard prior-set work inherited at the base revision.

Next: finish the focused `tmt_` rules suite, repair failures, then run the complete
release validator with Python smoke checks on fixed inputs.
Keep Sneak and the remaining Disappear consumers visible as engine work, not
source blockers. Complete all 124 originally gated cards plus inherited audits,
then full-set games, release validation and the final two independent reviews.

Current build cache on producing host: `<cache>/rustmage-tmt-first-batch`;
focused log `<cache>/rustmage-tmt-first-batch.log`. Inspect any active session
before editing its inputs. Every Cargo command must use
`python3 scripts/resource_guard.py -- cargo ...`. Read the updated checkpoint
before resuming; a historical session ID is not portable.
