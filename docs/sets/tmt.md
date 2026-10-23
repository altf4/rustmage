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

Initial admission: **71/195**, **124 gated**. Current admission: **195/195**,
**0 gated**, zero unresolved. All 71 inherited admissions have source/rules
audits and combined release evidence. **124 newly authored physical cards have rules scenarios; 102 have integrated release validation**;
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
Subsequent local batches add exact-incarnation Sneak history and the consumers
recorded below.

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

Initial checkpoint source revision `f12b667a` on `codex/tmt-complete-set`; subsequent
checkpoint changes affect only this ledger, obligation states and validation
records. No active build/test processes remain. Completed logs are under
`<cache>/rustmage-tmt-*.log`; reports and exact retained library paths/hashes are
in the validation JSON. Focused disposable outputs have been cleaned. A new
host rebuilds missing libraries rather than inventing old producing-host paths.

**Next action: complete integrated release validation for the latest ten cards,
then implement Michelangelo, Improviser; Raphael’s Technique; and Leonardo,
Sewer Samurai.** The shared Sneak kernel has 23 of 26 physical consumers authored
and rules-tested. The 71 inherited cards still need complete source/rules audits.
Remaining Disappear consumers are Krang & Shredder (linked exile/free casting),
Pizza Face, Gastromancer (conditional animation), Rat King, Verminister
(three-Rat payment and same-name reanimation), and West Wind Avatar
(optional token-or-land sacrifice). Other outstanding mechanics include Class
levels, special casting permissions, linked choices, equipment and copy effects.
These are engineering work, not known external access/source blockers.

Continue auditing the 71 inherited cards and implementing the remaining 88
gated cards. Keep the 195 denominator fixed. Integrate locally, regenerate with
`python3 scripts/link_catalogue.py`, authenticate with
`python3 -m scripts.set_status TMT --check`, and run the complete release
validator on fixed inputs. Every Cargo command must use
`python3 scripts/resource_guard.py -- cargo ...`. Do not publish a partial PR.

Resume with: “Use rustmage-implement-set to continue TMT from docs/sets/tmt.md.”

## Active conditional Sneak batch

Six further physical cards pass the expanded 31-scenario optimized Sneak suite:
Leonardo, Leader in Blue; Karai, Future of the Foot; Turncoat Kunoichi; Donatello,
Gadget Master; Dark Leo & Shredder; Kitsune’s Technique. Sparse public exact-entry
history preserves departure LKI and prevents new incarnations from inheriting
payment. The suite covers normal/Sneak entry differences, timing of the
reanimation branch, actual artifact copies and copied ETB/leave abilities,
post-token Ninja thresholds, and rounded-up milling/life loss. Focused log:
`<cache>/rustmage-tmt-sneak-six-tests-second.log` (terminal success, 31 passed).
Integrated full release validation for these six remains pending. Next: The Last
Ronin’s Technique using the existing defender-selection and attacking-entry
planner, plus its missing white 1/1 Ninja Turtle Spirit token.

The Last Ronin’s Technique and its white Ninja Turtle Spirit token are now
implemented, with all-target Clippy and ten direct-authoring/portability checks
passing. Its new scenario covers normal/Sneak token entry, individually chosen
player/planeswalker defenders, actual combat, and persistence through the end
step. This seventh card is awaiting integrated rules/release validation together
with the preceding six. Admission flags are 104/195; the additional flag is not
yet a tested-card claim.

## Active selection and destruction integration

The complete optimized Sneak suite now passes **37 scenarios**, including the
preceding seven consumers plus Michelangelo’s Technique; Shark Shredder, Killer
Clone; and Shredder’s Technique. It covers private zero/one/two top-eight picks
with an aggregate six-mana budget, shuffled-bottom remainder, actual first-strike
and regular combat damage with graveyard ownership/control and captured defender,
and successful enchantment destruction/LKI with indestructibility, invalidated
targets and death-trigger ordering. Focused report log:
`<cache>/rustmage-tmt-sneak-destruction-tests.log` (terminal success).

The integrated release run at `9079cf00` passed all 3,303 library rules tests,
including Ronin’s attacking tokens, but stopped at the catalogue token-deck
rejection test. Its failure evidence remains in
`<cache>/rustmage-validation-6vq35vln/report/result.json`. The new Spirit factory’s
default deck limit was corrected to the existing token convention (`None`);
the catalogue regression must pass in the next integrated run. No failed run
is treated as accepted validation. The latest ten physical candidates await
that full run. Next Sneak consumers: Michelangelo, Improviser; Raphael’s
Technique; Leonardo, Sewer Samurai.

## Validated 107-card checkpoint

All nine release-validation phases pass on `881adc0d`, including the ten latest
physical consumers. There are **107/195 admission flags**, **88 gated**, zero
unresolved; **36 newly authored physical cards have integrated rules/release
validation**, and **71 inherited entries await source/rules audits**. No final
published-head review or set-only full-game acceptance is claimed.

