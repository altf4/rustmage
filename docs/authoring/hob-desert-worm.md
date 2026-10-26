# Desert Were-Worm: first qualifying attack declaration

Isolated author base cc636931, branch codex/hob-desert-worm. Root owns generated
registration, final integrated acceptance and two independent nonauthor reviews.
No public wire/schema changes, new identities, PR, push or merge.

Oracle81c2913f-42b7-4cfa-adc0-b3416fbe2ec6:4RR,0/5 red Dragon Wurm. Current
controlled Mountain count modifies power by2 each; first attack declaration each
turn whose creatures' total power reaches12 untaps all current attacking creatures
and adds a combat phase after this one. The Worm need not attack.

Complete XMage DesertWereWorm.java at492369c7844b8d76479f0393ff9eb6b0b0cf30d7
and full behavior-bearing helpers are pinned in paired JSON. StaticFilters fixed
creature/attacking-creature bindings were checked at854/1295; their constructors
and AttackingPredicate were read. The Java watcher is game scoped and is installed
with a provider. Rust deliberately records history without a provider and keeps
separate player histories: later entry, control transfer and ability reacquisition
cannot manufacture a new first qualifying attack.

Rules authority: official2026-09-25CR cached at
`<cache>/rustmage-authoring/rules/MagicCompRules-20260925.txt`, SHA256
8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca.
CR508.1/508.1m freezes declaration before attack triggers;508.3a excludes creatures
put onto the battlefield attacking. CR107.1b preserves signed power;613.4c puts the
bonus in7c after7b base setters.113.7a preserves queued abilities independently of
the source;608.2c orders untapping before the additional combat; untapping does
not itself remove a creature from combat. This event condition is not an
intervening-if and does not recheck power on resolution.

| Obligation | Independent outcome proof |
| --- | --- |
| Paid4RR0/5, subtype/color/current Mountain7c modifier | exact payment, hand0power, own basic/nonbasic Mountain count; Forest/enemy Mountains excluded; later7b setter, counter, paid Giant Growth, Mountain departure and source control transfer |
| Source itself may supply threshold |5Mountains=>10/tapped/noextra;6=>12/untapped/oneextra |
| Actual legal attack selection | newly paid source summoning sickness, enemy and tapped creatures reject transactionally; empty declaration leaves historyNone |
| First qualifying attack can be later combat | real Aurelia attack11 grants combat; paid post-declaration Growth cannot retrotrigger; second14 grants Worm combat; third14 gives none |
| Signed event snapshot | signed11/12/13 boundary, including-1power attacker; remove6power attacker before resolution; already qualified packet still untaps/addscombat |
| Provider-independent history | attack12 without Worm via Aurelia, then new Worm sees no fresh first; two Worms both see initial event and add two combats |
| Suppression/reacquisition | paid Witness removes scaling and trigger; qualifying declaration still recorded; Aura departure restores scaling but later qualification does not trigger |
| Queued source removal/suppression/control | exile, actual paid flash Eaten by Piranhas or controller change after declaration cannot cancel packet; idle tapped creature stays tapped |
| Resolution recipients | freshly put-attacking Dragon untaps; source not attacking and departed/reentered attacker stay tapped |
| Put attacking is not an attack declaration | declared2 plus two put-attacking6tokens neither qualify nor change history |
| Controller and turn lifetime | opposing provider does not trigger; its transfer does not reset actor history; real turn initialization clears both histories; next-turn first declaration qualifies |
| Owned/failure boundaries | raw/index/restore at selected attackers, parent disposal, identical queued outcomes; signed-sum overflow after taps aborts complete FinishAttackers transaction, preserving revision/snapshot/taps/history |

Mechanics: one private TriggerCondition::FirstAttackWithTotalPowerAtLeast(u16),
one private Option<i128> maximum per player. None distinguishes no actual attack,
including threshold0, without a sentinel. Empty declarations do not update it.
The full signed declared-creature sum and prior maximum are frozen after
simultaneous declaration taps and before per-attacker callback dispatch. Only
these event facts feed the new trigger; every valid nonempty declaration updates
history even with no observing source. Fields clone/snapshot with PlayerState,
are included in its existing size accounting, reset in begin_turn, and are not
projected into PlayerView/TurnFactsView. No new pending choices or allocations.
Existing UntapCreatures(any,attacking) and AdditionalCombat are reused unchanged.

Fixture preflight: every named card/token is admitted; Hamlet Glutton is6/6,
Aurelia3/4 vigilance/haste, Hill Giant3/3, Bear Cub2/2, exact red6/6flying Dragon
factory5416. Shivan Dragon is5/5 and was rejected during preflight as the wrong
threshold fixture. Eaten by Piranhas has flash and costs1U, allowing a legal
paid response; Witness Protection is used only before combat. No fabricated cast
timing or missing/gated fixture is required.

