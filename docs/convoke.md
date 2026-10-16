# Intrinsic Convoke and complete adjacent Equipment

Issue [#233](https://github.com/altf4/rustmage/issues/233) adds nine complete intrinsic Convoke cards and the adjacent Bespoke Bō Equipment. The integrated catalogue has **1,648 executable Standard cards**, six regression cards, **77 immutable token recipes** and one emblem. It remains a bounded migration: unknown whole-card residuals reject the body.

The pinned [admission manifest](convoke-admissions.json) records every full Oracle body, primary Java constructor and relevant helper at XMage `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All sections, including the gated neighbors and shared payment/control helpers, contain **88 hash records / 56 unique pinned Java sources**. The primary rules basis is [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf), CR702.51a–d, 601.2f–i, 302.6, 400.7, 506.4b and 603.3. The Java helper's historical “additional cost/reduction” Javadoc and fabricated pool-mana implementation are not the Rust rules model.

## Complete bodies

| Card | Complete behavior |
|---|---|
| Appeal to Eirdu | Instant; exactly one or two distinct creatures of either controller, each +2/+1 until end of turn; independent target-incarnation revalidation. |
| Disruptor of Currents | Flash 3/3 Merfolk Wizard; optional zero or one **other** nonland permanent of either controller returns to its owner's hand. |
| Lofty Dreams | Creature Aura; +2/+2 and flying; actual entry draw; ordinary target legality, attachment and departure cleanup. |
| Protective Response | Instant; destroy an attacking or blocking creature, with live combat membership checked again at resolution. |
| Sun-Dappled Celebrant | 5/6 Treefolk Cleric with vigilance. |
| Unexpected Assistance | Instant; draw three, then one real private discard in the existing owned continuation. |
| Vote Out | Sorcery; destroy a creature, retaining timing and ordinary indestructible/fizzle rules. |
| Winter, Team Player | Legendary 3/3 Human Warrior; each own noncreature spell actually cast queues the full +1/+0-to-own-creatures effect until cleanup. A countered spell still caused the cast event, and the packet survives Winter's departure. |
| Merrow Skyswimmer | All three printed hybrid equivalents; 2/2 with flying and vigilance; entry creates exactly one white-and-blue 1/1 Merfolk creature token with no added abilities. |
| Bespoke Bō (adjacent) | Optional zero or one **other**, unrestricted-controller nonland return on entry; equipped creature +2/+1 and vigilance; real sorcery-speed equip {3}, own-creature targets and normal attachment cleanup. This card has no Convoke flag. |

The shared exact target compiler now represents `(Creature, 1, 2)` and `(OtherNonland, 0, 1)`. Both minimum and maximum survive announcement and resolution. “Other” compares the exact source object, rather than excluding a physical card forever. Bō's complete existing Equipment path becomes executable through the second target primitive; its admission is explicit rather than an unnoticed grammar side effect.

The Wandering Rescuer remains gated because its tapped-recipient hexproof grant needs a complete typed predicate in actual and virtual keyword layers. Temporal Cleansing remains gated because second-from-top-or-bottom is a different destination contract. Chosen-type bodies, arbitrary copy, Rooms/transform and effects granting Convoke to other spells remain unsupported. Fixed intrinsic Convoke is kept closed against X, kicker, offspring, bargain, collect evidence, cycling, additional costs, target-dependent reductions and Saddle compositions. Existing board-wide ordinary reductions still apply before payment and stay locked.

## Funding and owned payment

Convoke replaces an individual colored or generic mana payment after the spell's total cost is locked. It produces no mana, cannot pay required {C}, and does not alter printed or on-stack mana value. One untapped creature currently controlled by the caster pays either one matching W/U/B/R/G requirement or one generic requirement. A multicolored creature chooses exactly one. A colorless creature pays generic only. These are non-targeting tap costs: summoning sickness and hexproof do not prevent them, and tapping a blocker or untapped attacker does not remove it from combat.

Casting retains the ordinary staged flow: announce, select the exact hybrid equivalent and targets, lock cost, then activate real mana abilities. `BeginConvoke` closes that mana-ability window and opens an actor-private `convoke_choice`. `ChooseConvoker { object, contribution }` reserves an exact incarnation and one typed credit. `PayCosts { payment }` pays the remaining real mana and commits all reserved taps as one transaction. Direct full-mana `PayCosts` remains legal; an empty Convoke frame can also finish with full real mana. Once no payable component remains, extra contribution choices disappear. No mana action or reopening is offered after `BeginConvoke`.

Selected creatures remain untapped until that final payment. Both viewers see the actual simultaneous taps after commit; only the actor sees selected reservations and remaining cost beforehand. Original targets and locked total remain owned by the announcement. A late capacity failure restores the frame, pool, creature taps, queued triggers, RNG and revision. Earlier successfully activated mana abilities remain committed when branching from a later checkpoint. A selected source that leaves and returns is a different object and cannot satisfy the old reservation.

Funding integrates those exact contributions with the existing virtual pure-mana graph. Tapped or sacrificed virtual mana creatures cannot also contribute. Untapped sick creatures can contribute even when their {T} mana ability cannot activate. A required colorless symbol always needs real mana. Ordinary activated abilities and related Ward/resolution payment frames never gain Convoke credits.

The terminal feasibility calculation uses **32 five-color masks and Hall's 31 subset inequalities**, followed by the generic-capacity check. Real colored mana first covers matching colored requirements; any feasible allocation can exchange those payments without losing a creature assignment. Hall then matches the uncovered colored requirements to distinct available creatures. Unassigned creatures and remaining real mana can pay generic. A five-bit subset transform counts the eligible unions in fixed work. No creature subset search or enumeration of individual numeric mana units is introduced. The virtual search's optimistic bound can double-count a prospective mana creature only as an upper bound; the exact terminal uses its actual counterfactual tapped/present status.

All printed mana requirements and actual pool components retain the existing u32 bounds. Deficit arithmetic uses u128/u64 intermediates; the CardId arena bounds the number of mask terms, so aggregate counts do not wrap. Counterfactual pools retain the existing widened funding representation. Actual mana-source overflow reports the existing atomic capacity error rather than truncating or fabricating a continuation. Unknown and oversized printed symbols remain whole-body gated.

## State size and preservation

No object/permanent field, per-turn Convoke journal or mutable catalogue is added. The only persistent extra payload belongs to an active spell announcement: an optional box with remaining fixed cost and selected `(ObjectRef, contribution)` pairs. Logical initialized bytes include that box and each initialized selection; allocator overhead and spare capacity are excluded consistently with existing measurements. Empty/frame and per-selection byte deltas are asserted by native tests.

The measured 60-card-per-player initial state is **20,088 bytes**, versus the exact Saddle parent's **20,056**: **+32 bytes from enclosing layout**. Standard100 reaches **21,376 bytes**. Clone and snapshot ownership includes suspended contribution choices and queued tap/cast packets.

Canonical comparison to parent `9539958e909267260b9dd9f7532aef891be618cb` proves **10 changed / 5,250 unchanged definitions**, one added Merfolk factory, all **76 previous recipes** preserved, all **5,183 ordinary identities** and every prior color preserved. All **530 old numeric helper arms** are preserved; Aura/Bō metadata adds two, and the separate `has_convoke` match contains exactly nine identities. Raw definition comparison changes 211 rows because token/factory references are catalogue-positioned; normalization uses immutable factory identity.

## Validation and reproduction

Native coverage includes exhaustive matching against individual assignments, widened extremes, both controllers and every hybrid equivalent; actual zero-pool casts; non-targeting sick contributions; exact ownership/colors/incarnations; optional full real payment; legal extra credit choices; mana-creature tap exclusivity and actual sacrificed-source funding; related Ward isolation; locked old reductions and unchanged mana value; tap-trigger ordering and late rollback; combat membership; full source-departure/countered Winter packets and cleanup; Appeal's minimum/two-target/partial-illegality behavior; and both controllers' optional unrestricted Disruptor/Bō targets, complete equip and departure cleanup.

Four compiler methods cover all ten complete bodies, strict declaration/cardinality grammar, unknown residuals, invalid printed costs, reciprocal composed-cost gates and the explicit gated neighbors. The public Python module has **two methods / 32 legal paid scenarios**: all ten cards in both seats, all three actual hybrid equivalents in both seats, real zero-contributor payment and zero-target alternatives. Every action checks clone/snapshot/restore/stale replay; private frames check wrong actors, illegal contributors and duplicate reservations without mutation.

Two Standard100 runs reproduce **72,648 exact decisions/replays**. Every full body has `[5,5]` paid casts and effects across the two controllers. Each intrinsic Convoke body has `[4,4]` actual Convoke casts and `[1,1]` zero-contributor casts. Hybrid modes are `[[2,2],[2,2],[1,1]]`; the ledger records 135 actual contribution taps, 207 private frames, ten paid Bō equip effects and ten Winter cast effects. All games use legal 60-card Standard decks and finish with an engine outcome.

A concurrent native 1,000-game benchmark measured 571,049 decisions at about **173,673 decisions/s**, **343 ns clone / 328 ns restore**. It includes native policy/observation work, excludes neural inference/tensor/JSON work, and makes no Java performance comparison. This is a smoke workload; Convoke behavior is measured separately by the acceptance ledger.

```sh
cargo test --all-targets
python3 -m unittest discover -s scripts
cargo build --release
PYTHONPATH=python python3 -m unittest discover -s python
cargo run --release --example standard_convoke -- 100
cargo run --release --bin benchmark -- 1000
cargo clippy --all-targets -- -D warnings
cargo fmt --check
python3 scripts/generate_catalogue.py --check
```

For an isolated HOME-backed build, pass `CARGO_TARGET_DIR` and inject its immutable release library explicitly into `Game(..., library=path)` before public tests. An environment variable does not change the binding's default library path. Do not overwrite a library mapped by any live Python process.
