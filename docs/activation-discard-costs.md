# Owned discard costs for activated abilities

Issue: https://github.com/altf4/rustmage/issues/126

Seven complete Standard bodies are admitted:

| Card | Complete implemented behavior |
| --- | --- |
| Charging Strifeknight | {2}{R}, 3/3 Spirit Knight; haste; tap and discard a card to draw a card |
| Flamecache Gecko | {1}{R}, 2/2 Lizard Warlock; entry intervening-if opponent lost life this turn adds {B}{R} on trigger resolution; {1}{R} and discard a card to draw a card |
| Inner Demons Gangsters | {3}{B}, 3/4 Human Rogue Villain; discard a card for +1/+0 and menace until end of turn, only as a sorcery |
| Iron-Shield Elf | {1}{B}, 3/1 Elf Warrior; discard a card for indestructible until end of turn and tap itself on resolution |
| Masked Meower | {R}, 1/1 Spider Cat Hero; haste; discard a card and sacrifice itself to draw a card |
| Murmuring Volume | {3}, artifact Book; tap for one mana of any color; {2}, tap and discard a card to draw a card |
| Thunderhead Gunner | {4}{R}, 4/5 Shark Pirate; reach; discard a card to draw a card, only as a sorcery and once each turn |

Full Oracle text and seven Java SHA-256 hashes are in
[activation-discard-admissions.json](activation-discard-admissions.json), pinned
to XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.
Catalogue fingerprint: `ec260f7f622bf935aeaa47b86b51738f876188bec68e203e5c9f749ef9b9a990`.
Coverage is 1,112/5,164 Standard identities, plus six regression definitions,
49 token factories and one emblem; 4,052 identities remain unsupported.

The immutable activation cost holds an optional one-object external cost.
Cycling keeps its distinct self-discard cost. Announcements require an affordable
mana route and an exact eligible hand card. The existing owned announcement
continuation selects the external card after targets and before the payment
window. Direct/prepaid activation actions cannot bypass this selection, even
with mana already floating. The exact object generation is reserved throughout
counterfactual mana search and concrete payment. Murmuring Volume cannot tap its
mana ability and then tap again for its enclosing draw activation. Funding
queries do not mutate the game.

Only the controller sees candidates and the selected card before payment.
Forged, stale, opposing, out-of-zone and wrong-actor selections are rejected
atomically. Selection does not move the card or fire discard observers. Payment
commits discard once, pays all source/mana costs, then finishes the ability
announcement. The discarded card becomes public in the graveyard; its discard
triggers wait above the completed ability. Activation is neither casting a spell
nor cycling. Masked Meower pays both discard and source sacrifice even if the
source is absent when the draw resolves. Source costs that synchronously return
exiled permanents can suspend for entry choices without repeating payments;
a synthetic future-link test exercises that continuation. The printed Meower
does not create exile links.

The compiler accepts exactly “Discard a card” and the combined sorcery/once-per-
turn restriction. It resolves “Tap it” only in the exact sentence sequence
“This creature gains … until end of turn. Tap it.” Initial ambiguous “It deals”
with an external discard is rejected. Multi-card/filtered discard costs,
arbitrary external sacrifices, discard combined with life/counter costs,
discard-plus-source-sacrifice on triggered observers, discard-cost mana abilities and unsupported
residual clauses remain gated. Mana search does not yet model consuming hand
cards to produce mana, so no such ability is admitted. All seven new abilities
are nontargeted; they share existing target-before-cost sequencing with targeted
announcements. Cost-order alternatives that change outcomes need wider payment
machinery before affected bodies can be admitted. This slice uses a legal
payment order: external discard before source sacrifice/tap/mana costs.

Rules references: CR601.2b–i, 602.2 and 605.3 in the pinned
[Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
Observation schema **46** extends existing `additional_cost`,
`ChooseAdditionalCost` and private `additional_cost`/`payment_request` projections
to activated abilities. ABI 1 and control protocol 2 remain unchanged. No
persistent state field is added. Initial logical state remains **19,752 bytes**;
immutable specifications are shared catalogue data. Logical initialized state
excludes capacity, allocator/COW metadata, statics, observations and external
snapshots.

Validation: 818 Rust tests (806 unit and 12 integration), 152 compiler/data tests
and 108 freshly built release Python binding tests. Formatting, Clippy with
warnings denied, catalogue consistency and diff checks pass. Eight new native
tests cover private mandatory costs, invalid/direct actions, ownership, source
and hand-card funding reservations, haste/tap/sorcery/once restrictions, discard
trigger ordering, source sacrifice, entry suspension, resolution tapping,
cleanup and both Gecko entry branches. The new Python test plays seven native
cases from legal actions, branching and replaying each card's private choice,
payment and body through the C ABI.

`cargo run --release --example standard_activation_discard -- 100` runs two
legal identical 60-card Standard decks through 100 complete games. The workload
checks all seven entries and ability bodies, both Gecko entry branches, exact
payment requests and discarded objects, source sacrifice/tap results and
resolved draw/boost/keyword effects. It produces **61,730 decisions, 2,088
activation payments, 2,088 body checks, 29,219 clone/restore/replays and 21,060
maximum initialized state bytes**. Empty-library draws correctly contribute
zero successful draws. An external per-turn set suppresses repeated printed
abilities to keep the smoke policy finite; the engine still offers legal
repeats. The policy delays some Gecko casts until after an opponent's life loss
to cover the positive entry branch. This is a rules/replay workload, not a
neural policy or a claim of full-format acceptance.

A fresh 1,000-game Freeform baseline yields 571,049 decisions at 383,435.60
decisions/s and 671.46 games/s, clone 127.97 ns and restore 127.32 ns. This is an
unpinned timing sample including smoke policy and observation construction;
it excludes model inference, tensor encoding and JSON and establishes neither
a performance change nor XMage parity.

Full Standard and native RL/search acceptance remain incomplete and active.
