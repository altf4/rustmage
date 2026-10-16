# Exact library searches and hand discard activations

Issue [#217](https://github.com/altf4/rustmage/issues/217) adds **nineteen complete Standard bodies**, extending the initial ten after auditing natural additional bodies made executable by the same primitives. Historical standalone core measurements below use `2b67f9a221f5d077f4942ecbc5f6b5c8aefbd514`; the integrated release builds on main `85587e2121a36cc5fe35350e4ab35aa38b7d2e3a` and its evidence follows at the end. Support advances: **1,533 → 1,552 / 5,164** supported Standard identities, six regression cards, **72 → 73** complete token factories, and one emblem. Unsupported complete bodies remain rejected; a recognized search or cost never authorizes dropping the rest of a card.

| Card | Complete body |
| --- | --- |
| Circuitous Route | {3}{G} sorcery; up to two basic land **or** Gate cards enter tapped, then shuffle |
| Map the Frontier | {3}{G} sorcery; up to two basic land **or** Desert cards enter tapped, then shuffle |
| Dragonstorm Forecaster | {U} Human Scout 0/3; {2}, tap searches for exactly Dragonstorm Globe or Boulderborn Dragon, reveals into hand, then shuffles |
| Magitek Infantry | {W} artifact Robot Soldier 1/1; +1/+0 while another owned artifact exists; {2}{W} searches its exact name onto the battlefield tapped, then shuffles |
| Honored Knight-Captain | {1}{W} Human Advisor Knight 1/1; entry creates a white Human Soldier 1/1; {4}{W}{W}, sacrifice searches Equipment onto the battlefield, then shuffles |
| Mystical Teachings | {3}{U} instant; instant **or actual flash ability** search, reveal into hand, shuffle; flashback {5}{B} |
| Glimpse the Core | {1}{G} sorcery, both modes: basic **and** Forest search onto the battlefield tapped/shuffle; or return an exact owned graveyard Cave tapped |
| Radioactive Spider | {G} Spider 1/1, reach/deathtouch; {2}, sacrifice searches Spider **and** Hero card, reveals into hand, shuffles; sorcery timing |
| Spinewoods Armadillo | {4}{G}{G} Armadillo 7/7, reach/ward {3}; {1}{G}, discard this exact hand card searches basic land **or** Desert, reveals into hand, shuffles, then gains 3 life |
| Call the Mountain Chocobo | {3}{R} sorcery; Mountain search/reveal into hand/shuffle **then** full Chocobo token; flashback {5}{R} |
| Chocobo Racetrack | {3}{G}{G} artifact; owned landfall creates a full Chocobo token |
| Gysahl Greens | {1}{G} sorcery; full Chocobo token; flashback {6}{G} |
| Harvester of Misery | {3}{B}{B} Spirit 5/4, menace; entry gives all **other** creatures -2/-2 this turn; {1}{B}, discard this hand card gives a target creature -2/-2 this turn |
| Outcaster Greenblade | {2}{G} Human Mercenary 1/2; entry basic land **or** Desert search/reveal into hand/shuffle; live +1/+1 per owned Desert |
| Pia, Aether Ascetic | Legendary {2}{G} Human Druid 2/2; entry optional actual discard, only if paid enchantment search/reveal into hand/shuffle |
| Simulacrum Shaper | {1}{G}{G} Elf Druid 2/2; optional entry basic land search onto battlefield tapped/shuffle; mandatory death draw |
| Solemn Simulacrum | {4} artifact Golem 2/2; optional entry basic land search onto battlefield tapped/shuffle; optional death draw |
| Steel Wrecking Ball | {5} artifact; entry deals 5 to target creature; {1}{R}, discard this hand card destroys target artifact |
| Sureshot Sower | {1}{G} Human Archer 3/1, reach; {3}{G}, discard this hand card destroys target creature with current flying |

The [complete provenance manifest](exact-search-admissions.json) records **148 SHA-256 records across 67 Java files**, including full admitted Oracle bodies, all admitted constructors, relevant effects/costs/predicates/token helpers and the explicitly gated Fabled Passage and The Masters of Evil. Every recorded file matches both the pinned Java revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7` and the source checkout. Fabled Passage still requires a binding to the exact searched object after shuffling and a conditional untap. The Masters of Evil's separate Villain anthem/Plan filter is not implemented here. Arbitrary names, mixed destinations, different-name requirements, variable quantities and unknown residuals remain gated.

## Immutable predicates and private transactions

`SearchFilter` gains shared immutable `AnyOf`, `AllOf`, `Name`, `Kinds` and `Keyword` values. Union branches retain their individual meaning: basic land or Gate is not “basic Gate,” and basic Forest is an intersection rather than a nonbasic Forest match. Spider Hero has two subtype predicates, without an invented creature restriction; a Kindred card with changeling can qualify. Changeling never supplies land, Equipment or other noncreature subtypes. Named searches compare printed names exactly. Mystical Teachings inspects the Library card's operative keyword characteristics; flashback and an “as though it had flash” casting permission do not satisfy Flash. Accepted cards have immutable characteristics outside the battlefield; dynamic Library ability grants and unsupported faces remain gated.

Observation schema is **60** because these filter values appear in the existing owner-private `search_choice.filter` contract. No new observation fields or mutable state fields are added. C ABI remains 1; control protocol remains 2. The ordinary `ChooseSearch { card }` and `FinishSearch` actions are unchanged.

Every search can fail to find, even with visible eligible cards, under [CR 701.23b](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf). Distinct objects with identical printed names may both be selected; selecting one object twice is illegal. Partial choices stay in their original Library incarnations, remain private and leave RNG/order unchanged. Only `FinishSearch` commits simultaneous destination moves, new generations and shuffle. Revealed hand arrivals use the existing public `known_hand` exact references. Source departure does not cancel an independent search. The whole owned continuation, selections, pending entries, RNG and public knowledge participate in snapshots, clones and atomic rollback.

Declining Shaper/Solemn's optional search neither reveals the library nor shuffles it. Accepting and then finding zero still shuffles. Pia's optional discard is a real committed payment before searching; declining, or lacking a hand card, skips the search. An error on a later search commit restores that action while preserving a discard already committed by an earlier action. Following instructions wait for the complete search transaction, including entry replacements: Armadillo gains life after shuffle even on zero selection; Call creates its Bird only after the Mountain search completes.

Glimpse's second mode uses an owned graveyard Cave target and `ReturnToBattlefieldTapped` through the shared entry pipeline. It revalidates the same exact incarnation; a departed and reentered card is a new object ([CR 400.7](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf)). The mandatory tap is entry state, rather than a later operation that could occur after entry triggers.

## Ordinary hand costs and complete tokens

`Discard this card` uses the existing immutable activation `origin: Hand` and discard-source payment, with **cycling false**. It uses normal target declaration, funding, mana payment and priority; payment discards the exact source before opponents respond. It generates a real ordinary discard event and no cycling event or spell-cast event. Graveyard, battlefield and exile copies cannot use a Hand-only ability. Unsupported combinations with tap/sacrifice/exile/counter/life/additional costs, and discard-funded mana production, are rejected. Radioactive Spider's separate battlefield sacrifice activation retains sorcery timing.

The new canonical token factory is `@token/607bf09599a16caa64c6fee1`: printed **Bird Token**, green Bird creature **2/2**, with its full immutable “Whenever a land you control enters, this token gets +1/+0 until end of turn” ability. Chocobo sources reuse that complete recipe. Racetrack's later-created token did not exist when the land entered, so it gains no retroactive boost. Existing Birds each trigger for each simultaneously entering searched land; Racetrack creates its new Birds only when its independent triggers resolve. Temporary growth expires in cleanup, and token departures follow the ordinary token lifecycle.

## Preservation, memory and checks

Exactly **19** existing definitions change and **5,237** remain semantically unchanged after canonical factory-reference normalization. One new factory is added; all **72** prior recipe definitions and metadata, all **5,183 ordinary card IDs**, every previous color mask and all **476** prior numeric helper arms remain unchanged. Five helper arms are added (three flashback costs and two live scaling sources). Token/emblem numeric IDs are catalogue-scoped and may shift when a canonical recipe is inserted. Replay compares the same catalogue fingerprint: `04ead6ed7234c31bb805d57b3726cd5810053daefca6166f5aedc0a047f93c07`.

Initial logical state remains **19,944 bytes**, exactly the parent measurement. The filter tree and token trigger program are shared immutable metadata, with no per-object graph or new state allocation. The filter remains 24 bytes and its SearchSpec remains 32 bytes. The active two-land search fixture occupies 8,304 initialized logical bytes; a partial selection adds exactly four bytes per owned selected CardId; the native memory fixture checks the exact initialized-byte delta and snapshot restoration. Logical bytes exclude spare capacity, allocator overhead, static definitions and externally retained snapshots. A final 1,000-game smoke benchmark completes 571,049 decisions at approximately 395,000 decisions/s, with 102 ns clone and 111 ns restore on this host. This includes native policy/observation construction, excludes model inference, tensors and JSON, and is not an XMage comparison; concurrent host load makes these timings approximate.

The final core passes **1,393 Rust tests** (1,381 unit and twelve integration), all-target Clippy with warnings denied, formatting, generated-catalogue verification and whitespace checks. All **266 compiler/data tests** pass in 385.277 seconds, and all **166 existing Python tests** pass against the freshly rebuilt final release library in 121.392 seconds. Seventeen new native groups cover all nineteen complete casts in both seats, full static/entry/death residuals, both modes, all new flashbacks, predicate matrices including Kindred changeling metadata, source departure and target reentry, real discard/noncycling events, target-before-payment ordering, funding/timing/origin denials, live flying, hidden/revealed exact references, simultaneous entries/landfall, optional search versus zero selection, cloned replay/stale revisions, and early/late generation/life overflow rollback. Separate public acceptance adds six Python methods covering **88 controlled scenarios** (29.241 seconds) and a twice-identical legal **100-game Standard workload with 105,176 decisions and exact replays**. It witnesses all nineteen paid casts/effects, all eight activation bodies, fourteen search bodies and all three flashback bodies under both controllers. Those two acceptance files are a separate commit; final integration reruns their checks. The witnesses use actual costs, complete committed transactions and independent expected predicates and printed characteristics.

```sh
cargo test --all-targets
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release
python3 -m unittest discover -s python -p 'test_*.py'
cargo clippy --all-targets -- -D warnings
cargo fmt --check
python3 scripts/generate_catalogue.py --check
cargo run --release --bin benchmark -- 1000
```

## Integrated release evidence

Combined support is **1,552 / 5,164 Standard cards**, with **3,612 gated**, six regression bodies, **73 token factories**, one emblem and **5,257 immutable identities**. The exact nineteen-definition delta, 5,237 preserved rows, canonical recipe/ordinary-ID/color preservation, 476 retained helper arms plus five additions, and all-section 148 hash records across 67 Java files are independently rechecked against main. Fingerprint remains `04ead6ed7234c31bb805d57b3726cd5810053daefca6166f5aedc0a047f93c07`.

The integrated release passes **1,393 Rust tests** (1,381 unit and twelve integration), **266 compiler/data tests** (436.659 seconds), and **174 fresh-release Python tests** (234.879 seconds), including all six new methods/88 controlled scenarios and main's attachment acceptance. All-target Clippy, formatting, generated-catalogue and whitespace checks pass. No runtime changes follow acceptance checkpoint `33eaad8`; subsequent changes document the combined evidence.

The root legal Standard100 reproduces **105,176 exact decisions/replays**, matching both author runs. Every one of nineteen cards has positive paid-cast and effect witnesses under both controllers; all eight activation bodies, fourteen search bodies and three flashback bodies are separately witnessed under both controllers. In table order, paid cast pairs are `[[8,13],[19,14],[11,8],[6,7],[15,10],[27,17],[11,8],[13,6],[1,1],[43,44],[6,10],[35,35],[1,1],[9,9],[9,9],[5,8],[10,6],[2,1],[2,3]]`; the full activation/effect/search/flashback matrices are enforced by `examples/standard_exact_search.rs` and retained in its validation log.

Additional Standard witnesses include **270 deferred selections, 131 public revealed-hand references, 718 zero searches, 68 actual Hand discards, 20 post-search life-three tails, 87 post-search Chocobos, 21 death draws, one tapped Cave return, Glimpse mode counts 18/1, optional-accept counts 17/27 and 3,663 illegal-action rejections**. Controlled acceptance additionally covers both Cave-return seats, optional declines and exact old/new-token landfall ordering. The adjacent attachment Standard100 retains all 74,702 decisions/replays and its entire prior vector.

Initial logical state remains **19,944 bytes**, and Standard100 reaches **27,544 bytes**. The selected-ID/fixed-filter memory fixture remains unchanged. A combined 1,000-game smoke run retains 571,049 decisions at approximately 226,000 decisions/s with 190/193 ns clone/restore under concurrent host load. These measurements retain the exclusions above and are not an XMage performance comparison.
