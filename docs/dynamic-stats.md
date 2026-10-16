# Characteristic stats and count-scaled bonuses

The engine derives characteristic-defining power/toughness in every zone,
including hands, libraries, the stack, graveyards, exile and outside the game.
Counts use the source's owner (the current engine's controller model), intrinsic
types/subtypes and current zones. Battlefield creature counts include the source
only while it is itself on the battlefield. Graveyard/exile card counts exclude
tokens. A fixed toughness remains fixed when only power is characteristic.

Characteristic values apply in layer 7a, before temporary modifiers, counters,
attachments and static/scaled bonuses in layer 7c. Shared characteristic counts
for both axes are evaluated once per query. Live scaling supports source bonuses
and attached creature bonuses, counting from the modifier source's perspective.
Ethereal Armor counts itself; an owned Blanchwood Armor on an opposing creature
counts the Aura controller's Forests. Basic Islands exclude typed nonbasic lands
and Wastes. Changeling participates in creature-subtype counts, including
Boneclub Berserker's other-Goblin predicate.

Immutable `StatValue`/`CountFilter` definitions cover owned battlefield/zone card
counts, type unions and exclusions, intrinsic subtypes, basic status and source
exclusion. Values can sum independent counts or find the greatest fixed mana
value. Tarmogoyf counts the union of ordinary card types across both graveyards,
not the number of cards, supertypes or subtypes. Immutable type masks widen to
u16 to represent Kindred and Battle alongside the seven previous ordinary types.
This does not implement the gameplay of otherwise unsupported cards of those
types. Double-faced gameplay and non-Standard special card types remain gated.

No count calls derived stats, preventing recursive layer dependencies. These
rules do not implement power/toughness setting, switching, general dependencies,
control/type changes, color counting, variable mana or conditional expressions.
The exact parser removes only recognized clauses, validates printed star axes
and preserves the residual complete card body. Unrelated card text—including
intrinsic land-mana reminders—is left intact when no stat rule is extracted.
Unsupported residual abilities still reject the entire definition.

Eighteen complete Standard identities are added:

| Card | Complete admitted semantics |
| --- | --- |
| Adelbert Steiner | Lifelink; +1/+1 per owned Equipment, attached or not |
| Blanchwood Armor | Creature Aura; +1/+1 per owned Forest |
| Boneclub Berserker | +2/+0 per other owned Goblin |
| Burrowguard Mentor | Trample; both stats count owned creatures |
| Cosmogoyf | Power counts owned cards in exile; toughness is that value plus one |
| Crusader of Odric | Both stats count owned creatures |
| Enigma Drake | Flying; power counts owned instant/sorcery cards in graveyard; toughness 4 |
| Ethereal Armor | Creature Aura; +1/+1 per owned enchantment and first strike |
| Guidelight Synergist | Flying; +1/+0 per owned artifact, including itself |
| Harmonious Grovestrider | Ward 2; both stats count owned lands |
| Kraven, Proud Predator | Vigilance; power is greatest mana value among owned permanents; toughness 4 |
| Mirkwood Pathmaker | Both stats count owned lands |
| Moon-Vigil Adherents | Trample; +1/+1 per owned creature plus owned graveyard creature card |
| Regal Bunnicorn | Both stats count owned nonland permanents |
| Snow Villiers | Vigilance; power counts owned creatures; toughness 3 |
| Tarmogoyf | Distinct graveyard card-type power; toughness plus one |
| Tempest Djinn | Flying; +1/+0 per owned basic Island |
| Xande, Dark Mage | Menace; +1/+1 per owned noncreature, nonland graveyard card |

All values are queried from owned game state and immutable definitions, with no
mutable derived cache or additional per-object storage. Scaling sources join the
existing live-source index; central transitions remove them and snapshots own
the index. All simultaneous departures capture their characteristics before any
move changes counts. Resolving instants reach the graveyard before the next SBA
check, so Tarmogoyf can survive damage because the new card type increased its
toughness. Removing another creature can cause a count-scaled survivor to die on
a later repeated SBA check. Both paths are regression fixtures.

Validation: 533 Rust tests (525 unit plus eight integration), 95 compiler tests,
61 rebuilt release Python tests, catalogue consistency, rustfmt and Clippy with
warnings denied. Fifteen focused engine fixtures cover all-zone counts, cards
versus tokens, distinct types, asymmetric fixed stats, subtypes/basic status,
attachments, ward, counter/temporary/static layer order, simultaneous last-known
information, repeated SBAs and clone/restore. Type-only graveyard/index fixtures
are explicitly distinguished from executing unsupported planeswalker/kindred/
battle abilities or playing an unattached Aura legally.

`standard_dynamic_stats` runs two supported 60-card Standard decks. One hundred
games completed naturally with 34,101 decisions, 22,117 independent public stat
checks and 4,912 cloned/replayed boundaries. The public checker independently
counts card types, creatures, lands and nonlands; it skips attached or modified
creatures when checking bare characteristic values. Maximum initialized state
was 12,088 bytes, excluding allocator metadata, spare capacity, external snapshots
and immutable definitions. This workload is not an XMage speed comparison or
proof of complete Standard conformance.

Coverage is 907/5,164 Standard identities, six regression definitions and 41 token
factories, with no previous admissions removed. Observation schema remains 33.
Remaining card semantics, wider layers/replacements, packed/batched RL encoding,
information-set MCTS and native training integration remain incomplete.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 208.2a, 604.3, 613.4a/c, 205.2 and 704.