The run passed 3,308 library rules tests (one ignored), all 37 Sneak scenarios,
the complete existing native integration suites, and seven Python smoke checks.
It held 11,855 inputs fixed for 617.06 seconds. The token catalogue regression
now passes. The guard retained the 6 GiB cap, one build job and one test thread;
peak memory was 3,828,125,696 bytes, with zero OOM/max events. Completed
disposable validation outputs were cleaned automatically (840,941,568 bytes);
required libraries remain at their exact paths/hashes in the cache manifest.
See [validation evidence](tmt-sneak-consumers-validation.json).

Freeform smoke measured 185,521 decisions/s and 372.29 games/s on the previously
declared workload. TMT simulation throughput is still unmeasured. The next
authored work is Michelangelo, Improviser’s private simultaneous hand placement,
then Raphael’s Technique and Leonardo, Sewer Samurai. Those three printed Sneak
consumers and the remaining set mechanics/audits are engineering work, with no
known external blocker. Keep the full set goal open and do not publish a partial
implementation PR.

## Active Improviser integration

Michelangelo, Improviser passes all-target Clippy and the expanded optimized
**40-scenario Sneak suite** (`<cache>/rustmage-tmt-sneak-hand-tests.log`, terminal
success). Its optional private hand choice selects at most one creature and one
land before a single simultaneous entry batch, with zero/one/two-card branches.
Land-first Rampaging Baloths proves the shared entry event; tests also cover
source departure, exhausted land-play allowance, empty/foreign hands, duplicate
roles, and untapped nonattacking enchantment-creature entry. Both seats, private
observations and owned raw/index branch/restore are exercised by paid actions.
Observation schema is 172. Full release validation for this card is pending.
Next printed Sneak consumers: Raphael’s Technique and Leonardo, Sewer Samurai.

## Active optional hand refresh

Raphael’s Technique passes all-target checks and the expanded optimized
**45-scenario Sneak suite** (`<cache>/rustmage-tmt-sneak-wheel-tests-third.log`,
terminal success). Both public APNAP answers precede a single whole-hand discard
batch, followed by APNAP draws. New owned optional state preserves choices in
both observers’ views and branch/restore. Scenarios cover all acceptance masks,
either active player for normal instant casting, empty hands, simultaneous
empty-library losses, discard replacements entering before draws resume, and
transactional rollback of the final answer and partial discard events. A live
draw prohibition prevents choosing the impossible seven-card refresh, tested
against Mornsong Aria. The pinned Java loop interleaves each accepter’s discard
and draw; the authored implementation follows Oracle and CR101.4/608.2e/121.2c.
Observation schema is 173. Integrated release validation for Improviser and
Raphael’s Technique is pending. Next: Leonardo, Sewer Samurai, including exact
graveyard permission choice, normal/Sneak casting, and intrinsic finality-counter
replacement on every permanent under CR122.1h. Keep the full 195-card goal open.

## Validated 109-card checkpoint

All nine release phases pass on `c08f76c6`: 3,316 library rules tests
(one ignored), all 45 Sneak scenarios and seven Python binding smoke checks.
Michelangelo, Improviser and Raphael’s Technique now have integrated release
validation. Admission is 109/195, with 38 newly authored locally verified
candidates, 71 inherited admissions awaiting audit and 86 gated cards.
See [portable evidence](tmt-sneak-refresh-validation.json) for report hashes,
retained libraries, guard limits and measured throughput. Set-only games,
entire-pool acceptance and independent reviews remain pending.

Current next action supersedes earlier resume notes: implement Leonardo, Sewer
Samurai’s explicit graveyard permission and finality counters. The release
process is terminal; inputs may now change. Keep the full-set goal open.

## Active Sewer Samurai integration

Leonardo, Sewer Samurai and shared finality counters pass the optimized
**53-scenario Sneak suite** (`<cache>/rustmage-tmt-sewer-tests-fourth.log`,
terminal success). All 26 printed Sneak consumers are authored and locally
rules-tested; Ninja Teen’s granted Sneak remains outstanding. Explicit static
graveyard permission actions preserve independent alternative costs and mark
finality only for Samurai’s route. Scenarios cover both seats, ownership,
turn/timing, power-or-toughness qualification with characteristic-defined stats,
source loss/ability loss/incarnations, countering, actual double-strike combat,
Muldrotha’s separate quota and Sneak/Warp routes, and compact 40-byte CastInfo.

Intrinsic finality replacement works on all permanents, survives ability loss,
competes with self-shuffling replacement once, and clears on bounce/reentry.
Paired provenance confirms agreement with pinned Java, CR122.1h and official
TMT rulings: XMage’s isDiesEvent helper covers every battlefield-to-graveyard
move. An earlier note incorrectly inferred a creature restriction from its name. Source/provenance and portability checks pass.
Observation schema is 174. Admission is 110/195, 85 gated; the 39th newly authored
card awaits integrated release validation. Next: the five Class cards, including
true Class level designations, gained abilities and Ninja Teen’s granted Sneak
with composed mandatory costs. No external blocker or final review is claimed.

