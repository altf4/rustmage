# Derived additive power and toughness

The engine now evaluates fixed additive power/toughness modifiers in layer 7c.
Printed stats, independent temporary power/toughness deltas, +1/+1 and -1/-1
counters, and matching static anthems/lords contribute to the same query.
Observation, combat damage amounts, lethal assignment, state-based deaths and
cleanup use that result. Nonbattlefield creatures retain their appropriate
printed/stored characteristics; battlefield anthems do not affect hand cards.

Immutable StaticModifier definitions hold a creature filter and two deltas.
Filters can select controller/opponents/everyone, other creatures, an exact
supported subtype, an implemented printed keyword, attackers, tokens/nontokens,
or artifact creatures. Control/type changes and static keyword grants remain gated; predicates use
effective supported temporary keywords and combat state.
All admitted modifiers commute: no base-setting/switching/dependency effects are
silently approximated as additions.

A compact live-source ID index is updated by central zone transitions and token
births. It is owned by State and cloned/restored with the rest of the engine.
Stat queries traverse that index without scanning libraries/hands or allocating
mutable derived-stat caches. Cards entering later receive applicable static
bonuses immediately; removing a source removes its contributions immediately.

Resolving Grow/GrowCreatures instructions instead write independent until-end-of-
turn deltas to the affected incarnations. A mass instruction determines its set
on resolution; creatures entering later receive no temporary bonus. Source-only
effects check the captured generation, so an old activation cannot pump a source
that left and returned. Zone changes reset deltas; cleanup removes deltas and
damage together before checking for resulting deaths. Temporary effects survive
their originating spell/ability or source leaving the battlefield.

Simultaneous zone batches capture every pre-event ObjectView before any object
moves, retaining contributions from departing lords in every last-known view.
Repeated SBA batches then discover deaths exposed by earlier source departures.
The source index and all temporary deltas are complete snapshot state.

The compiler requires complete supported text. This adds 40 Standard definitions,
including Anthem of Champions, Empyrean Eagle, Imperious Perfect, Goblin
Oriflamme, On the Job, Sugar Rush and Shivan Dragon. Coverage is 279/5,164,
with six regression cards and 28 token factories. Observation schema remains 12;
existing power/toughness projections report derived values.

Native tests check source-index lifecycle, lord stacking/exclusion, exact filters,
static entrants versus locked mass effects, combat, chained deaths, simultaneous
LKI, cleanup/counters, source reincarnation and branch/restore. Python loads the
same library and verifies anthem stats through cloning and replay.

Unimplemented layers include characteristic-defining values, base-setting,
switches, type/color/control changes, general ability changes and dependencies. Attachments,
variable/conditional bonuses and wider card/rules mechanics also remain gated.
This is shared additive infrastructure, not full layer-system or Standard
acceptance. The broader rules and RL/search goal remains open.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 611.2c, 611.3, 613.4c, 514.2–3 and 704.
