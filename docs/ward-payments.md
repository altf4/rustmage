# Ward and mana payments during resolution

Mana-based ward is a triggered ability, not an additional targeting restriction
or casting cost. The engine queues it when an opponent's spell or ability first
binds the exact battlefield permanent as a target. Repeated target occurrences
on the same stack item do not create extra triggers from the same ward instance.
Separate printed ward instances retain separate abilities. Own targets, stack
cards and non-targeted effects do not trigger ward.

Ward stores an exact stack ID and that item's controller. The associated item
may be a spell, activated ability or triggered ability; a physical card slot is
not its identity. The association survives the ward source's departure, costs,
trigger ordering and snapshots. Declining a ward payment removes only the linked
stack item, moving a countered spell to the graveyard while leaving ability
sources alone. A later spell using the same physical card is a different item.

An owned resolution payment records payer, fixed colored/generic cost, victim,
counter destination and continuation. Schema 26 adds public `resolution_payment`,
`related_stack`, `PayResolutionMana` and `DeclineResolutionMana`. Only the payer
receives actions. The player can activate admitted mana abilities during the
request, pay an exact legal allocation, or decline. Mana abilities are available
even when the complete payment cannot be assembled; generated mana remains if
the player declines. Ordinary spells and nonmana abilities cannot interrupt the
request, and neither priority nor state-based actions occur mid-resolution.

Mana-ability costs can open existing entry transactions. An Aura attachment
choice then suspends the payment without consuming the next program instruction;
its owned continuation resumes the same payment window afterward. Source
self-sacrifice, life/counter costs, local RNG, trigger queues and all continuation
fields remain part of the snapshot. Memory accounting includes the boxed request.

Counter-unless-paid spells share this mechanism. Their paid continuations belong
to the counter's controller: Don't Make a Sound offers private surveil only after
payment, and Divert Disaster creates a Lander only after payment. No More Lies
exiles a spell when its declined payment results in the counter. Restricted
counter targets distinguish creature, noncreature, instant/sorcery, artifact and
enchantment spells; they never accept stack abilities. Illegal targets still
fizzle before requesting payment.

These rules follow CR702.21, 605.3a–c and the resolution/cost rules in the
[September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
The parser admits exact fixed mana ward costs and complete counter statements;
nonmana/variable ward, granted ward, conditional cost increases and all residual
clauses remain gated. Paid counter branches must contain exactly one complete
sentence; multi-sentence tails remain unsupported until their scope is represented
explicitly.

Sixteen complete Standard definitions are added:

| Cards | Newly executable complete semantics |
| --- | --- |
| Archive Dragon | Flying, ward two, entry scry two |
| Hulking Raptor | Ward two and first-main-phase mana trigger |
| Spider-Rex, Daring Dino | Reach/trample and ward two |
| Toadstool Admirer | Ward two and paid +1/+1 counter activation |
| Unflinching Hortimancer | Ward one and life-gain counter trigger |
| Wrecking Gecko | Ward two and paid growth/trample activation |
| It'll Quench Ya! | Counter unless two is paid |
| Spell Pierce | Noncreature counter unless two is paid |
| Don't Make a Sound | Counter unless two is paid; paid branch surveils two |
| Divert Disaster | Counter unless two is paid; paid branch creates Lander |
| No More Lies | Counter unless three is paid; countered spell is exiled |
| Essence Scatter; Negate | Creature-only/noncreature-only counters |
| Diversion Unit | Flying and paid self-sacrifice instant/sorcery counter unless three is paid |
| Confusticate and Bebother | Counter-unless-four or draw/discard modes |
| Theorix Charm | Noncreature counter-unless-two, creature debuff or mill/draw modes |

Validation includes 366 Rust tests (358 unit and eight integration), 64 compiler
tests, 39 rebuilt release Python tests, catalogue consistency, rustfmt and Clippy
with warnings denied. Native fixtures cover exact stack identity, target timing,
source departure, wrong actors/allocations, ability counters, restricted target
masks, paid/unpaid branches, private continuation choices and replay.
Synthetic target-group, colored-payment and linked-Treasure/Aura fixtures exercise
generic engine vocabulary; they do not add partial-card admissions or claim
printed abilities that those cards do not have.

One hundred supported Standard games completed naturally: 53,266 decisions,
319 payment requests (60 ward), 56 payments, 263 declines, 20 resolution mana
activations and 6,351 cloned/replayed boundaries. Maximum initialized state was
12,412 bytes for this workload, excluding allocator metadata, spare capacity,
snapshots and static definitions. The smoke policy uses only its observation and
offered actions; payment mana choices reduce the visible payment deficit to
avoid cycling conversions. This is not a competitive policy or an XMage speed
comparison.

Coverage is 728/5,164 Standard identities, six regression cards and 36 token
factories. Other ward costs, generic optional mana effects, uncounterability,
additional targeting events, cost/replacement/control layers, remaining card
semantics and native RL/search integration are incomplete. Full Standard
acceptance remains outstanding.