Validation passed coherent guarded author run. Consistent opt0 author target,
persistent driver/full outer log/results for every group. Production-profile
release acceptance and complete-pool gameplay/performance remain root-owned.

Initial run-b856c2a683d642f89b8682d00640e114 naturally terminated with compile
exit101 before scenarios: a test match arm returned ZoneChange while other arms
returned unit. Repaired the test arm with an explicit statement; no engine change.
All inputs remained fixed until both original exec and resource report were terminal.
Full original outer/results/resource logs remain preserved.

Second run-d90b5708c2e84e39be713d357e70cc64 naturally terminal101:8passed/4failed,
compile29.07s,guard29.267826s,peak2494091264bytes,noOOM/timeout/orphans. Three
Mountain-power failures identified a missing leaf CardExtras.scaling_modifiers
entry (the availability fragment alone does not populate the modifier table).
Added the same const modifier to the existing table; no shared engine repair.
The fourth failure was the put-attacking fixture passing one defender for two
tokens; corrected to one defender per token. All inputs stayed fixed to terminal.

## Final terminal receipt

`<cache>/rustmage-authoring/hob-desert-worm/run-21586afd81314560a2fa35bd569e713c`
preserves driver.py, ENTIRE guard stdout/stderr outer.log, results.json, resource.json
and all4group logs. Original session47123 naturally returned0, resource terminal=true,
exit_code0. One Cargo job/test thread,6GiB enforced cap, no memoryhigh/max/OOM events,
no timeout/orphaned process. Source inputs stayed fixed through natural completion.

104 successful scenario executions across four groups:12Were-Worm,49attack,
29combat,14additional-trigger regressions. Filters overlap; 93 unique scenarios passed.
0failed/0ignored. Compile28.37s (consistent opt-level0 author profile), final guard
42.741315644s including shared-slot wait/driver overhead, peak2499051520bytes.
Reported executable scenario times total0.62s. This is a bounded author workload,
not full-game simulation or production performance. Production profiles unchanged.

Implementation scope:1whole card plus first-qualifying-declaration core, from
isolated checkout creation2026-10-26T22:48:52Z through final guard terminal at
2026-10-26T23:01:47.376395+00:00. Elapsed 775.376s including source reading, authoring, failed/fixed
builds and waits. All three guarded attempts and their failure evidence remain
preserved; this local interval is not an all-set throughput claim.

Full source pins5417definitions/1570source blobs verify. Final catalogue version
171a9e3cc7fb7e1c1c4f932ab3fda884585a691485538580849bf97a67655fca.
Formatting and diff-whitespace pass. No known whole-card behavior blocker remains.
Two independent nonauthor reviews, generated registration and integrated release
acceptance remain root-owned. Local generated catalogue/coverage and assurance/mod
registration are retained for replay but excluded from source commit.

Compiled source SHA256:

- `src/card_definitions/hob/desert_were_worm.rs`: `3f89593b75edc32895b04d3333176ffcd7a610b213806d27b1df41f65d914161`
- `src/card_definitions/hob/desert_were_worm.json`: `a15978e90ebac7b43994fa0a528ba3b263d5fe79aac1ff746e335e42e11b9a50`
- `src/cards.rs`: `790a725136f1fe48626fa1d93abef14fd937ee091e9aa23d9f8af049c67d0741`
- `src/engine.rs`: `1ee00eb5d8559d9b9692dc7c375305e3d5d89992ef1a4af06a30622b7bb289df`
- `src/engine_turn_facts.rs`: `15ad8cd1d167d1b4da9ba7c403b043d8e50b769dac3582eff7ba1bb8782615ad`
- `src/engine_programs.rs`: `54f272f8e85eaea36f23b5f0544dd5e20d92999b95b921d60d4891f5b0ce11f4`
- `src/engine_stats.rs`: `e73a95f7c38babad0f7fa667c931572c03a2ce0631f74f6eeda32a4c9eaf2204`
- `src/assurance/hob_desert_worm.rs`: `8016fb7300150f10b6e7a5412ccb0daec8d5e1c598412c56cb118fc9a4e8449c`

Receipt SHA256:

- `driver.py`: `0ab0dee71bead16651a04a5f08d89e8e364a8face0fd571364ace8ec6a70d783`
- `outer.log`: `0e51d9f748d7808f544a78d8b5dee12968c8c776b4cc29cea328647d02cc51b7`
- `results.json`: `eac4d371ad4d44e1cd34e49f79b873b06b83f3a2389cff81c99b518e923ec19e`
- `resource.json`: `ce81dd4eb88126413201f1458a598b3b3119eb0d781626d02382612146e72cda`