## Validated 110-card checkpoint

All nine release phases pass on `780c67e7` in 623.29 seconds,
with 3,324 library rules tests (one ignored), 53 Sneak scenarios, existing native
full-game integrations and seven Python binding smoke checks. All 26 printed
Sneak consumers are now locally rules/release verified. Ninja Teen’s granted
Sneak remains outstanding. Admission is 110/195, 85 gated, zero unresolved;
39 newly authored physical candidates are locally verified and 71 inherited
admissions still await audits. No set-only acceptance or final review is claimed.

See [portable evidence](tmt-sneak-finality-validation.json) for report/library
hashes, guard measurements and actual elapsed implementation throughput.
The full validator is terminal and preserved all required shared libraries.
Next work is Class levels and the five Class cards, with source research and
scratch preparation complete for the shared kernel and Leader’s Talent.

## Active Class integration

Leader’s Talent, Cool but Rude and Does Machines pass all-target Clippy and the
optimized **87-scenario TMT suite**, including 14 Class scenarios, all 53 Sneak
scenarios and the initial 20 TMT scenarios. Terminal focused log:
`<cache>/rustmage-tmt-class-machines-tests.log`. Separate owned Class levels
retain exact-incarnation designations independently of counters, copies and
ability loss. Common frozen departure observers preserve simultaneous bounce
and exile/LKI; the new conditional trigger primitive applies to every zone.

Paid scenarios cover Class progression/timing/reset, attack targets, source
ability loss, any-counter departure qualification, spell-cast mass counters,
private optional discard/draw, required search/shuffle, random discard and
capacity rollback including RNG. Does Machines covers mill/draw/discard order,
zero/one/two artifact recovery with partial invalidation, persistent additive
Robot animation, equipment unattachment, actual Vehicle crewing without base
reset, and counter replacement. All steps exercise owned raw/index branching,
parent disposal, both observations and restore. Observation schema is 175.

Admission is 113/195 with 82 gated and zero unresolved. The three newest
physical candidates await integrated release validation: 42 newly authored
cards have focused rules evidence, 39 have full release validation, and 71
inherited flags still need audits. Next: Ninja Teen’s dynamic Sneak and composed
mandatory costs, and Party Dude’s cross-player Food creation and attack trigger.
No implementation PR, final review or set-only acceptance is claimed.

## Validated 113-card checkpoint

All nine release phases pass on `0faafd3d` in 623.89 seconds:
3,338 library rules tests (one ignored), all 87 TMT scenarios (14 Class,
53 Sneak, 20 initial), existing native full-game suites and seven Python
binding smoke checks. Admission remains 113/195, 82 gated, zero unresolved.
42 newly authored physical candidates are now locally rules/release verified;
71 inherited admission flags still need audits. No set-only acceptance or
independent final published-head review is claimed.

See [portable evidence](tmt-class-validation.json) for report hashes, exact
retained libraries, resource guard limits and measured elapsed throughput.
The first Class run passed 3,337 tests but found a missing expected category
in the exhaustive source-index regression; its failure report is preserved
in `<cache>/rustmage-validation-ixygovu8/report/result.json`. The corrected
index test still checks registration, departure and snapshot/restore for
every admitted definition and passes in the complete rerun.

Next: Party Dude's shared cross-player token event, then Ninja Teen's dynamic
Sneak/cost composition. The full 195-card goal stays open.

Finality source-note clarification after the 113-card producing revision: full
`ZoneChangeEvent.java` confirms that XMage’s `isDiesEvent()` is an all-permanent
battlefield-to-graveyard predicate. Samurai’s provenance now pins that helper
and corrects the earlier claimed discrepancy. No gameplay behavior changed.

## Active Party Dude integration

Party Dude and a generic staged cross-player token event pass all-target
Clippy and the optimized **94-scenario TMT suite** (21 Class, 53 Sneak,
20 initial), log `<cache>/rustmage-tmt-party-tests.log`, terminal success.
Token replacements for every affected player complete before a single shared
entry batch. Pending plans and copy recipes are fully owned and measured.
A copied Doorkeeper Thrull suppresses the simultaneously entering Ninja’s
entry trigger; no token enters between the two players’ replacement choices.
The last choice rolls back all plans, recipes and partial allocations on failure.

