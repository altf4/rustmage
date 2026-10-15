# Resolution sacrifice choices and conditional payments

The engine supports exact sacrifice instructions and optional discard/sacrifice
costs paid while a spell or ability resolves. A sacrifice moves an eligible
controlled permanent to its owner's graveyard. It does not target or destroy;
hexproof and indestructible do not prevent it (CR701.21). Control changes remain
gated, so owner and controller still coincide for admitted objects.

For an instruction affecting multiple players, choices occur in active-player
order. All chosen permanents remain on the battlefield until everyone finishes;
the instruction then commits one simultaneous zone-change batch. Earlier public
selections are visible to later players. Mandatory effects do as much as possible
when a player controls fewer eligible permanents than requested. Source creatures
can be chosen unless the text excludes them. A batch captures departure/death
information before movement and processes linked returns after the whole batch.
State-based actions and death-trigger placement wait for the enclosing resolution.

`You may [pay]. If you do, [effect]` is a resolution cost, distinct from an ordinary
forced discard (CR118.12). The complete cost must be payable. Insufficient resources
skip the conditional block; declining also skips it. Selected cards/permanents do
not move until selection is complete. `DeclineEffectPayment` can abandon an
uncommitted partial optional selection. A full selected payment commits and runs
the conditional instructions. No priority is offered between payment and its
consequences. Reflexive `when you do` triggers remain gated.

`Action::Sacrifice { object }` binds an exact incarnation. Existing `Discard`
actions select private hand cards for discard payments. Observation schema 20 adds
`effect_choice`, with choice kind, requested/required amounts, optional status,
selected objects and candidates. Sacrifice selections are public; discard-payment
choices and hand identities are visible only to their actor. Opponents see the
cards when the discard commits to the public graveyard. Actions are exposed only
to the deciding player, and stale action indices remain revision-checked.

Exact filters cover basic permanent types/unions, nonland, token/nontoken,
source-only/another, supported keywords and Food/Clue/Treasure/Lander subtypes.
Filters use current keywords and exact source identity. A returned incarnation
is independent of the old source. Planeswalker catalogue identities now carry
their type bit for type-union predicates; planeswalker gameplay is still unsupported
and those definitions are rejected at deck admission.

The boxed continuation, required amounts, selected references, source, instruction
focus and skip offset belong to each game/snapshot. No mutable globals or callbacks
are involved. `state_bytes()` includes initialized continuation storage. Search
and other subsequent private choices resume from the same owned instruction
program. Damage from a sacrificed source uses central departure LKI.

Twelve complete definitions bring executable Standard coverage to 611/5,164,
plus six regression cards and 35 token factories. Rust fixtures cover APNAP/public
selection, exact payment versus partial mandatory effects, atomic rejection,
source identity/LKI, token cessation, linked returns, fizzling and search replay.
Synthetic kernel programs exercise combinations not yet admitted as full cards.
Python checks privacy, public simultaneous selection and native clone/restore;
`standard_payments` exercises supported Standard decks through natural outcomes.

Compiler support is limited to exact fixed filters and single conditional effect
sentences. Greatest-power/value restrictions, extra casting/activation sacrifice
costs, wider sacrifice/discard-triggered events, reflexive triggers, mixed payment
alternatives and ambiguous conditional scope remain gated. This milestone does
not establish full Standard or complete RL/search acceptance.

Rules: [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
CR101.4, 118.12, 608 and 701.21.
