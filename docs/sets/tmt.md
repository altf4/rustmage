# Magic: The Gathering | Teenage Mutant Ninja Turtles (TMT)

## Scope and baseline

Implement every physical TMT identity in the repository's frozen **2026-10-15
Standard tabletop/MTGO snapshot**. This work preserves that target.
Integration branch: `codex/tmt-complete-set`, base `1a795dcf`.
Branch created 2026-10-23 04:41:28 UTC (2026-10-22 local).
Tracking issue: https://github.com/altf4/rustmage/issues/464.
**The set is incomplete. No implementation PR has been opened.**

Pinned XMage revision: `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.
[Frozen membership](../../data/sets/tmt.json): **195 physical identities**,
320 pinned XMage printings, zero unresolved identities. Independent Scryfall
`set:tmt`, `unique=cards`, `include_extras=true` reconciliation matches all 195;
[evidence](tmt-membership-reconciliation.json). No member appears on the frozen
target's banned list. Commander/bonus-set products outside TMT are excluded.
Oracle facts remain pinned in `data/standard/cards.json`. The
[obligation ledger](tmt-obligations.json) accounts for every printed line and
canonical definition, including reprints; it is not a second membership source.

Initial admission: **71/195**, **124 gated**. Current admission: **97/195**,
**98 gated**, zero unresolved. The 71 inherited admission flags still require
source/rules audits. **26 new physical cards are locally tested candidates**;
zero have independent final published-head reviews or upstream merges.

## Active Sneak integration

Shared Sneak casting and thirteen whole-card consumers passed all nine release
validation phases on `f04dc2ff`: 3,295 library rules tests, 24 Sneak scenarios,
all native integration games, and seven Python binding smoke tests. Validation
held 11,852 inputs fixed and completed in 616.53 seconds under the 6 GiB guard
(3.5 GiB measured peak). Exact report and library hashes are recorded in
[tmt-sneak-validation.json](tmt-sneak-validation.json); producing-host paths stay
in its referenced cache manifest. The compact `CastInfo` remains 40 bytes.

This batch covers thirteen cards: Donatello's Technique; Foot Ninjas; Leonardo,
Big Brother; Oroku Saki, Shredder Rising; Jennika's Technique; Splinter's
Technique; New Generation's Technique; Leonardo's Technique; Leonardo, Cutting
Edge; Splinter, Hamato Yoshi; Raphael, the Nightwatcher; Shredder, Unrelenting;
and Karai's Technique. Tests exercise actual paid actions, authorized graveyard,
exile and library origins, exact attacker reservations and defenders, hybrid
costs, private mandatory search, partial invalidation, combat, source departure,
and raw/indexed branching and restore.

Freeform regression measured 191,326 decisions/s and 383.94 games/s on the
previously declared smoke workload. This is not TMT-specific simulation evidence.
The next active batch adds exact-incarnation Sneak history, Leonardo, Leader in
Blue; Karai, Future of the Foot; Turncoat Kunoichi; and Donatello, Gadget Master.

## Card obligations and local batches

All three batches were authored locally by the integrator on base `1a795dcf`,
with complete Oracle bodies, complete XMage classes and relevant reusable
helpers read before authoring. Paired JSON records pin the read bytes and retain
historical typed-lift records. The ledger links each card to its source record,
printed obligations, rules scenarios and [validation evidence](tmt-validation.json).

| Local batch | Newly tested physical cards | Shared behavior |
| --- | --- | --- |
| 1 | Foot Mystic; Insectoid Exterminator; Michelangelo, Game Master; Lord Dregg, Insect Invader; Putrid Pals; Featherbrained Filcher; Zoo Escapees | Disappear/Revolt history; conditional triggers and pre-entry counters; departing-source tokens and token payments |
| 2 | Action News Crew; Armaggon, Future Shark; High-Flying Ace | Instant-speed Channel; Flash; zero-to-three targets and partial invalidation; resolution-time target restrictions |
| 3 | Mouser Foundry; Sewer-veillance Cam; Ragamuffin Raptor | Entry/departure abilities; sacrifice costs and source LKI; optional tap/untap; graveyard creature/Food union targeting |

Supporting definitions append black 1/1 Ninja and black 1/1 flying Insect Warrior
tokens without renumbering existing identities. Existing Food, Mutagen and Robot
tokens have audited source pins and paid interaction evidence.

Disappear history is public, per player, and recorded only on an actual committed
battlefield departure under its pre-move controller. Tokens, lands, bounce and
exile count; nonbattlefield movements and control changes do not. Both players'
histories reset at each turn start. The typed guard serves intervening-if
triggers and pre-entry counter replacements, including Putrid Pals through the
normal counter-replacement pipeline. Snapshots and branches own the history;
observation schema 169 exposes it to both viewers.

Cam's typed tap/untap instruction uses the target's state when its optional
effect resolves and follows normal tapping/untapping replacement paths,
including stun counters. The engine contains no card-name dispatch or English
interpretation for these cards.

## Validation, reviews and game coverage

Source revision **`f12b667abdda9abb0212130a5ec15547c1ff5baa`** passed
`python3 scripts/validate.py --release --python smoke`: **all nine stages passed
in 610.68 seconds**, with **11,849 tracked inputs unchanged**. This includes
format/link checks, **54 Python source tests** (three skipped), Clippy,
**3,271 library rules tests** (one ignored), **20 TMT scenarios**, nine catalogue
checks, 22 Foundations game suites, existing ECL acceptance and protocol suites,
production release compilation, the regression benchmark and **seven Python
binding smoke tests**. All native targets total 3,307 passing tests.

TMT scenarios exercise actual paid legal actions, private scry choices, both
seats, rejected-action atomicity, exact incarnations, queued source departure,
partial target invalidation, owned raw/indexed branches, parent disposal and
restore. The existing full native suite also tests capacity-failure rollback of
simultaneous departures. The repaired Cam scenario correctly passes priority
before activating on an opponent's turn. Earlier failed test attempts remain in
cache logs; the final complete validator passed.

Resource guard: one Cargo job, one test thread, shared **6 GiB cap**;
**3,709,431,808-byte peak**, zero OOM events. Validator cleaned 818,245,632
allocated bytes; completed focused builds cleaned another 97,030,144 bytes.
Every required shared library remains at its exact producing-host path/hash in
[tmt-validation.json](tmt-validation.json). Full report:
`<cache>/rustmage-validation-iywsk6p6/report/result.json`.

**TMT full-game acceptance, journal replay across set-specific games, entire-pool
interaction coverage and final independent reviews remain pending.** No local
reviews are claimed. Publish one complete-set PR only after all implementation
and set-level validation gates pass; then obtain two independent nonauthor
reviews of the exact final published head.

## Measurements

Thirteen new locally verified physical cards from branch creation at
04:41:28 UTC to validation completion at 05:13:19.924082 UTC:
**24.48 cards/hour over 1,911.92 seconds**. This includes source intake,
authoring and release verification; inherited admissions are excluded.

The separate existing **Freeform smoke** workload completed 100 natural games,
49,832 decisions in 0.278615334 seconds: **178,855.91 decisions/second**,
358.92 games/second, 270.45 ns clone and 126.34 ns restore. Both decks contain
24 Mountain, 20 Lightning Bolt, 12 Goblin Piker and four Hill Giant; seeds 0–99,
`policy::smoke_index`, starting player P0. Intel Core Ultra 5 335, rustc 1.98.0,
release opt-level 3/thin LTO, bounded single test thread. Simulation includes
initialization, legal actions, policy, apply, observation construction and
terminal checks; it excludes compilation, model/tensor/JSON work and the
separately timed 100,000 clone/restore iterations.
**This is regression evidence, not TMT simulation throughput. TMT-specific
simulation throughput has not yet been measured.**

## Resume here

Current source revision `f12b667a` on `codex/tmt-complete-set`; subsequent
checkpoint changes affect only this ledger, obligation states and validation
records. No active build/test processes remain. Completed logs are under
`<cache>/rustmage-tmt-*.log`; reports and exact retained library paths/hashes are
in the validation JSON. Focused disposable outputs have been cleaned. A new
host rebuilds missing libraries rather than inventing old producing-host paths.

**Next action: implement and test the four conditional/copy Sneak consumers
listed above.** The shared Sneak kernel and thirteen of 26 physical consumers
are now verified. Remaining consumers and the 71 inherited cards still need
complete source, rules and acceptance evidence.
Remaining Disappear consumers are Krang & Shredder (linked exile/free casting),
Pizza Face, Gastromancer (conditional animation), Rat King, Verminister
(three-Rat payment and same-name reanimation), and West Wind Avatar
(optional token-or-land sacrifice). Other outstanding mechanics include Class
levels, special casting permissions, linked choices, equipment and copy effects.
These are engineering work, not known external access/source blockers.

Continue auditing the 71 inherited cards and implementing the remaining 98
gated cards. Keep the 195 denominator fixed. Integrate locally, regenerate with
`python3 scripts/link_catalogue.py`, authenticate with
`python3 -m scripts.set_status TMT --check`, and run the complete release
validator on fixed inputs. Every Cargo command must use
`python3 scripts/resource_guard.py -- cargo ...`. Do not publish a partial PR.

Resume with: “Use rustmage-implement-set to continue TMT from docs/sets/tmt.md.”
