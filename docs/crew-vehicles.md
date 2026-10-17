# Crew and live Vehicle characteristics

Issue #250. Author checkpoint from main74c1f0c3 (1,707 supported Standard bodies, schema73). This stage admits the 24 complete bodies below, bringing the standalone catalogue to1,731 with the same83 token recipes. Public observation schema74 exposes captured current kinds, private Crew selection, paid stack context and historical paid contributors. The completed validation below distinguishes the final stage from its earlier compiling checkpoint.

Crew is an ordinary instant-speed activated ability. Its non-targeting cost taps any chosen subset of OTHER untapped controlled creatures with sufficient total power. Summoning sickness does not prohibit contributing. Zero, negative, excess and repeated contributors across different paid activations remain legal when the exact signed total satisfies the cost; already selected refs cannot repeat within one payment. The exact signed accumulator does not clamp or overflow i128. Crew has no mana component and creates no mana-payment window.

`ChooseCrewer { object: ObjectRef }` and `FinishCrew` operate on an actor-owned pending frame. Both actual tap events and historical contributor refs commit as costs, independently of whether the ability resolves. `SpellView.crew` owns that activation's ordered contributors and represented total (`None` if the exact total lies outside i128). `crewed_vehicles` is a public per-turn union of distinct exact contributors, including countered Crew costs. It is historical participation, not a statement that an animation resolved. History resets at the next turn, including when cleanup requires extra priority.

On resolution, the same surviving Vehicle incarnation becomes an artifact creature until cleanup. Animation preserves counters, colors, subtypes, generation and printed card identity. A sparse State-owned sorted exact-ref registry supplies live battlefield kinds; immutable `Card.kind_mask` and `Card.is_creature` remain printed traits. Hidden zones and spell filters use printed traits. `ObjectView.kinds` and `DamageFacts.kinds` capture live characteristics before movement; the graveyard version of the same card returns to printed Artifact identity. A printed star requires a wholly compiled characteristic ability: Lumbering Worldwagon uses actual controlled-land count rather than a zero approximation.

All live creature queries, attachment legality, combat, target predicates, entry/death/damage events, stat counts, derived keyword grants, activation sickness and actual/virtual funding use the same borrowed kind projection. Virtual resource normalization distinguishes animated and nonanimated identical Vehicles. Crew is never synthesized inside another payment's mana search. A newly entered unanimated Caravan, Quinjet or Bike can tap for mana; animating that summoning-sick source prevents its tap-symbol ability unless it gains haste. Becoming a creature is not an entry/cast/attack event. Torpor Orb does not suppress an unanimated Vehicle's artifact entry; Doorkeeper Thrull can.

Cleanup clears animation simultaneously with damage and temporary stats/keywords. Illegal creature Auras go to the graveyard and Equipment detaches through state-based actions, preserving the cleanup priority cycle. Counters persist and apply if that same incarnation becomes a creature again. No immutable catalogue metadata is mutated and no eager per-object animation flag is allocated.

## Complete bodies

Adventurer's Airship; Air Response Unit; Boosted Sloop; Broadcast Rambler; Burner Rocket; Careening Mine Cart; Clamorous Ironclad; Cultivator's Caravan; Debris Beetle; Dependable Quinjet; Fire Nation Warship; Haunted Hellride; Hulldrifter; Lumbering Worldwagon; Magitek Armor; Ripclaw Wrangler; S.H.I.E.L.D. Helicarrier; Skybox Ferry; Spotcycle Scouter; Strixhaven Skycoach; Thopter Fabricator; Tundra Tank; Turtle Blimp; Veloheart Bike.

The closed compiler authenticates the entire Crew declaration and recognized historical/current reminder, then binds only deictic `this Vehicle` source phrases for its existing source-effect grammar. It preserves the original Artifact/Vehicle metadata and accepts only complete residual bodies. External Vehicle subjects, unsupported cost combinations and unknown remaining clauses are not erased. Clamorous Ironclad and Skybox Ferry retain actual Cycling; Lumbering retains its complete optional entry-or-attack search and dynamic power. Unknown/history-dependent/restricted-mana/alternate-animation/Equipment-Vehicle/DFC relatives remain gated.

## Rules and evidence

Primary rules: [2026-09-25 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),301.7,302.6,702.122. The snapshot reminder uses an older abbreviated description without `other untapped`; current cost semantics follow the current rules, including self-Crew rejection.

Pinned Java492369c7844b8d76479f0393ff9eb6b0b0cf30d7. All24 complete constructors, CrewAbility and AddCardTypeSourceEffect were read. The [all-section manifest](crew-vehicles-admissions.json) records214 hashes across110 unique pinned Java files:24 complete bodies, six explicitly gated relatives and shared helpers. Every record was checked against both the checkout bytes and pinned Git blob. Initial full-constructor audit is retained at `<temp>/issue-crew-scout-full-constructors.log`.

