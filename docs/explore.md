# Explore and the complete Map token

Issue [#222](https://github.com/altf4/rustmage/issues/222) adds **15 complete Standard bodies**. The catalogue now executes **1,577 / 5,164 Standard cards**, plus six regression cards, with 74 immutable token factories and one emblem. Its fingerprint is `e6e347ae7e1520091f4a62d0b80f004413cf1ab2741b266f4ddf8bbde6ffe879`.

| Complete body | Behavior |
| --- | --- |
| Cartographer's Companion | Artifact creature, 2/1; entry creates a Map. |
| Cenote Scout | 1/1; entry explores. |
| Fanatical Offering | Sacrifice an artifact or creature as an additional cost; draw two and create a Map. |
| Get Lost | Destroy a creature, enchantment or planeswalker; its captured controller creates two Maps. |
| Kinjalli's Dawnrunner | 1/1, double strike; entry explores. |
| Miner's Guidewing | 1/1, flying and vigilance; death targets a controlled creature to explore. |
| Over the Edge | Destroy an artifact or enchantment, or a controlled target explores twice sequentially. |
| Pathfinding Axejaw | 4/3; entry explores. |
| River Herald Guide | 3/1, vigilance; entry explores. |
| River Herald Scout | 1/2; entry explores. |
| Seeker of Sunlight | 1/1; {2}{G} explores, only as a sorcery. |
| Sentinel of the Nameless City | 3/4, vigilance; entry or attack creates a Map. |
| Spyglass Siren | 1/1, flying; entry creates a Map. |
| Waterwind Scout | 2/2, flying; entry creates a Map. |
| Merfolk Cave-Diver | 2/4; each completed exploration by a controlled creature gives it +1/+0 and makes it unblockable this turn. |

The [provenance manifest](explore-admissions.json) records full Oracle bodies, constructors and supporting code: **92 SHA-256 records across 55 Java files**, verified against XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Full-body admission remains closed. Amalia Benavides Aguirre needs its power-qualified destruction; Nicanzil needs result-qualified observers and optional land entry; Defossilize needs binding to the reanimated incarnation; Brackish Blunder needs pre-bounce tapped history. They remain gated. Unknown residual text, arbitrary copying, animation, crew and variable costs remain unsupported where needed.

## Rules and the public contract

[Comprehensive Rules 701.44](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf) determine the sequence: reveal the top card; put a land into hand, or put a +1/+1 counter on the explorer before choosing whether to put a nonland into the graveyard. An empty library still adds the counter and completes exploration. A departed source explores using captured identity and characteristics; a new incarnation receives no counter. Targeted exploration still requires a legal target when resolution begins. Two explores finish sequentially; queued triggers wait for the surrounding resolution to complete.

Observation schema **62** exposes `explore_choice` to both viewers, with `explorer`, `revealed` and `counter_added`. The explorer carries its exact object reference, immutable card and controller. The revealed card carries its exact reference and card identity. Only the controller may submit `ChooseExplore { object, graveyard }` at decision `explore`; the reference must exactly match the revealed card. The counter has already committed before this choice. Keeping the card preserves its Library reference; moving it creates the ordinary new Graveyard incarnation.

`last_explore` is the latest historical completed event: `sequence`, `explorer`, optional `revealed`, optional `destination`, optional resulting exact `result`, and `counter_added`. A land goes directly to Hand, with public known-hand identity and a new reference; this is not a draw. Empty exploration has no revealed card, destination or result. The sequence increments for every completion, including empty exploration. One action can complete multiple automatic explorations; this field retains the latest event rather than an unbounded history. Consumers requiring every event must observe each action and account for sequence changes and public Hand movements.

`known_library_top[owner]` separately records current public top knowledge. A kept explored nonland remains known. Drawing a publicly known top transfers its identity to public known-hand information. Private rearrangement, uncertain insertion and shuffling invalidate position knowledge before private selection can disclose an outcome. Historical events remain historical through later draws and rearrangements. No unrelated library order becomes public.

Map is the complete colorless `Artifact — Map` recipe: **{1}, {T}, sacrifice this artifact: target creature you control explores; activate only as a sorcery.** Its target may be tapped. Ordinary timing, funding, target incarnation and cost enforcement apply; the sacrificed token ceases through normal state-based actions. Every Map provider uses this same immutable factory.

Get Lost captures the valid target's controller before destruction. Indestructibility prevents destruction but does not prevent the two Maps. If its sole target becomes illegal, the spell fizzles and creates no Maps. Cave-Diver listens to genuine completed exploration, including empty-library and departed-source exploration; counters alone never manufacture this event. Its temporary power and unblockability use existing exact-source cleanup machinery. Entry-trigger suppression applies to entry explorers and Map providers while leaving attack and death triggers intact.

## Ownership, rollback and size

An optional boxed `ExploreState` holds latest-event data, top knowledge and an indexed observer list. It allocates only when needed, without a per-object trait graph or an explore marker on every object. The structure is **176 bytes**, each indexed source contributes four bytes, and an active boxed public choice contributes **56 bytes**. Logical accounting includes these allocations and excludes spare capacity and allocator overhead. Initialized state remains **19,944 bytes**, measured against the exact parent; layout padding absorbs the added optional pointers. The explore Standard workload reaches **21,580 logical bytes**.

Game clones and snapshots own exploration, knowledge, counters, observer indexes, continuations and RNG. A rejected action restores every change made by that action, including late counter/event/token allocation failures. Earlier completed actions, such as a counter committed before a later choice, remain committed. Wrong actors, stale indexes and stale exact references cannot advance the continuation.

Canonical comparison against parent `7704e0f0bba799c465bf5dd8c44c93970fcaa10a` proves **15 changed / 5,242 unchanged definitions**, one new Map factory, all **73 old factory recipes and metadata preserved**, all **5,183 ordinary card IDs and old color masks stable**, and all **493 old numeric helper arms preserved**, with one additional mandatory-cost arm. Factory numeric positions remain catalogue-scoped; canonical recipe identity is the preservation contract.

## Validation

Sixteen new native groups cover every complete body and both seats; land, nonland and empty branches; counter-before-choice; public knowledge and private invalidation; source departure/reentry; exact targets; full Map costs and restrictions; Get Lost indestructibility and fizzle; sequential exploration and observer ordering; suppression, cleanup and rollback. Full Rust passes **1,425 tests**. The full compiler/data suite passes **274 tests** (428.522 seconds in this concurrent run). Four new compiler methods enforce closed grammar, full Map behavior, Get Lost's single controller-capturing operation and full-body rejection. Formatting, all-target Clippy and generated-catalogue checks pass.

The independently authored public Python module exercises **62 paid scenarios in nine methods**, including actual opposing indestructibility, source departure, empty libraries, known-top draws, all Map providers, both modal branches, vigilance attacks and illegal blocks followed by cleanup. Its separate acceptance commit is integrated by the coordinator.

`cargo run --release --example standard_explore -- 100` plays legal Standard decks and records actual paid casts and full effects for all fifteen cards under both controllers. Two runs reproduce the same full witness arrays. The workload completes **215,656 exact decisions and owned replay boundaries**, with **2,958 completed explorations: 1,195 land puts, 459 graveyard choices, 1,229 kept tops and 75 empty explorations**. It observes 1,688 public choice frames, rejects 3,376 invalid actions and verifies every Map provider's tokens through actual paid activation in both seats. Over the Edge's two mode/controller vectors are `[[15,10],[12,18]]`; Sentinel makes 64 actual vigilance attacks; Cave-Diver has 39 buff witnesses and 13 actual unblockability frames, with cleanup checked once per observed buff instance. Each successful action checks both viewers and clone/restore; printed cast/effect/provider arrays make the ledger independently inspectable.

A concurrent 1,000-game native smoke benchmark measured 571,049 decisions at about **196,000 decisions/s**, with 358 ns clone and 316 ns restore, and independently confirmed the 19,944-byte initial state. It includes native policy and observation construction, excludes inference, tensors and JSON, and supplies no Java performance comparison. Timings depend on host contention.

Reproduce with:

```sh
cargo test --all-targets
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release
python3 -m unittest discover -s python -p 'test_*.py'
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
cargo run --release --example standard_explore -- 100
cargo run --release --bin benchmark -- 1000
```
