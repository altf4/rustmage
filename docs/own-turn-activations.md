# Own-turn activations and exact return targets

Issue: https://github.com/altf4/rustmage/issues/130

Four complete Standard bodies are admitted:

| Card | Complete implemented behavior |
| --- | --- |
| Professor Zei, Anthropologist | {U/R}{U/R}, legendary 0/3 Human Advisor Ally; tap/discard a card to draw a card on either turn; {1}, tap and sacrifice itself to return target instant/sorcery card from its controller's graveyard to hand, only during that controller's turn |
| Rescue Girl, First Responder | {2}{W}, legendary 1/3 Human Cleric; flying; tap to return another target controlled permanent to its owner's hand, only during its controller's turn |
| Shipwreck Dowser | {3}{U}{U}, 3/3 Merfolk Wizard; prowess; entry trigger returns target instant/sorcery card from its controller's graveyard to hand |
| Zealous Lorecaster | {5}{R}, 4/4 Giant Sorcerer; entry trigger returns target instant/sorcery card from its controller's graveyard to hand |

Full Oracle text and pinned Java card/turn-condition hashes are in
[own-turn-admissions.json](own-turn-admissions.json), at XMage revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Catalogue fingerprint:
`645980960a22c6932b79117b6d63c03efcaed89b2ca3954863e6c9c014dc03f7`.
Coverage is **1,118/5,164** Standard identities plus six regression definitions,
49 token factories and one emblem; **4,046** Standard identities remain unsupported.

`ActivationGuard::OwnTurn` is an immutable starting restriction that compares
controller with active player. Real activation enumeration and counterfactual
mana-source enumeration enforce the same restriction, without requiring an
empty stack or main phase. Both direct and announced activations are rejected
on the inactive turn; legal priority windows in upkeep, combat and end step
remain available. Captured abilities finish after source departure. The guard
does not replace tap/summoning-sickness, haste, source-generation, mana or
sacrifice obligations. Neither new printed own-turn activation produces mana;
an explicit future-IR test covers the virtual funding guard helper.

Two immutable object-target rules extend existing announcement/resolution
matching: another controlled battlefield permanent, including lands and tokens;
and an instant/sorcery card in the controller's graveyard. Both revalidate the
bound generation at resolution. Invalidated targets fizzle without refunding
costs. Returning a token makes it cease to exist after leaving the battlefield.
Zei's captured ability remains independent after paying tap/sacrifice; a future
exile-link fixture also verifies entry-choice suspension and cost commitment.
The printed Zei has no exile link. Target ownership currently follows the
engine's admitted owner-equals-controller model; control-change bodies remain
gated.

The compiler normalizes only exact source-name sacrifice cost components,
authenticated against that card's full name or existing legendary short-name
rules. It does not rewrite effect text or arbitrary named objects. The own-turn
suffix has a distinct guard from sorcery timing; compound unsupported conditions
and unknown residual text fail whole-body admission. Remaining dependencies in
Prehistoric Pet, Wishclaw Talisman, Path to Redemption, M.O.D.O.K., Scour for Scrap
and Oltec Archaeologists stay gated. The typed graveyard return also unlocks
Dowser/Lorecaster's complete entry bodies; Dowser retains its separate prowess
trigger, including end-of-turn expiration.

Rules references: CR201.5, 602.1b, 602.2, 602.5, 608.2b and 111.7 in the pinned
[Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
Observation schema remains **47**, ABI 1 and control protocol 2. This slice adds
no observation payload or persistent state field. Initial logical state remains
**19,784 bytes**. Immutable guards/target definitions are shared catalogue data.
Logical initialized accounting excludes capacity, allocator/COW metadata,
statics, observations and external snapshots. Pending discard choices remain
private; returned hand cards are inspected through their owner's view.

Validation passes **833 Rust tests** (821 unit and 12 integration), **155
compiler/data tests**, **112 rebuilt-release Python tests**, formatting, Clippy
with warnings denied, catalogue and diff checks. Seven new native tests cover
own-turn versus inactive-turn legality across priority windows, nonempty stack,
both action paths, exact target types/zones/ownership, lands/artifacts/creatures/
enchantments/planeswalkers/tokens, sickness/haste/tapping, Zei's unrestricted loot,
source sacrifice, stale targets, suspended entry, full ETB bodies/prowess and the
virtual guard. The public binding suite exercises all four new bodies plus
Zei's private discard ability through native branching/restoration.

`cargo run --release --example standard_own_turn -- 100` runs 100 complete
games with legal identical 60-card Standard decks. It checks **90,735 decisions,
10,291 offered restricted-activation timing checks, 1,643 recoveries, 65 loot
draws, 347 prowess resolutions, 49,053 clone/restore/replays and 21,252 maximum
initialized state bytes**. All four cards enter and recover; Zei's two activations
and Rescue Girl's activation are exercised. Recovery covers both instant and
sorcery cards and land/nonland/token permanent routes. Invalidated targets are
excluded from successful-return assertions, and token returns check cessation.
The checker uses the source controller's observation for private returned hand
cards. An external per-turn set limits repeated policy abilities. Selected seeds
preserve a Treasure until Rescue Girl can return it, and pass on that ability
to prevent spending its target in response; these are policy choices, not engine
restrictions. Native tests separately cover inactive turns and legal responses
with a nonempty stack.

A fresh 1,000-game Freeform baseline yields 571,049 decisions at 397,383.78
decisions/s and 695.88 games/s, clone 158.69 ns and restore 123.84 ns. This
unpinned sample includes smoke policy and observation construction and excludes
model inference, tensor encoding and JSON. It establishes neither a performance
change nor XMage parity.

Full Standard and native RL/search acceptance remain incomplete and active.
