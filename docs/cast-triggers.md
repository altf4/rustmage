# Completed-cast events and prowess

Casting triggers fire when the spell becomes cast, after all choices and costs
(CR601.2i). The native engine records this event for direct casts, prepaid target
declarations, modal casts and explicit target/payment windows. Merely proposing
a spell, moving it to the stack for a declaration, choosing a mode/target, or
activating mana does not emit it. Countering a spell never undoes its cast event.
Playing a land, activating an ability and directly entering the battlefield do
not count as casts.

Immutable trigger conditions describe own/opponent/any-player spell events and
creature, noncreature, instant-or-sorcery, artifact or enchantment filters. Exact
second-spell conditions count all spells cast by that player during the current
turn, including earlier spells cast before the source entered. Both players'
counts reset at the beginning of every turn. Casting on another player's turn
uses the caster's count for that turn. Observation schema 18 exposes
`players[].spells_cast_this_turn` publicly.

A completed spell is on the stack before trigger placement begins. Its caster
retains priority; state-based actions and APNAP trigger ordering/target choices
finish before that priority can be used. Stack entry alone is insufficient to
record a cast: the prepaid empty-declaration path settles its deferred costs
before explicitly completing the cast event. Triggered/activated stack entries
never recursively emit spell-cast events.

Prowess compiles as an independent noncreature-spell trigger giving its exact
source incarnation +1/+1 until end of turn. Multiple printed instances each
trigger (CR702.108); they can be ordered separately and resolve separately.
They are not a bit in the static keyword mask. A departed or returned source is
not modified by its earlier prowess trigger. Unsupported prowess-granting auras,
static grants and tokens remain gated.

An owned live-source index avoids scanning every card for each cast event. Zone
transitions maintain the index; token creation uses the same registration hook.
The index, turn-local counts, source references, queued triggers and all partial
announcements belong to each game/snapshot. No callback or global history is
needed to replay a cast. Tests compare index maintenance with complete admitted
card definitions and exercise independent clones at declaration/payment/cast
boundaries. The `standard_casts` example restores/replays each completed cast.

Supporting instructions add source-incarnation untapping (respecting stun) and
source-aware damage to each opponent. Damage uses current or last-known source
keywords, including lifelink, and remains distinct from life loss.

Thirty-three complete definitions bring executable Standard coverage to
580/5,164, plus six regression cards and 34 token factories. Complete-text
compiler checks retain the other 4,584 identities as unsupported. Conditional
mana-spent clauses, casting restrictions/discounts, copies, magecraft, additional
costs, broader history queries and remaining card-specific rules are not admitted
by recognizing a cast-trigger prefix. This milestone does not establish full
Standard or RL/search acceptance.

Rules: [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
CR601.2i, 603 and 702.108.