Other paid scenarios cover independent token doubling/ownership, opponent
artifact-token deaths versus own artifacts/finality exile, simultaneous Class
and artifact departure, zero/one attack targets, player-versus-planeswalker
defenders, live hand count after a response, source departure and expiration.
No new token identity or fake copy event is needed for ordinary Food creation.
Admission is 114/195, 81 gated, zero unresolved. The 43rd newly authored card
awaits full release validation; 42 have integrated release evidence and 71
inherited flags await audits. Next is Ninja Teen’s dynamic Sneak and composed
mandatory costs, then the remaining cards and inherited audits.

## Validated Party Dude checkpoint

All nine release phases pass on `a746f8fd` in 626.64 seconds.
3,345 library rules tests (one ignored), all 94 TMT scenarios (21 Class,
53 Sneak, 20 initial), existing native full-game suites and seven Python
binding checks pass. Admission is 114/195, 81 gated, zero unresolved.
43 newly authored cards now have integrated release evidence; 71 inherited
admission flags still require audits. Set-only acceptance and final reviews
remain pending. See [portable evidence](tmt-party-validation.json).

Next: Ninja Teen's dynamic Sneak permission and composed additional costs.

## Ninja Teen integration candidate

Ninja Teen completes the five-card Class family locally. All 23 focused Ninja
scenarios pass in `<cache>/rustmage-tmt-ninja-tests-locking.log`; the preceding
combined run passed all 116 then-existing TMT scenarios. The final integrated
release run is pending, including the now-admitted definition's paid normal cast.
Admission is 115/195, 80 gated, zero unresolved; 44 new physical candidates have
focused rules evidence, 43 have integrated release evidence, and 71 inherited
admission flags still need audits.

Level two supplies live +1/+0 and menace. Level three freezes granted Sneak costs
before the spell leaves the graveyard, preserves native equivalents, and separates
Ninja, Samurai finality and Muldrotha quota permissions. Sneak now composes its
return with printed sacrifice/behold/exile costs in either order, using real
post-return hand incarnations. Kicker, offspring, convoke, fixed-cost X=0 and
mana-instead-of-behold routes compose with the alternative cost. Mana abilities
can supply a witness before payment; no mana window opens after partial payment.
Cost determination remains fixed when a reducer enters during mana production.

The rare owned completion driver preserves private witnesses, replacement/entry
choices, exact references, mana provenance domains and transactional rollback.
Ordinary Sneak retains its borrowed funding path; Object and compact CastInfo do
not grow. Observation schema is 176. Rules scenarios cover both seats and owned
raw/index branching, parent disposal, observations, replay and restore.

A nested generic mana-completion proof originally tried to finish the deliberately
suspended Sneak projection. Its run was stopped with inputs frozen, with the log
preserved at `<cache>/rustmage-tmt-ninja-tests-expanded.log`. Dedicated projection
ownership now avoids that duplicate completion search. Missing-witness bounds
skip irrelevant tap-mana permutations while retaining object-changing/linked
return paths. All 23 scenarios, including the linked-return and rollback cases,
pass after that correction. No implementation PR or final review is claimed.

Next: full release validation, then the shared counter-LKI/animation consumers
(Donatello, Mutant Mechanic; Pizza Face; The Ooze; Mind Transfer Protocol), followed
by remaining gated cards, inherited audits and set-only acceptance.

The first admitted Ninja release run (`4f10e0cf`) passed 3,366 library tests but
found two animation fixtures attacking the freshly cast Class despite summoning
sickness. Its complete failure evidence remains in
`<cache>/rustmage-validation-zx31120f/report/result.json`. Both fixtures now let
the controller's next real turn begin before animating/attacking; gameplay
sickness enforcement is unchanged. The integrated release rerun is pending.

## Validated 115-card checkpoint

Ninja Teen and all shared cost composition pass all nine release phases on
`efe5d031` in 638.33 seconds. All 3,368 library tests (one
ignored), 117 TMT scenarios (23 Ninja, 21 other Class, 53 Sneak, 20 initial),
existing full-game suites and seven Python binding checks pass. All five
Classes now have integrated release evidence. Admission is 115/195, 80 gated,
zero unresolved; 44 new physical candidates are rules/release verified, while
71 inherited admission flags still need audits. Final independent reviews
and set-only acceptance remain pending.

See [portable evidence](tmt-ninja-validation.json) for report/library hashes,
resource limits, elapsed implementation throughput and the Freeform regression
measurement. Completed disposable validator outputs were cleaned; required
shared libraries remain at their exact producing paths and hashes.

Next: general departure-counter LKI and animation consumers (Donatello, Mutant
Mechanic; The Ooze; Pizza Face; Mind Transfer Protocol; Super Shredder).
The full set goal stays open; no partial implementation PR is published.

## Departure-counter and animation integration candidates

Donatello, Mutant Mechanic; The Ooze; Pizza Face, Gastromancer; Mind Transfer
Protocol; and Super Shredder pass all **138 TMT scenarios**, including 21 new
rules scenarios, log `<cache>/rustmage-tmt-departure-tests-complete.log`.
Admission is now 120/195, 75 gated, zero unresolved. 49 new physical candidates
have focused rules evidence; 44 have integrated release evidence and 71
inherited flags still need audits. Full release validation of this batch is
pending, including public paid casts after admission.

