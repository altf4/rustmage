# Bargain: exact optional sacrifice costs

Issue [#223](https://github.com/altf4/rustmage/issues/223) adds ten complete Standard bodies to the in-process library. The integrated branch implements **1,587 / 5,164 Standard identities**, with 3,577 gated identities, 74 immutable token factories and one emblem. The catalogue fingerprint is `8adc126ecaca5eab9b92e2248811b4e2ca4db6f70df03647cd217bd25a2a53e7`.

| Card | Complete behavior |
| --- | --- |
| Archon's Glory | Target creature gets +2/+2; bargained grants flying and lifelink until cleanup. |
| Candy Grapple | One target creature gets -3/-3 or, when bargained, -5/-5 until cleanup. |
| Farsight Ritual | One private top-four or top-eight window, put two into Hand, randomize the rest onto the bottom. Short libraries put all available cards into Hand; this never draws. |
| Hamlet Glutton | Bargained costs two generic less; 6/6 trample and an unconditional gain-three entry trigger. |
| High Fae Negotiator | 3/5 flying; bargained entry makes the opponent lose three life and its controller gain three. |
| Ice Out | Bargained costs one generic less; counter target spell. The two blue requirements remain. |
| Johann's Stopgap | Bargained costs two generic less; bounce a nonland permanent, then draw one. All-target illegality prevents both. |
| Kellan's Lightblades | One operation deals three damage to an attacking or blocking creature, or destroys it when bargained. |
| Tenacious Tomeseeker | 3/2; bargained entry returns a targeted own instant or sorcery from Graveyard to Hand. |
| Troublemaker Ouphe | 2/2; bargained entry exiles a targeted opponent artifact or enchantment. |

The [provenance manifest](bargain-admissions.json) contains complete snapshot Oracle texts and **82 SHA-256 source records across 45 Java files**, pinned to XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. The ten full constructors and their direct effects/cost helpers were audited against CR702.166a–d and CR601.2b/f–h from the pinned September 25, 2026 rules. No existing card or token recipe is partially extended.

## Public cast declaration and real payment

Observation schema **63** includes `Action::ChooseBargain { sacrifice: Option<ObjectRef> }`, decision `bargain`, and `CastInfo.bargain: Option<bool>`. `None` on the action declines; `Some(reference)` selects one exact controlled artifact, enchantment **or** token. The action is offered only to the actor and precedes targets, mana abilities and payment. A nonartifact creature token qualifies. An ordinary creature or land does not qualify solely by being a permanent. Existing owner/controller boundaries remain enforced.

`CastInfo.bargain == None` means there is no bargain declaration or it is not yet chosen. `Some(false)` and `Some(true)` record the declared branch, separately from origin, casting method, offspring and kicker. The same declaration path is used for a real authorized exile cast; direct casting shortcuts cannot bypass it. None of the ten bodies has its own flashback ability. Composite bargain plus another additional cost/forage, offspring, kicker, flashback or target-dependent reduction remains gated until the whole combined recipe is implemented.

Funding queries reserve the chosen exact permanent and never sacrifice it or emit events. The complete colored/generic total must remain reachable with that reservation. A Treasure cannot be used both for its sacrifice-to-produce-mana ability and for bargain. A different Treasure may produce mana while the reserved artifact survives until payment. Ordinary tap-only mana use of a reserved permanent remains legal if it leaves the permanent available to sacrifice. The existing additional-cost transaction makes the actual sacrifice and its real death/graveyard events at payment; targets may legally include the permanent being sacrificed and then become illegal at resolution.

The three conditional generic reductions use the declared branch before the total cost locks. Colored requirements, printed mana value and ordinary alternative-origin authorization do not change. Once locked, subsequent resource consumption and departures of ordinary reducers cannot reprice the spell. Invalid actors, targets, resource generations, underpayments and late capacity failures restore the entire action, including mana, sacrifice, queued triggers, history and RNG.

## Historical entry and complete resolution

`SelfBargained` observes only the exact spell's staged entry event. There is no per-object bargained marker. Reanimation and later incarnations are unqualified; already queued entry triggers retain historical qualification if the source leaves or reenters. Normal target generation and source/program provenance still apply. Entry-trigger suppression prevents those triggers. Hamlet's gain-three entry trigger is unconditional.

`IfBargained` is immutable program control flow. Candy and Kellan execute a single mutually exclusive instruction. Farsight selects a single four/eight-card window, retains private looked information until a deferred commit, uses the existing fixed-two selection and random-bottom transaction, and never exposes the private frame or selected cards to the other viewer. Normal all-target fizzle rules apply to targeted spells and entry triggers.

Agatha's Champion remains gated because it requires a real untargeted source operand for fighting an optional enemy target. The manifest lists the other bargain residuals that remain gated: new-zone target replacement, free casting, type/base changes and Roles, top/grave arrangements, arbitrary mana allocation, larger reveal-to-battlefield operations, death-exile replacements and X. They are excluded from the ten-card count.

## Standalone validation and memory

The following measurements describe the original ten-card branch before integration with explore and Map tokens.

Fourteen native groups cover all 40 card/branch/controller combinations, typed costs and discounts, pure funding, actual Treasure reservation, token/enchantment OR filtering, live target predicates and reincarnation, paid exile, short-library private selection, cleanup, source departure/reentry, suppression and late rollback. The complete native suite passes **1,423 tests** (1,411 unit and 12 integration). The compiler suite passes **274 tests**. The canonical comparison preserves all 5,257 indexed identities, with exactly ten complete definitions changed and 5,247 preserved after normalizing the new explicit false filter default. All 493 prior typed helper arms, color masks and complete token/emblem definitions remain unchanged; ten bargain metadata arms are new.

The complete fresh-release Python suite passes **178 tests in 224.334 seconds**, including the two new methods and **48 bargain scenarios**. Full Clippy, Rust formatting and regenerated catalogue checks pass. The adjacent kicker Standard100 retains exactly **102,219 decisions/replays**.

The public [Python acceptance](../python/test_bargain.py) uses real paid games, both seats and branches, real artifact/enchantment/nonartifact-token sacrifices and actual exile permissions. Its constrained Treasure games reject sacrificing the sole mana-producing Treasure while accepting an independent artifact reservation; a cloned continuation really spends the Treasure for mana and pays the separate sacrifice without changing its parent. Every played decision is checked through clone, snapshot, restore and stale-revision rejection.

The [Standard workload](../examples/standard_bargain.rs) plays legal 60-card decks. Two independent 100-game executions produce identical full vectors and complete **85,454 decisions and matching replay boundaries** with all 40 cast/effect cells nonzero, 161 real sacrifices, 120 private frames/80 selected cards, 44 returns, 28 draws, nine exiles, 23 all-invalid fizzles, six countered packets and 666 rejected actions.

Measured initial state is **19,976 bytes**, an increase of 32 bytes from 19,944. `CastInfo` is five bytes and `SpellView` is 272 bytes (previously 256); enclosing owned context layout accounts for this cost. No per-object or new persistent State field is added. The Standard100 peak is **23,084 logical bytes**. Active bargain selection reuses the announcement's existing additional-cost reference; funding scans existing resources without enumerating sacrifice subsets or cloning game state.

## Integration with explore

On the merged explore base, the combined branch passes **1,439 Rust tests (1,427 unit and 12 integration), 278 compiler/data tests and 187 fresh-release Python tests**. The latter suites complete in 560.083 and 332.048 seconds during concurrent validation. All-target Clippy, formatting, regenerated catalogue and whitespace checks pass.

Canonical comparison with the final explore runtime proves exactly **10 changed / 5,248 unchanged definitions**, all 74 existing factories/metadata, ordinary identities/colors and 494 previous numeric helper arms preserved after normalizing the explicit false sacrifice-filter default. Ten new bargain helper arms encode the admitted metadata. All 82 provenance records across 45 files match the pinned Java checkout.

The bargain workload reproduces all 85,454 decisions/replays and complete vectors. The adjacent explore workload retains all 215,656 decisions and vectors; its peak becomes 21,644 logical bytes with the expanded context. Initial state remains 19,976 bytes. A concurrent 1,000-game native smoke run reports 571,049 decisions at about 262,000 decisions/s, with 182 ns clone and 187 ns restore. It excludes inference/JSON/tensors and provides no Java performance comparison.
