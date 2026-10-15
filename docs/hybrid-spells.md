# Committed two-color hybrid spell costs

Normal-layout spells can now have the ten two-color hybrid symbols in their
printed mana costs. The compiler expands them into unique fixed nonhybrid
equivalents, combining repeated-symbol permutations. For example, `{G/W}{G/W}`
has three equivalents: two green, green plus white, or two white. Other symbols,
including ordinary colored/colorless requirements and generic amounts, retain
their existing meaning. Printed cost text and mana value remain unchanged;
choosing a color to pay does not change a card's colors.

The staged native casting API uses an owned `spell_cost` decision before mode,
target or mana-ability choices. `ChooseSpellCost { cost }` commits one payable
equivalent. Subsequent funding searches and `PayCosts` use only that fixed cost.
The player can legally float extra mana while paying, but cannot switch the
committed colored requirements. Illegal indices, wrong actors, unaffordable
choices and incorrect payments are rejected without state or revision changes.
The spell becomes cast only after all declaration steps and payments finish.

Prefunded `Cast`/`BeginCast` shortcuts already commit their payment before any
later declaration decisions. They enumerate allocations across all equivalents
and deduplicate identical allocations. Fixed-cost spells keep their allocation-free
fast path. Cost alternatives are static immutable catalogue data; only the chosen
fixed cost is stored in the owned announcement, copied by snapshots and clones.
Current mana value remains the printed value for every equivalent. Native tests
also compare every admitted Standard card's printed mana cost/value with the
frozen oracle snapshot.

Schema 32 adds actor-only `cost_choices` (stable catalogue-local index plus fixed
cost) and public `announcing_cost` after commitment. The payment request is absent
until a hybrid equivalent has been chosen. Other viewers receive no actions or
private hand selections. Restoring a snapshot invalidates prior action indices
while retaining the precise committed cost and funding continuation. Inline state
accounting includes the announcement; static alternatives are excluded.

These rules follow CR107.4e, CR202.3f and CR601.2b in the
[September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
Hybrid activation, cycling and resolution-payment costs, monocolored hybrid,
Phyrexian/snow/X costs, broader alternative/additional costs and reductions remain
separately gated. Hybrid spell support never skips the rest of a card's text.

Eighteen complete Standard identities are added:

| Card | Complete admitted semantics |
| --- | --- |
| Abandon Attachments | Optional private discard, then draw two if paid |
| Cat-Owl | Flying; attacking untaps a targeted artifact or creature |
| Duskwatch Hunter | Token blocking restriction; entry targeted creature counter |
| Foot Elite | Attack targets another owned creature for temporary power/indestructible |
| Gallant Citizen; Pond Prophet | Entry draw |
| Head of the Homestead | Entry creates two Rabbit tokens |
| Large Bear | Reach, trample and haste |
| Mary Jane Watson | Once-limited owned Spider entry draw |
| Mechanized Ninja Cavalry | Entry Robot artifact creature token |
| Mob Lookout | Entry targets an owned creature to connive |
| Moonrise Cleric | Flying; attack life gain |
| Noggle Robber | Entry/death Treasure creation |
| Pretending Poxbearers | Death Ally token |
| Punk Frogs | Generic mana ward |
| Slithering Cryptid | Entry Mutagen token with its complete activation |
| Tempest Angler | Completed noncreature casts grow source |
| Wandering Musicians | Attack temporarily grows owned creatures |

Coverage is 834/5,164 Standard identities, six regression definitions and 40 token
factories; no prior admissions were removed. Other hybrid cards remain rejected
when their additional rules are missing, including changeling, hybrid graveyard
activations, compound attack events and continuous entry restrictions.

Validation: 468 Rust tests (460 unit plus eight integration), 82 compiler tests,
51 rebuilt release Python tests, consistency, rustfmt and Clippy with warnings
denied. Seventeen focused Rust fixtures cover all equivalents, exhaustive small
pool allocation comparison, committed funding, wrong actions, private effects,
source/target relations, all new printed bodies and owned replay. The pre-target
ordering test explicitly injects a synthetic declaration target plan; no additional
printed definition is admitted by that fixture. Modal hybrid spell compilation is
covered synthetically; current complete printed admissions have no modal spell.

One hundred supported Standard games between mono-green and mono-blue decks
completed naturally: 45,437 decisions, 801 committed hybrid choices, 801 staged
hybrid payments, 300 prefunded hybrid shortcuts and 4,674 cloned/replayed
boundaries. Maximum initialized state was 13,144 bytes, excluding allocator
metadata, spare capacity, external snapshots and static definitions. These runs
exercise supported decks; full Standard conformance, remaining native RL/search
integration and an XMage speed comparison remain outstanding.