A generic typed departure filter reuses the existing frozen pre-event provider
set for any permanent type, controller, destination and counter requirement.
Leader's Talent and Ninja Teen keep their existing typed definitions. Only
programs consuming counters capture a full owned counter snapshot in the
existing rare event payload; ordinary spell packets and hot objects do not
acquire another pointer. Mixed counters use the existing placement/replacement
batch, preserving new counter timestamps. All 19 counter kinds which can reach
a graveyard are tested; finality's actual exile exclusion is tested with Ooze.
Simultaneous source death, exact references, foreign ownership/pre-death control,
zero/stale targets, normal doubling and integer-overflow rollback are covered.

Mind Transfer Protocol sets 4/5 until end of turn while preserving types,
subtypes, colors and abilities under CR205.1b; Equipment unattaches normally.
Donatello and Pizza Face animate only if the target is still noncreature after
its counters are placed. A Vehicle crewed in response preserves its printed
base. Pizza Face may target another artifact or creature on either side;
The Ooze may target either graveyard. Source loss, invalid targets, Mutagen
cost/timing, Food entry, Pizza's actual 10/tap/sacrifice life gain and Super
Shredder's real menace blocking are tested with owned branch/replay checks.

The first focused run exposed incorrect unrestricted Object-target relation
constants and a helper consuming a new optional triggered announcement. Those
were corrected; the complete 138-scenario run passes. Observation schema is
177 and additive Mutant animation joins the existing compact subtype grants.
The full 195-card goal, inherited audits, set-only acceptance and final reviews
remain open; no partial PR is published.

## Validated 120-card checkpoint

All nine release phases pass on `1974e593` in 640.90 seconds.
3,389 library rules tests (one ignored), all 138 TMT scenarios (21 departure,
23 Ninja, 21 other Class, 53 Sneak, 20 initial), existing native games and seven
Python binding checks pass. The admitted scenarios now use normal paid casts
for the five new cards and real turns before their creature tap/attack fixtures.
Admission is 120/195, 75 gated, zero unresolved; 49 new physical candidates have
integrated release evidence, while 71 inherited flags still await audits.

[Portable evidence](tmt-departure-validation.json) retains report hashes, exact
shared libraries, resource limits, measured implementation throughput and the
Freeform regression benchmark. Disposable completed validator outputs are cleaned.
No TMT-specific simulation acceptance or final independent reviews are claimed.

Next: Mutagen Man; Mikey & Leo; Ray Fillet; Michelangelo, Mutant BFF; and
Mutant Chain Reaction, with shared activation-cost and counter-trigger support.
The full set goal remains open, with one implementation PR only after acceptance.

## Counter-consumer candidates and inherited land audit

Mikey & Leo, Chaos & Order; Michelangelo, Mutant BFF; and Mutant Chain Reaction
pass the complete 159-scenario TMT focused suite. The shared counter observer
records the placing player, includes the source itself, and retains existing
other-creature exclusions. The once-per-turn limit is consumed at triggering.
A current any-counter blocking rule applies after characteristic layers under
CR613.11, so removing recipient abilities cannot erase it. Optional zero targets,
all-targets-illegal fizzle, failed destruction, control changes, prevented and
entry counters, all counter kinds and replacement batches are exercised.

Admission is 123/195, 72 gated, zero unresolved. 52 newly authored physical
candidates have focused evidence; 49 have integrated release evidence. Ten
inherited land audits are candidates: the five TMT gainlands use new paid-land
scenarios; five basic reprints reuse their canonical source pins and the existing
shared land-play/payment/privacy/branching scenarios. Their typed behavior is
retained. Integrated release validation is pending, including paid public casts
of the three newly admitted cards; 61 other inherited flags remain unaudited.

The first focused run found three fixture setup failures, corrected before the
159-scenario pass. An accidental production-LTO focused build was explicitly
stopped with its log and guard report retained; no successful test result is
claimed for that run. Subsequent focused builds use dev or release-test profiles.

Parallel isolated authors now own five mana consumers, eight spell/entry consumers
and fifteen inherited nonland audits. The integrator owns Mutagen Man/Ray Fillet
activation-cost mechanics and all combined catalogue regeneration and validation.
The full goal and one complete-set PR boundary remain open.

## Validated 123-card checkpoint and parallel continuation

All nine release phases pass on `07cceabc` in 658.91 seconds:
3,410 library rules tests (one ignored), all 159 TMT scenarios, existing native
games and seven Python binding checks. The three newly admitted cards use public
paid casts, including all three hybrid equivalents. Admission is 123/195,
72 gated, zero unresolved; 52 newly authored physical candidates are release
verified. Ten inherited land audits now have integrated release evidence,
including the existing canonical five-basic-land paid-spell scenario; 61
inherited admission flags remain unaudited.

