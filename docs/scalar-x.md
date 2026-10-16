# Scalar X declarations and complete numeric domains

Issue [#224](https://github.com/altf4/rustmage/issues/224) adds twelve complete Standard bodies to the in-process library. The integrated checkpoint supports **1,599 / 5,164 Standard cards**, 1,605 total implemented definitions, 74 immutable token factories and one emblem. Its catalogue fingerprint is `34a70c2fe2060ab6c58b9abb6075e834d1c084a5d96808ccd853429d8e2f1e54`. The [manifest](scalar-x-admissions.json) records every complete pinned Oracle text, **75 source records across 50 Java files** for constructors and direct rules-helper dependencies at XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

| Card | Complete behavior |
| --- | --- |
| Choco-Comet | X damage to any target, then the existing complete 2/2 green Bird landfall token. |
| Mind Spring | Draw X cards. |
| Pedal to the Metal | Target creature gets +X/+0 and first strike until cleanup. |
| Procrastinate | Tap the target and add twice X stun counters. |
| Slice from the Shadows | Cannot be countered; target gets -X/-X until cleanup. |
| Song of Totentanz | X black 1/1 Rats that cannot block; all controlled creatures gain haste until cleanup. |
| Spectral Denial | Reduce generic cost by the live count of controlled creatures with power at least four; counter unless its controller pays X. |
| Syncopate | Counter unless its controller pays X; exile only an actually countered spell. |
| Traumatic Critique | X damage to any target, then draw two and discard one. |
| United Front | Create X white 1/1 Allies, then put a +1/+1 counter on every controlled creature, including the new tokens. |
| Morlun, Devourer of Spiders | Legendary 2/1 with lifelink; enter with X counters; exact own entry trigger deals X to an opponent. |
| Pterafractyl | Flying 1/0; enter with X counters and gain two life from its entry trigger. X=0 dies normally and still queues that trigger. |

## Declaration API

Schema **64** adds `Action::ChooseX { value: u32 }`, decision `x`, and `SpellView.x: Option<u32>`. The actor's `Observation.x_choice` and native `LegalActionSpace.x_choice` expose an exact object reference and sorted, disjoint inclusive intervals. Other viewers receive no numeric choice or actions. An example disconnected domain is:

```json
{"card":{"card":17,"generation":3},"intervals":[{"minimum":0,"maximum":0},{"minimum":2,"maximum":2}]}
```

X is declared before targets and cost payment. `None` is an unchosen scalar declaration; `Some(0)` is a real zero choice. Normal Hand casting and authorized exact exile casting both use the staged announcement. Direct `Cast`/`BeginCast` cannot bypass the declaration. The existing C ABI remains unchanged. Python submits any concrete represented integer using `game.act(obs, {"kind":"ChooseX", "value":65})`.

`legal_actions()` is exhaustive for an interval at most 64 wide and otherwise emits its minimum, midpoint and maximum. These indexed representatives are conveniences for policies. A concrete `ChooseX` accepts every integer in the complete domain, including values absent from the representative list. Querying and choosing do not enumerate a numeric domain proportional to X.

## Cost domains and contextual mana value

With The Scarlet Witch at printed power two and exactly UU available, Mind Spring is affordable at **X=0 and X=2**, but not X=1: X=1 has mana value three and receives no discount; X=2 has mana value four and receives the two-generic reduction. A single zero-to-maximum range would admit an illegal choice.

The domain solver partitions the supported u32 range at every recursive immutable mana-value threshold used by eligible own/static reducers. Within a segment, those predicates and the proposed spell's fixed targets/base cost are constant. The exact shared funding solver includes floating mana, untapped sources and supported virtual activation routes. Generic requirements increase monotonically within that segment, so a binary search finds its last payable value using at most 33 funding queries (one lower-bound check plus at most 32 binary-search checks). Adjacent payable segments merge; disconnected ones remain separate. Work depends on reducer thresholds and real funding routes, rather than the magnitude of X. For a fixed segment, a concrete successful funding route for a larger generic requirement also funds a smaller one: its colored requirement and ability costs are identical, its sacrifices/conversions remain legal, and excess generated mana may remain unused. The cost is evaluated before that route and remains locked while resources change. Native tests independently exercise both color conversion and actual sacrificial Treasure funding for the disconnected domain.

The compiler admits one fixed X symbol, fixed colored costs and fixed target arity. All own reduction combinations except Spectral Denial's exact one-per-power-four creature rule remain gated. Hybrid, multiple X symbols, flashback/offspring/kicker, additional costs and target-dependent discounts combined with these bodies remain gated. Global supported battlefield reducers still apply. They cannot change target completion or introduce a new scalar-dependent nonmana cost in this closed scope. Costs lock after targets and before mana abilities/payment, so later source departures or virtual funding changes do not reprice the spell.

Raw X plus printed generic and accumulated discounts use widened u64 arithmetic **before** subtracting and narrowing payable generic to u32. A legal discount may bring a raw total above u32::MAX back into range. Scalar declarations are explicitly bounded to u32; counts, object generations and other capacity bounds retain their existing independent checked representations. Unsupported JSON negative/fractional/out-of-range scalars fail without changing state. Late capacity failures restore the entire pre-action state, scalar declaration, costs, mana, queued packets, revision and RNG; earlier completed actions remain committed.

`SpellView::mana_value()` uses the original printed fixed mana value plus the selected X only for an actual spell. Existing stack-counter target predicates, recursive cast-trigger filters and proposed-spell reduction predicates use this contextual value. Discounts and payment amounts never change mana value. Off-stack cards and trigger packets retain printed X=0 mana value. A permanent stores no historical X field.

## Entry provenance, effects and rollback

The resolving spell transfers X through its exact entry event. Entry counters are a replacement applied before entry observers and state-based actions. The exact own entry-trigger packet captures X independently of `CastInfo`; trigger packets do not masquerade as spells. Morlun's queued damage retains its chosen amount if the original permanent leaves or a later incarnation reenters. Ordinary reanimation does not inherit the old spell's X. Source keywords and target incarnations continue to use their existing captured/live rules.

Scaled instructions share ordinary damage, draw, token, counter, temporary modifier and resolution-payment paths. Counter payments can actually pay zero or decline zero; declining a legal zero payment can still result in a counter. Syncopate exiles only an actual counter. An uncounterable target remains on the stack. If every target becomes illegal, Choco-Comet's entire token tail is skipped. Temporary growth/keywords expire through ordinary cleanup, and the Bird's later landfall is its complete existing recipe.

All numeric choices and suspended draw/discard/payment/entry frames belong to owned clones and snapshots. Each public acceptance action compares an independent clone, restores an owned snapshot, rejects the stale revision, reapplies the exact action and compares the resulting state/observation.

## Standalone validation and reproduction

The following counts and measurements describe the original scalar branch before integration with explore and bargain.

The thirteen native scalar regression groups cover complete bodies at X=0/3 in both seats, exact disconnected domains, exhaustive small-domain comparisons with real mana sources and reducer thresholds, a u32-wide domain with a concrete unlisted value, contextual old counter/cast predicates, locked discounts, actual zero and positive resolution payments, exact entry provenance across departure/reentry, whole-target fizzle, uncounterable interaction, and late capacity atomicity.

Public [Python tests](../python/test_scalar_x.py) exercise actual paid declarations/effects in both seats, actor-private intervals, illegal actors/values, JSON bounds and legal zero-payment continuations. The [Standard ledger](../examples/standard_scalar_x.rs) uses legal 60-card/four-copy decks, positive paid casts and complete effect vectors for all twelve cards in both controllers, exact entry packets, domain bounds, and per-action clone/restore/stale replay.

```sh
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
python3 -m unittest discover -s scripts
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py'
cargo run --release --example standard_scalar_x -- 100
```

Final validation passes **1,410 native library tests** plus every integration/example target, **272 compiler/inventory tests** (261 catalogue methods and all eleven other scripts methods), **179 fresh public Python tests**, formatting and all-target clippy with warnings denied. The Python run uses an immutable finished release library.

The deterministic Standard100 ledger records **192,778 decisions/replays**, 617 exact scalar domains and 1,234 illegal-action rejections. Initial logical state is **19,976 bytes**, an increase of 32 bytes over this lane's 19,944-byte base; observed maximum is **23,536 bytes**. X is one bounded scalar in each spell/entry continuation; no per-value objects or action arrays are retained. Numeric domains allocate only transient interval/representative data. Actual created tokens and real triggered packets still consume memory proportionally to actual game objects/events.

Rules basis: [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf), CR 107.3a/b/g/m, 202.3e, 400.7d and 601.2b/c/f/g/h. These distinguish declaration from payment, off-stack X=0, exact own entry information, target ordering and locked total costs.

## Integrated validation

The integrated library contains the merged explore and bargain implementations alongside scalar X. It passes 1,452 Rust tests (1,440 unit and twelve integration tests), 190 fresh-library Python tests, 282 final compiler/inventory tests (337.926 seconds), all-target Clippy with warnings denied, formatting and catalogue regeneration checks. The final compiler hardening rejects malformed fixed bargain costs and overflowing scalar fixed bases; reciprocal X/bargain composites remain gated. Both independent reviews and their exact tested checkpoints are recorded in [the PR review](review-pr-228.md).

The integrated Standard100 X workload reproduces all standalone vectors and 192,778 exact decisions/replays, 617 domains and 1,234 illegal rejections. Adjacent bargain and explore workloads preserve their exact 85,454 and 215,656-decision ledgers. Initial state remains 19,976 logical bytes relative to the merged bargain base; the observed X-workload maximum is 23,536 bytes. A final fresh release after fingerprint-only regeneration also passes all three public scalar methods.

A contended native 1,000-game smoke benchmark recorded 571,049 decisions at about 204,275 decisions/second, with 325 ns clone and 289 ns restore averages. These include smoke-policy and observation construction, exclude model/tensor/JSON work, cover the admitted catalogue, and provide no Java/XMage speed comparison.
