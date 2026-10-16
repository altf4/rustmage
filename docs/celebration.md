# Celebration and historical nonland entries

Issue [#229](https://github.com/altf4/rustmage/issues/229) adds **ten complete Standard bodies**. This standalone stage executes **1,609 / 5,164 Standard identities**, plus six regression cards, with 74 immutable token factories and one emblem. Its fingerprint is `39c59f51d2ded342b767fa381c8f25f786cd016af2734133c89aec6f3c6e6681`.

| Complete card | Behavior |
| --- | --- |
| Armory Mice | 3/1; celebration gives +0/+2. |
| Ash, Party Crasher | Legendary 2/2, haste; celebration attack trigger adds a counter to the exact source. |
| Belligerent of the Ball | 3/3; own combat celebration gives an owned target +1/+0 and menace until cleanup. |
| Bespoke Battlegarb | Equipment gives +2/+0, equip {2}; own combat celebration may attach it to up to one owned creature. |
| Gallant Pie-Wielder | 2/3, first strike; celebration grants double strike. |
| Grand Ball Guest | 2/2; celebration gives +1/+1 and trample. |
| Lady of Laughter | 4/5, flying; own end-step celebration draws one. |
| Pests of Honor | 2/2; own combat celebration adds a counter to the exact source. |
| Raging Battle Mouse | 2/1; the second spell each turn costs {1} less, plus own combat celebration gives an owned target +1/+1 until cleanup. |
| Tuinvale Guide | 2/3, flying; celebration gives +1/+0 and lifelink. |

The [manifest](celebration-admissions.json) records **69 SHA-256 records across 41 Java files**, including all ten full Oracle bodies/constructors, their direct helpers, `CelebrationCondition` and `PermanentsEnteredBattlefieldWatcher`, at pin `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Goddric, Cloaked Reveler remains gated: its Dragon type/base-stat/ability replacement and granted activation are not implemented. Unknown residuals and unimplemented printed costs reject the whole card.

## Rules, timing and API

Celebration is an ability word describing the exact condition that **two or more nonland permanents entered under the controller's control this turn**. It counts historical entries, including creature and noncreature tokens, and includes the payoff source itself. A permanent leaving does not remove that entry. A returning incarnation counts again. Lands never count; drawing a land or putting it into Hand through exploration does not enter it onto the battlefield. Entries before the payoff source arrived still count. Current admissions retain the engine's explicit owner-equals-controller scope; control-changing bodies remain gated.

Schema **65** adds public `PlayerView.nonland_permanents_entered_this_turn: u64`, identically visible to both viewers. The actual zone-transition and token-creation paths update the shared checked count. Every member of a simultaneous batch exists and its entry is counted before entry observers inspect the resulting battlefield. Speculative affordability queries, cloning, observation, replacement preflight and failed actions never increment the committed history.

Both players' counts reset at the beginning of each new turn. They remain intact during cleanup and any additional priority cycle required by CR514.3a. Temporary celebration buffs and menace expire through normal cleanup; static celebration stats and keywords continue while the historical condition remains true, until the next turn resets it.

A shared immutable `StatValue::NonlandEntriesThisTurn` feeds existing stat-scaling and keyword conditions, including the borrowed virtual funding evaluator. Static effects update immediately at the second entry. Triggered effects use the existing typed `StepGuard::Expression` and intervening-if machinery: the condition must hold at the event and again at resolution. An entry after the event cannot retroactively create a trigger. Trigger source and target references retain exact incarnations; departing and reentering an Equipment or target does not attach to a replacement object. An already queued target buff may resolve after its source leaves if its target remains legal and its condition still holds.

Bespoke's optional target really permits zero; that choice does not detach or move Equipment. Its ordinary equip ability retains sorcery timing, full {2} payment, owned-creature targeting and normal attachment legality. Raging retains its complete second-spell reduction, including colored requirements, selected-X contextual mana value and the existing total-cost lock before mana payment. It does not discount the first or third spell.

Rules basis: [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf), ability words in CR207.2c, intervening-if CR603.4, independent triggered abilities CR113.7a, new objects CR400.7 and cleanup CR514.3. The pinned Java watcher records copied entry facts under their entry controller; the Rust representation stores the equivalent bounded historical count because mutable types/control are outside admitted scope.

## Snapshot ownership, efficiency and preservation

The implementation adds **two fixed u64 counters**, with no per-object flag, entry journal, mutable catalogue or source graph. Earlier-layer static/funding evaluation reads the same historical counter in constant time. Measured initial logical state is **20,008 bytes**, versus the exact parent's 19,976: **+32 bytes from enclosing layout**, rather than assuming two field widths equal the layout delta. Logical measurements exclude spare vector capacity and allocator overhead. The Standard workload reaches 22,628 bytes.

Snapshots and clones own both counts and all suspended trigger/target/payment frames. A late failure on the second token entry restores the first token, history, object arena, queued events, RNG and revision. Earlier successful actions remain committed. Exact stale indexes, wrong actors and illegal targets cannot mutate the state.

Canonical comparison with parent `0e0f181f789a85b665af833c7c4087ec82ddd372` proves exactly **10 changed / 5,248 unchanged definitions**, all **74 prior factory recipes/metadata preserved**, all **5,183 ordinary IDs and old colors stable**, and all **517 prior numeric helper arms preserved**, with eight new arms. Factory numeric positions remain catalogue-scoped.

## Validation and reproduction

Sixteen focused native groups cover all ten actual paid bodies in both seats; 0/1/2 history; simultaneous creature/noncreature tokens; source as second or later entry; departures/reentries; lands and actual Map exploration; event-versus-resolution checks; exact attachment incarnations; optional zero and paid equip; real double-strike/lifelink combat; shared virtual keywords; retained second-spell reductions with an actual selected-X spell; cleanup priority/reset and late overflow rollback. Four compiler methods enforce the complete bodies, exact historical qualifier, bounded costs, optional attachment grammar, whole equip/reducer residuals and unknown-tail rejection. The exhaustive scaling-source inventory explicitly includes the three newly admitted scaling sources.

The fresh public module has **two methods / 42 paid scenarios**: every card under both controllers at both false and true thresholds, plus both controllers declining optional attachment and then really paying equip. It checks both viewers, full clone/snapshot/restore/stale replay, illegal opponent targets and wrong actors. Lady's negative case reaches its actual end step; Ash's negative case makes an actual attack.

Two deterministic legal Standard100 runs reproduce **134,837 exact decisions/replays** and all printed witness arrays. Every card has positive paid-cast and full-effect witnesses under each controller. Independent per-action accounting checks 4,133 actual nonland entries (1,494 tokens), 3,581 land entries, unchanged history on departures and both-player reset boundaries. The ledger includes 21 actual second-spell discounts, eleven paid equip resolutions, 99 optional zero-target choices and 769 real attacks. Both snapshot ownership and stale-index rejection are checked after every action.

A concurrent 1,000-game native smoke benchmark measured 571,049 decisions at about **173,000 decisions/s**, 344 ns clone and 387 ns restore; it independently measured the 20,008-byte initial state. The workload includes native policy and observation construction, excludes neural inference/tensors/JSON and makes no Java performance comparison.

```sh
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
python3 scripts/generate_catalogue.py --check
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo build --release
python3 -m unittest discover -s python -p 'test_*.py'
cargo run --release --example standard_celebration -- 100
cargo run --release --bin benchmark -- 1000
```