[Portable evidence](tmt-mutagen-validation.json) preserves report/library hashes,
resource limits and measured implementation throughput. The accidental LTO test
build was stopped and its incomplete guard report retained; the final validator
passes under unchanged limits. Completed disposable validator outputs were
cleaned, preserving exact shared-library paths/hashes in the producing cache.
The Freeform regression benchmark remains distinct from pending TMT simulation
throughput and full-game acceptance.

At the user's request, three isolated authors are working on five mana cards,
eight spell/entry cards and fifteen inherited-card audits. The integrator owns
Mutagen Man and Ray Fillet's cost kernel in a fourth isolated checkout while
this integration branch remains available for combining completed batches.
No author batch is counted as integrated or independently reviewed yet. The full
195-card goal and one complete-set implementation PR boundary remain open.

## Parallel candidate integration at 136 admitted cards

Three independent isolated authors supplied five complete mana consumers,
eight complete spell/entry consumers, and thirty inherited-card audits in two
fifteen-card batches. Source commits are `42165f57` (mana), `5de3c322` (spells),
`08adab3e` and `822e39de` (audits); integration preserves all disjoint source,
provenance, test modules and shared-engine changes. Catalogue regeneration is
owned by the integrator. Each author used actual paid checked actions, full
pinned Oracle/XMage sources and isolated guarded builds.

The candidate branch admits 136/195, 59 gated, zero unresolved. 65 new physical
candidates have focused evidence, while 52 are integrated release verified;
40 inherited audits have focused evidence, while ten are integrated release
verified. 31 other inherited flags remain unaudited. Combined release validation
is pending and no full-set acceptance or final review is claimed.

Shared mana mechanics include artifact-or-activation and subtype-union restricted
mana, any-land tap bonuses, and exact current-power production in rare projected
funding worlds. Fifteen audit scenarios were not assumed from admission flags:
17 and 18 focused scenarios cover the two inherited batches. The mana author
passed 14 focused and 3,403 library tests; the spell author passed 18 focused,
19 existing library-movement regressions and Clippy, with the full library suite
passing before a final private-movement extraction. Integrated validation must
verify the exact combined source.

Manhole Missile's optional private bottoming suppresses public targeted-library
movement history, preventing disclosure of the selected hand card. Other typed
additions include creature-or-token sacrifice, positive kind intersection,
pre-destruction target mana-value conditions and simultaneous damage. Public
observation schema is 178. The authors continue on disjoint batches, while the
integrator implements Mutagen Man/Ray Fillet cost mechanics separately.

## Validated 136-card checkpoint

All nine release phases pass on `8154b4bb` in 652.55 seconds: 3,477 library
rules tests (one ignored), all 226 TMT scenarios, existing native games and
seven Python binding checks. Admission is 136/195, 59 gated, zero unresolved.
65 newly authored physical candidates and 40 inherited audits now have combined
release evidence; 31 inherited audits await integration.

[Portable evidence](tmt-parallel-validation.json) retains exact reports, library
hashes, resource limits and measured implementation throughput. Completed
disposable outputs were cleaned, preserving required libraries. The measured
Freeform regression is 178,454 decisions/s and 358 games/s; TMT-specific full-game
acceptance, remaining cards and final independent reviews remain pending.

The next isolated batches cover activation costs, top-library permissions and
blinking, reflexive/death interactions, and the final inherited audits.

## Final inherited audit integration

Source commits `3156da2b` and `fb7a7b65` add the remaining 31 inherited
nonland audits, with 19 and 18 focused scenarios respectively. All four
inherited nonland audit modules pass 72 combined scenarios. Full pinned Oracle
and XMage sources were read; no behavior corrections were required. All 71
inherited cards now have audit evidence, with 40 integrated release verified
and the final 31 awaiting the next combined release. Admission remains 136/195.

## Activation, blink and reflexive candidates at 151 cards

Source commits `22d2f563`, `c24ef593` and `b12036a5` implement 15 new
physical cards, integrated alongside all 71 inherited audits. Admission is
151/195, 44 gated, zero unresolved; 80 newly authored candidates have focused
evidence (65 release verified), and all 71 inherited audits have focused
evidence (40 release verified). The full goal remains open.

Token artifact activation reductions apply only to generic costs, including
announced X, with total costs locked before mana. Ray Fillet's non-targeting
+1/+1 counter cost reserves exact creature incarnations and pays mana before
removal. The integrated Ray/Mona current-power cross scenario and all 278 TMT
scenarios passed before the final blink/reflexive integration. Mutagen Man's
inherited fixed green cost was corrected against pinned XGG Oracle metadata.

