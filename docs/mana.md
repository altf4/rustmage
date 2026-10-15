# Mana production and predefined artifact tokens

Mana pools and payments use six W/U/B/R/G/C slots. The final slot is colorless;
generic costs are separate. A required {C} consumes colorless mana, while
generic costs may consume any mana type. “Any color” offers W/U/B/R/G and does
not offer C. Fixed production can add multiple types at once.

Pure, nontargeted activated production resolves immediately through
`ActivateMana`, retaining priority without adding a stack object. All existing
activation costs and restrictions still apply, including tap/sickness/haste,
self-sacrifice, exact mana payment, life/counter costs and once-per-turn use.
Costs commit and production completes before state-based checks. Other activated
abilities, such as Food/Clue abilities, use the normal stack and response window.

Spells and admitted triggers can add fixed mana through their resolution
programs. Chosen production in a resolving program uses an owned continuation
and `ChooseMana`; only its actor sees `mana_choices`. Snapshot replay retains
the program counter and pending production. Mixed-effect activated mana programs,
restricted/variable mana and unimplemented restrictions remain gated.

The predefined compiler supports Treasure, Gold, Food, Clue, Mutagen and
Heartwood when referenced by fully accepted cards. Four new factories are
currently referenced: Treasure, Food, Clue and Mutagen. They have their actual
artifact types, subtypes, colors and abilities; they are not vanilla creatures.
Default names follow subtype plus “Token.” Investigate creates a Clue.

Treasure sacrifices for immediate colored mana. Food taps and sacrifices for
life, Clue sacrifices for drawing without requiring a tap, and Mutagen taps and
sacrifices for a targeted counter with sorcery timing. Normal token departure,
cessation, slot reuse, deck rejection and incarnation checks apply.

Schema **10** changes every mana vector to six slots and adds `ActivateMana`,
`ChooseMana` and actor-only `mana_choices`. The C ABI remains a JSON adapter;
consumers must use the new observation schema and catalogue fingerprint.

This adds 41 executable Standard definitions (212/5,164), with 27 referenced token
factories counted separately. This does not prove complete rules support.
Mana production is currently exposed at priority; activating mana sources during
casting/cost payment remains outstanding in issue #21. Prefunding costs supports
the current control loop, but does not represent every legal payment sequence.
Remaining payment windows, triggered mana abilities, spending restrictions and
other broader rules require further work before full acceptance.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 106, 107.4c, 111.4, 111.10, 117 and 605.
