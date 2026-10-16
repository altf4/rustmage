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
reject the whole card. Observation schema 35 retains the same fields with widened exact integer ranges:
64-bit unsigned counter storage, signed 128-bit derived stats/damage/life and
128-bit captured damage totals. Printed instruction constants stay compact.
Checked arithmetic prevents release-mode wrapping from turning a large positive
toughness into a false death. JSON integers remain numbers with exact digits;
Python receives ordinary arbitrary-precision integers. The control decoder also
handles signed 128-bit damage through nested requests and generic JSON values.
The optional JSON control layer enables serde_json's arbitrary_precision feature;
native Rust rollouts perform no JSON conversion.

This widening follows an independent reviewer finding in repeated legal Zimone
play. Tests cross the old signed-stat and unsigned-counter boundaries, complete
40 public self-targeted doublings, and preserve huge damage/lifelink/captured
event quantities. It increases fixed state size; the 100-game fixture reaches
18,764 initialized state bytes. Counter storage and stat arithmetic still have
finite checked representation limits; arbitrary native precision and a fallible
capacity-exhaustion interface were outstanding at PR #87. The latter is now
implemented by the [recoverable capacity contract](numeric-capacity.md); arbitrary
native precision remains outstanding. Values are never approximated by saturation
or false game outcomes.

Regression tests exercise every admission, legality/privacy of target selection,
source changes, live sets/changeling matching, exact mana/tap/additional costs,
partial fizzle, death ordering, cleanup, stun and replay. The explicitly synthetic
IR fixture verifies no mid-resolution SBA without admitting an unsupported card.
`standard_counter_groups` uses supported 60-card Standard decks and independently
computes expected counter values from public before/after observations, while
also cloning and restoring target/payment/resolution boundaries. Python tests
exercise mass-counter deaths and Zimone's activated doubling through the native
library. Full Standard acceptance and the wider RL integration remain incomplete.

A second independent finding concerned damage assignment after large growth.
Enumerating every integer up to the attacker's power is no longer used for
large ranges. `Game::legal_action_space()` exposes the complete union of concrete
choices and `DamageAssignmentRange { source, target, minimum, maximum }`. Every
integer in the inclusive range remains legal and `Game::apply` validates its
bounds and target directly in constant space/time. Native callers may query
`damage_assignment_range()` without allocating any action vector.

`legal_actions()`/`legal_actions_into()` and observation `actions` provide concrete
indexed conveniences. Damage ranges up to 64 wide remain exhaustive; larger
ones offer distinct minimum/lethal/maximum representatives. These representatives
do not restrict the legal range. Observation schema 35 exposes the full
`damage_assignment` to the actor, and Python `Game.act(observation, action)` sends
arbitrary legal amounts through the revision-checked native control path.
Indexed clients can still use `step`; models/searches needing interior numeric
choices must use the range and `apply`/`act`. Tests cover huge double-blocked
states, trample minima, interior values absent from the indexed choices, invalid
bounds/targets/actors, privacy, exact JSON input and snapshot/clone replay. The
public 40-doubling Python fixture then attacks into two real blockers and submits
an interior assignment without enumerating the trillion-value domain.