Five blink/library consumers passed 25 focused and 44 affected regressions;
eight reflexive/death consumers passed 24 focused and 95 affected regressions.
Both passed Clippy. New mechanics include explicit top-library permission
providers, frozen entry counters, simultaneous blink/control, noncopiable
temporary copy haste, actual spell-target artifact entry triggers, optional
sacrifice/reflexive windows, signed death toughness and Ninja creature trigger
multiplicity. Schema is 179. Combined release validation is next; no set-only
full-game acceptance, TMT simulation throughput or final reviews are claimed.

## Validated 151-card checkpoint

All nine release phases pass on `10c8ed1c` in 666.18 seconds: 3,578 library
rules tests (one ignored), all 327 TMT scenarios, existing native games and seven
Python smoke checks. Admission is 151/195, 44 gated, zero unresolved. All 80
newly authored physical candidates and all 71 inherited audits have combined
release evidence. Final reviews and full-set acceptance remain pending.

[Portable evidence](tmt-151-validation.json) retains report/library hashes,
resource limits and measured whole-card implementation throughput. Earlier
combined checks caught module formatting and an inherited reverse-iterator lint;
both are corrected on the validated source. The Freeform regression remains
separate from the prepared TMT-specific five-color game/replay workload.

All remaining 44 cards are assigned in disjoint current and subsequent batches:
equipment/spells for the integrator, combat/damage for the audit author,
exile/control/outside-game for the mana author, and tokens/Sagas/search for the
spell author. No partial PR is published.

## Initial TMT-only natural-game evidence

The five-color TMT game workload passes 20 complete Standard games on
`7b387699`, with 11,823 decisions and 205 owned raw/index/restore checks. All
45 selected physical nonlands were cast; seven actual Sneak announcements,
Class levels, counters, departure-associated triggers and 13 private decisions
were observed. No concessions or illegal-action shortcuts terminate these games.

[Initial game evidence](tmt-games-initial.json) records fixed seeds, hardware,
logs and limits. The development-build rate is 1,968 decisions/s including
privacy observations, snapshots, sibling equality and replay. This covers core
mechanics and 45 cast identities, not yet the entire pool or optimized throughput.

## 165-card candidate integration

Combat source `b52d6847`, exile/return source `a84ea2a7` and equipment source
`c9f4886c` add 14 focused-tested physical cards. Admission is 165/195, 30 gated,
zero unresolved. 94 newly authored candidates have focused evidence (80 release
verified); all 71 inherited audits remain release verified. These batches passed
17 combat, 20 exile/return and 15 equipment scenarios respectively, affected
regressions and Clippy in isolated worktrees. Combined release remains pending.

New shared behavior includes first-combat ordinals, retained received-damage
mana, limited linked-exile plays, exact permanent-entry history, linked return
choices, optional hand/graveyard shuffle, successful attachment reflexives and
granted any-recipient combat-damage batches. Novel Nunchaku uses the current
equipped creature (or source attachment LKI) under Oracle/CR608.2h, correcting
XMage's captured-original-target discrepancy. Observation schema is 180.

## 173-card candidate integration and outside-game API

Source `efbd038e` adds eight alliance/search/cast-history consumers and the
stable Dinosaur Soldier token factory at 5360. Its 21 focused scenarios, 157
affected regressions, 63 Java pins and Clippy pass. Triceraton Commander's
inherited fixed white mana components were corrected against pinned XXWW
Oracle metadata. Admission is 173/195, 22 gated, zero unresolved. 102 new
physical candidates have focused evidence (80 release verified).

The prior combined 165-card source `70249133` passed all 379 TMT scenarios and
the 20-game TMT workload. The new batch adds explicit resolution-time keyword
options, per-turn modal reservations, spell-type and actual mana-spent history,
optional life/counter choices and actual search movement completion.

Tested outside-game API source `daa08e99` supplies a checked constructor with
private owned sideboard pools; existing Config literals remain valid. It enforces
frozen deck rules and exact membership/movement/observation/branch behavior and
admits no card by itself. Schema is 181 for the added private outside-game field.
Combined release validation follows; full-pool games and final reviews remain
pending while isolated authors complete all 22 remaining cards.

## Validated 173-card checkpoint

All nine release phases pass on `b8275317` in 689.85 seconds, including
3,655 library tests (one ignored), 405 TMT scenarios, native games and seven
Python smoke checks. All 102 newly authored candidates and 71 inherited audits
have combined release evidence. Admission remains 173/195 with zero unresolved.

[Portable evidence](tmt-173-validation.json) records preserved libraries and
resource limits. The measured continuation interval verified 22 new physical
cards in 2,246.78 seconds (35.25 cards/hour). The last 22 cards, complete-pool
games, optimized TMT throughput and independent reviews remain pending.

## 188-card candidate integration

The final control/exile and damage-legend sources `a99afa97` / `0e9a9038`,
Casey source `99aae481`, and final spell source `0f4a895a` integrate 15 more
focused-tested physical cards. Admission is 188/195, seven gated, zero unresolved.
All 117 newly authored candidates have focused evidence (102 release verified);
all 71 inherited audits remain release verified.

