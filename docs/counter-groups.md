# Shared counter groups and doubling

Issue: https://github.com/altf4/rustmage/issues/86

The immutable program set now includes `PutCreatureCounters`, which snapshots a
matching battlefield creature set and places the chosen fixed counter amount on
every member, and `DoubleCounters`, which doubles the existing counter counts on
legal targets. The state still owns the three implemented counter kinds (+1/+1,
-1/-1, stun). Unsupported counter kinds remain whole-card admission gates.
Doubling adds as many counters of each present kind as are already there; it does
not create a different counter kind or turn existing counters into another kind.
General counter-placement replacements/triggers remain unsupported.

The compiler also supports each-of multi-target counter clauses with exact
minimum/maximum counts, and Zimone's creature/artifact union under your control.
These use existing distinct-target groups, hexproof checks, source incarnations,
partial-resolution legality and complete owned target-selection state.
Non-targeted each-creature effects use live resolution-time controller/type,
other-source, token and effective-keyword predicates. Creatures entering after
announcement can qualify; earlier departing incarnations cannot receive counters.

All recipients are selected before a group instruction changes any counters.
State-based actions wait until the entire resolution finishes. This preserves
simultaneous deaths and frozen death observers, opposing-counter cancellation,
and creatures saved by a later instruction in the same resolving program.
Indestructible prevents destruction, but does not prevent zero-toughness death.
Counters persist through cleanup and reset on zone changes; stun still replaces
untap attempts after doubling. Rules references: CR122, 608.2 and 704 in the
[pinned Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).

Twelve complete Standard admissions bring coverage to 922/5,164 (41 token
factories and six regression definitions): Abandoned Air Temple; Agent Bishop,
Man in Black; Agent Phil Coulson; Darkness Descends; Duty Beyond Death; Earth
Kingdom Soldier; Lyra Dawnbringer; Lyra, Archangel of Dawn; Minwu, White Mage;
The Crystal's Chosen; Web-Warriors; and Zimone, Paradox Sculptor. The Angel/Cleric
noun support also unlocks Lyra Dawnbringer's existing continuous rules. The
Crystal's Chosen composes unconditional `Then` sentences in printed order.
Residual conditions, unknown counter kinds and other missing mechanics still
reject the whole card. No mutable state or observation-schema field is added;
existing schema 34 exposes all counter values and choices in Rust/C/Python.

Regression tests exercise every admission, legality/privacy of target selection,
source changes, live sets/changeling matching, exact mana/tap/additional costs,
partial fizzle, death ordering, cleanup, stun and replay. The explicitly synthetic
IR fixture verifies no mid-resolution SBA without admitting an unsupported card.
`standard_counter_groups` uses supported 60-card Standard decks and independently
computes expected counter values from public before/after observations, while
also cloning and restoring target/payment/resolution boundaries. Python tests
exercise mass-counter deaths and Zimone's activated doubling through the native
library. Full Standard acceptance and the wider RL integration remain incomplete.