Checkpoint native:1,577 tests pass, including seven Crew groups. All24 have paid hardcasts and real Crew under both controllers, per-action clone/restore/stale-index replay, exact source/paid-history checks, zero/negative/excess/repeated subsets, actual/virtual mana sickness, Orb/Thrull entry suppression and actual cleanup attachment/SBA behavior. Four compiler methods pass, with malformed/overflow costs, declared counts, unknown residuals, external Vehicle subjects and unsupported compositions rejected. Completed compiler/public workload and final broad Python/native/lint results appear below.


## Standalone acceptance and preservation

`python/test_crew.py` passes48 actual-paid full-body scenarios (all24 under both controllers) in56.306seconds using immutable `<cache>/rustmage-immutable/crew-dd98399-schema74.so`, SHA256 `c4c9d7713a324e385ed8fef162e26ab70d41d8790e9630ea253aea5086b8b149`. Both viewers' observations round-trip after every action; private Crew frames deny the wrong actor and self-contribution, and parent-closed children remain usable. Actual tests cover paid Cycling for both bodies, three Vehicle mana abilities, Fabricator's real second draw through Thrill, Warship's real destruction into a Clue, both searches, Scry2, target boosts/keywords, draw/discard, tokens, life and attacks. The first pilot's token-count/keyword/optional-search fixture mistakes were corrected without changing runtime.

`examples/standard_crew.rs` completes100 legal60-card Standard games twice with byte-identical full vectors:95,925 decisions/replays,95,925 parent-closed branches,6,247 private Crew frames and108,419 invalid/stale denials. Every card has both-seat positive paid cast, Crew, entry and attack witnesses; both Cycling abilities, all three mana abilities, Fabricator's real Thrill draw and Warship's real death have separate nonzero effect ledgers. Ordinary combat/draw outcomes end every game. Both source spells and graveyard versions retain printed noncreature identity. Initial logical state is20,184bytes, compared with the measured exact parent20,152 (+32); this is combined layout accounting, not a per-object animation flag. Workload maximum is23,084bytes. No Java throughput comparison is claimed.

The original324-method compiler run exposed exactly three historical negative lists which still named Clamorous Ironclad, Thopter Fabricator and Tundra Tank as unsupported. Only these newly complete identities (and positive Skybox coverage) were updated; all other gated names remain negative. All324 compiler/data methods then passed in501.256seconds. Generated catalogue and formatting checks pass. Final Rust validation passes1,579 unit tests (4.96seconds) plus all12 integration tests; final all-target Clippy passes in2.10seconds. Two additional native groups now pass real animated/unanimated destruction LKI and an exact signed sum outside i128, plus late actual tap-trigger allocation failure restoring the full payment/history/private frame. The wide-sum rollback fixture initially tried exhausting a stack ID after its packet had already been allocated; it was corrected to a genuine Sunscribe tap-trigger allocation and exact Crew-packet lookup.

Canonical comparison against main74c1f0c3 proves exactly24 definitions changed and5,243 unchanged; all5,267 identities and color masks, all83 token recipes and one emblem are unchanged. All597 prior numeric helper arms are unchanged;75 immutable printed-Vehicle metadata arms and Lumbering's one actual dynamic-CDA arm are added. Snapshot/replay compatibility is catalogue-fingerprint scoped.


Final fresh-library Python collection passes all218 methods in562.326seconds, including the48 new paid scenarios. The external runner explicitly supplies the same immutable library to every Game constructor and the isolated NUL-card subprocess; no stale default library or ignored environment override supplies evidence. Runner and log: `<temp>/issue250-python-fresh-runner.py`, `<temp>/issue250-python-full-fresh.log`.

An optimized standalone benchmark linked against the release library completes1,000 smoke games/571,049 decisions in2.845seconds, approximately200,697 decisions/second; measured clone216.60ns and restore228.03ns. This includes native smoke policy/observation construction, excludes model/tensor/JSON, and is not an XMage speed/parity comparison. Log `<temp>/issue250-benchmark1000-final.log`. The final lint-only pilot rewrite reproduces all100-game output vectors exactly; logs `<temp>/issue250-standard100-first.log`, `<temp>/issue250-standard100-second.log`, `<temp>/issue250-standard100-final.log`. The initial Standard pilot incorrectly used non-Standard Hill Giant, omitted a legal mana-conversion action and attributed its Thopter's flying to Broadcast Rambler; all three fixture errors were corrected without changing production.

Final runtime/catalogue fingerprint: `8ea9ef36298eff71fd9ee7add7d2512e4ab11c5717931241b5f0d8b29115924b`. Production is unchanged from the reviewed all24 checkpoint `dd98399`; subsequent changes are acceptance, test corrections, provenance and evidence. Completed disposable Cargo outputs are removed after validation while every shared-library exact path is retained.

## Payment-library composition checkpoint