[Candidate evidence](tmt-final-candidates.json) records source hashes and terminal
checks: 63 control/exile scenarios; 17 damage scenarios plus 1,121 regression
executions; 15 final-spell scenarios, all 400 author-branch TMT tests, 27 kicker,
81 Aura and 213 return regressions; and all-target Clippy. Casey's six scenarios
are included in the author-branch TMT check. Shared mechanics retain owned
delayed-step packets, exact attacking-entry defenders, sparse hexproof counters,
color-specific timestamped hexproof, private outside pools and coupled nonmana
kicker funding. No final independent reviews are claimed.

The full-pool game workload is prepared with all 195 members, eleven mechanic
themes, actual private tournament sideboards, both starters, fixed seeds and
complete journal replay. It remains gated until the last seven cards integrate.

Combined source `431627ae` passes all 464 TMT scenarios, the 20-game workload
(with all 45 selected nonlands actually cast) and all-target Clippy. Source
`5cd4fe36` additionally passes the complete library: 3,715 passed, one ignored,
zero failed. The expanded acceptance integration target compiles; its execution
remains gated on seven pending cards.

The eventual single TMT PR targets `codex/portable-set-skill`: ECL PR463 was
already merged there at `32457107`; its tree matches the TMT baseline `1a795dcf`.
This preserves the existing stack above open prerequisite PR461 and keeps the
TMT diff separate from previously accepted ECL work.

A fresh nonauthor preparatory inspection found omitted boxed-packet accounting
and traversal in Old Hob's suspended token completion. The fix passes a real
paid-spell/two-Doubling-Seasons scenario for both seats, including exact owned
byte difference, packet visitation and public completion of four hasty tokens.
[Finding and evidence](tmt-preliminary-review-fix.json) distinguishes this
preliminary check from the still-pending final published-head reviews.

## Complete candidate integration, acceptance pending

Final source `341595ef` adds the last seven physical cards, with 28 focused
scenario families, 277 affected regressions, Clippy and 154 Java class/helper
pins passing. All 195 physical cards are admitted; zero gated or unresolved.
[Final candidate evidence](tmt-195-candidates.json) preserves the terminal logs.
All 124 newly authored cards have focused rules evidence; 102 plus all 71
inherited audits currently have combined release evidence.

The complete-pool natural games, private mixed-search and outside-card choices,
optimized throughput and final combined release are the next acceptance gates.
Final published-head reviews remain pending; no implementation PR is open.

The first complete-pool development run exposed slow dense-artifact decisions
(up to 28.083 seconds including verification). It was deliberately stopped
with inputs fixed, and is not counted as acceptance. Named counters globally
disabled equivalent-basic-land funding memoization; an isolated author is
testing a narrower exact-identity rule. The declared decks, seeds and policy
remain fixed. [Investigation record](tmt-performance-investigation.json)
preserves the stopped log and distinguishes verified-decision timing from
optimized simulation throughput.

Funding fix `9985ba23` integrates at `7b02e584`: 87 affected scenarios and
all-target Clippy pass. Counter-bearing resources remain exact, while unrelated
plain basics regain symmetry. Actual Weather Maker fixtures for both seats
visit 12/24/36 states with 6/12/18 Forests; the 18-Forest failed DFS takes about
13.5 ms in dev and leaves snapshots/observations unchanged. The same dense-game
2,304-decision prefix improves from 185.897 to 38.431 seconds including strict
verification. Both dev experiments were deliberately stopped and preserved.

The identical 172-game proof now moves to the required release validator's
optimized `release-test` profile with runtime assertions and overflow checks.
This is separate from the later production simulation throughput measurement.

The first 195-card release attempt passed formatting, provenance/source checks,
Clippy and all 3,746 library tests (493 TMT scenarios). Its full-pool game run
completed 75 games before an observation-only smoke-policy lookup panicked on
a legal top-library legacy cast; the overall validator failed and is not a pass.
The policy now uses the explicitly authorized `known_library_top` view, with a
both-seat paid Mikey scenario proving sibling hidden-next-card changes cannot
affect observations or scoring. [Fix evidence](tmt-library-policy-fix.json)
preserves the failure and focused pass. Complete validation is rerun with test
output enabled so successful game metrics are retained directly.

The second release attempt passed all 3,747 library tests, then failed in a
forest/artifact theme game when Krang's intrinsic affinity was queried as a
battlefield reduction. The generic reducer now excludes intrinsic rules before
requesting a continuous group identity. Both-seat paid Krang/subsequent-spell
regressions and all four short natural matchup replays pass.
[Diagnostic and repair evidence](tmt-krang-query-fix.json) preserves both failed
combined attempts; full acceptance and final reviews remain pending.
