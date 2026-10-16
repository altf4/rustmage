# Fixed kicker and exact paid-entry qualification

Issue [#218](https://github.com/altf4/rustmage/issues/218) adds ten complete Standard bodies to the in-process library. The catalogue supports **1,543 / 5,164 Standard cards**, with 3,621 gated identities, 72 immutable token factories and one emblem. The fingerprint is `b2d22840da19c23aa4b1cbef02cb99b9b4e4a98cefc6bf5cb2247e3b724ef0ba`.

| Card | Kicker | Complete behavior |
| --- | --- | --- |
| Burst Lightning | {4} | One operation deals 2 or 4 damage to any legal target. |
| Firebending Lesson | {4} | One operation deals 2 or 5 damage to a creature. |
| Gatekeeper of Malakir | {B} | Kicked ETB targets a player, who chooses a creature to sacrifice. |
| Grow from the Ashes | {2} | One private basic-land search selects up to one or two cards; normal battlefield entry and one shuffle. |
| Into the Roil; Whoosh! | {1}{U} | Bounce a nonland permanent; draw one afterward only when kicked and at least one target remains legal. |
| Nullpriest of Oblivion | {3}{B} | Lifelink, menace; kicked ETB reanimates a creature from your graveyard. |
| Sun-Blessed Healer | {1}{W} | Lifelink; kicked ETB reanimates a nonland permanent with mana value at most two from your graveyard. |
| Zuko's Conviction | {4} | One exact graveyard target returns to hand or, when kicked, directly to the battlefield tapped. |
| Gnarlid Colony | {2}{G} | Kicked entry replaces the event with two +1/+1 counters; all controlled creatures with an actual +1/+1 counter have trample while Colony remains. |

The [provenance manifest](fixed-kicker-admissions.json) records all complete snapshot Oracle texts and **116 SHA-256 records across 63 Java files** at XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. It includes admitted constructors, their helpers, shared casting/entry code and the three explicit exclusions. Jet's Brainwashing, Rite of Replication and The Eagles Are Coming! remain gated for their unimplemented control, arbitrary copying and delayed effects. Multikicker, variable costs, multiple kicker declarations and unknown residual clauses remain rejected. Combined offspring/kicker bodies remain gated until their immutable copy recipes preserve every kicker and counter-grant ability.

## Announcement and payment

Observation schema **60** adds `Action::ChooseKicker { pay }`, decision `kicker`, and `CastInfo.kicker: Option<bool>`. `None` means no kicker recipe or an unchosen declaration; `Some(false)` and `Some(true)` record declining and paying. Casting method, origin and offspring retain separate fields. The C ABI and control protocol remain unchanged.

The declaration precedes targets and resources. Only continuations fundable from the complete selected cost are exposed. The fixed typed surcharge combines with the selected normal/alternative base and other typed surcharges before generic reductions; it never changes printed mana value. Targets complete before the final total locks. Later mana abilities and reducer departures cannot alter that locked total. The shared target-dependent funding calculation includes kicker. Direct casting cannot bypass the choice; Hand, exact exile permissions and flashback authorization retain their existing restrictions. None of these ten cards has its own flashback ability.

The announcement, stack and suspension frames belong to owned snapshots. Wrong actors, unsupported origins, stale indexes, invalid targets, insufficient colored resources and over-capacity operations leave the complete state unchanged. The public API uses ordinary clone/restore; no client/server round trip is required.

## Entry qualification and complete effects

Only the paid spell's exact staged entry receives the historical kicked qualification. An ordinary reanimation or later incarnation does not inherit it. Kicked ETBs queue from that event, and retain their historical truth if the source subsequently leaves or reenters; current target legality still determines resolution. Unkicked ETBs do not trigger or request targets. Countering the parent prevents its entry and every associated ETB.

Gnarlid's counters are captured before entry choices and committed as an **entry replacement**. They apply under Torpor Orb and Doorkeeper Thrull, before power-qualified entry observers inspect the creature. The trample grant is an immutable counter-qualified recipient predicate, evaluated live for controlled creatures, including Colony itself. Counters, source departure and reentry update it immediately. The shared virtual funding characteristics see the same grant.

The spell replacement branches select one actual damage, search or movement operation. Grow does not run two sequential one-land searches. Zuko does not move the card through its owner's hand on the kicked branch. Bounce uses normal legal-resolution rules before its optional draw. Searches retain all selections privately in the library until the explicit commit; a qualified search may legally select fewer cards or none. Commit stages the complete entry batch, then shuffles with the owned RNG.

## Validation and reproduction

```sh
cargo test --all-targets
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release --lib
PYTHONPATH=python python3 -m unittest discover -s python -p 'test_*.py'
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
python3 scripts/generate_catalogue.py --check
cargo run --release --example standard_fixed_kicker -- 100
cargo run --release --bin benchmark -- 1000
```

The standalone suites contain **1,392 native tests, 266 compiler tests and 170 fresh-release Python tests**. Sixteen new native groups cover all ten bodies, both branches and seats, typed costs and joint funding, combined reductions and locked totals, source/target reincarnations, suppression/replacements, live grants, graveyard restrictions, real exile permissions, countering, qualified private searches and fail-to-find. Early stack allocation, a late bounce-followed-by-draw overflow and a second-land trigger allocation failure prove full rollback, including private selections and RNG. Four compiler methods enforce complete-body grammar, exact single-operation branches and unknown-residual rejection. Two Python acceptance methods exercise forty complete paid/declined card/controller scenarios plus two actual paid Alania impulse-exile/kicked Burst Lightning scenarios, with private selections and every-action clone/restore/stale-index rejection. Existing public tests use the additive schema and casting metadata.

Canonical comparison against parent `edea3fd97dc5f9e308a2db9459a1acd4123b32c2` proves exactly **ten changed / 5,246 unchanged definitions**, all 5,256 identity positions and colors preserved, all factories preserved, and **476 existing numeric helper arms unchanged**. Three new helpers contain twelve numeric arms for fixed costs, kicked-entry counters and counter-qualified grants.

Two legal 100-game Standard runs reproduce **102,219 decisions and owned replay boundaries**. Every card has positive cast and completed-effect witnesses in all four kicked/unkicked × controller cells. Independent public checks verify 36 searches, 53 normal-entry lands, 89 private frames, 153 live counter grants, 14 sacrifices, 40 reanimations, 72 bounces, 37 additional draws, three all-invalid fizzles and 545 rejected actions. Both arrays are printed in manifest order for review.

Initialized logical state remains **19,944 bytes**, excluding spare capacity and allocator overhead; the Standard workload reaches **22,992 bytes**. Kicker adds no per-object field: cast/entry event context owns its qualification. The 1,000-game native smoke benchmark completed 571,049 decisions at about 191,000 decisions/s, with roughly 263 ns clone and 371 ns restore in a concurrent validation run. These timings include native observations and policy, exclude inference/JSON/tensors, and do not claim XMage parity.