The integration branch composes standalone Crew `4cd27a3` with reviewed main
`63169e1` (actual cast payments). This interim composition admits 1,751/5,164
Standard identities, keeps 83 recipes and uses observation schema 75. It preserves
both owned `SpellView.crew` and `cast_event` contexts and counts both in snapshots.
Captured source power and Increment's creature predicate now use live Vehicle
kinds. The combined catalogue fingerprint is
`cf219c014a49df3f29bc127cf4da6abb3528e2c89410327cbb9c4fbf46338616`.

All 1,608 combined native tests, all-target Clippy and generated-data/format checks
pass. Standalone compiler/public/Standard evidence above remains attributed to its
standalone source; fresh combined-library acceptance and composition with the
replacement branch are still pending. This is an intermediate draft checkpoint,
not final integrated approval or a claim that the all-Standard goal is complete.

The fresh interim composition library at `b43d3cd` has SHA256
`2d11ecceae5e160b8aeef4f49de6cc6637eed23d96d5cb2a0bb16bfce85176c8`.
Root explicitly pins its immutable path and asserts schema75 before running the
Crew and cast-payment public modules together: five methods, 96 paid scenarios,
all pass in 116.780 seconds. Both exact compiled Standard drivers also pass:
Crew100 reproduces 95,925 replays/parent-closed branches, 6,247 private frames,
108,419 denials and all cast/Crew/entry/attack/residual vectors; payment100
reproduces 104,413 replays/parent-closed branches, 1,330 private frames, 208,826
denials and all eighty effect cells. All two hundred games end naturally.
Initial logical state is 20,280 bytes; maxima are 23,516 (Crew) and 23,552
(payment). These focused combined results supplement the 1,608 native tests;
they do not stand in for the pending final full collections after replacement
composition. Evidence logs are `<temp>/rustmage-root253-public-focused.log` and
`<temp>/rustmage-root253-standard-{crew,payments}.log`.

## Complete payment and replacement composition

The final execution source at `c84ab2a`, carried unchanged into reviewed-main
merge `7379c85`, includes payment PR251 and replacement PR252. Coverage is
1,760/5,164 Standard identities, with 84 token recipes, one emblem and schema76.
The fingerprint is
`92dcc6649eaf01aa27336a8c42d9e204324ff669782f85afd3cb01300bd32543`.
Both owned Crew and actual-cast-payment contexts survive branches. Counter
replacement events capture live battlefield kinds; incoming entry counters
retain the entering characteristics. Job attachment filters also query current
creature status. A native both-seat/order cross test actually pays Caravan Crew
and Snakeskin Veil, proving the animated Vehicle receives the noncommuting
Michelangelo/Loading Zone counter modifications, with atomic wrong-actor
rejection and parent-closed replay. Its initial public-frame visibility assertion
was a fixture mistake; battlefield counter choices are public, while only the
entitled actor can apply them. No production privacy change was needed.

All 1,624 native tests (1,612 unit and twelve integration), all-target Clippy,
formatting and generated checks pass. The clean compiler/data collection passes
334 tests in 377.673 seconds. The final fresh public collection passes all 225 unique methods in 510.027
seconds. It explicitly asserts the immutable library's schema/fingerprint, with
the same explicit path in its subprocess. The immutable library is
`<cache>/rustmage-root253/librustmage-92dcc-schema76.so`, SHA256
`118a17ec5a620c2695bdd6b8938cc01016b6fcb772f1ae02c3d661881adb1585`.

All three exact-source Standard100 drivers pass with natural terminal outcomes
and positive full-body effect vectors. Crew completes 95,925 decisions/replays
and parent-closed branches, 6,247 private frames and 108,419 denials; payments
completes 104,413 replays, 1,330 frames and 208,826 denials; replacements completes
187,965 replays, 44 frames and 375,930 denials, including actual Warp/exile/recast
routes. Initial logical state is 20,296 bytes; workload maxima are respectively
23,532, 23,568 and 21,144 bytes. These are current composition measurements,
separate from the historical standalone and interim metrics above.

Both independent nonauthor reviewers verify all 214 pinned source hash records
across 110 Java files, all 24 whole constructors and CR702.122. Against actually
merged main `f7a2fa0`, exactly 24 ordinary definitions change, 5,159 ordinary
definitions plus 84 token definitions and one emblem remain semantically intact:
5,244 preserved out of 5,268 total. All 5,183 ordinary IDs, canonical identities
and color masks and all recipe metadata remain intact. All 610 prior helper arms
are preserved; 75 printed Vehicle metadata arms and one dynamic CDA arm are added.
The reviewers' focused native/compiler/public/Standard evidence is separately
attributed in the final PR review record; neither claims root's broad collections
as their own runs. Final approval requires exact-head reconfirmation after the complete public
result. The remaining 3,404 identities and broader rules/RL work remain
part of the active goal.
