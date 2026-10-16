# Simultaneous battlefield group effects

Issue: https://github.com/altf4/rustmage/issues/92

Fourteen complete definitions bring coverage to 981/5,164 pinned Standard cards,
six regression definitions and 43 token factories. There are 4,183 unsupported
identities. [The admission record](battlefield-group-admissions.json) lists all
printed bodies, source paths and SHA-256 values at Java revision
492369c7844b8d76479f0393ff9eb6b0b0cf30d7. Oracle facts are authoritative; the Rust
engine does not call Java.

The new immutable battlefield filters are non-targeted. They select kinds,
controller relation, tapped/untapped status, missing keywords, subtype, nonland
status and fixed mana-value ceilings. A union enumerates each physical object
once, including a creature/enchantment matching both kinds. Hexproof has no effect
on these sets. Membership and indestructibility are evaluated before any move.
All pre-event characteristics and death observers are frozen by the shared zone
batch, so sources dying simultaneously still see the other deaths.

`MoveBattlefield` destroys or exiles that frozen set. Destruction skips objects
with indestructible; exile does not. Tokens count when actually moved, then cease
to exist at the next state-based check. The resolution owns an exact `moved_count`
for Fumigate's life gain and Death Begets Life's draw quantity. Later Aura SBAs and
newly returned objects are excluded from the original result. Linked-exile returns
may pause for attachment/life entry choices before the tail runs; the result and
program counter survive snapshot, clone, restore and action-capacity rollback.
No state-based action or trigger resolution runs between ordinary instructions.
Failed draws still allow remaining instructions before the final loss check.

`DamageBattlefield` freezes recipients and constructs one source-aware damage
batch, including players when the printed clause deals damage to both players
and creatures. It uses the shared damage kernel for life loss, damage history,
lifelink aggregation, deathtouch and later SBAs. Seismic Rupture checks current
flying; Tectonic Hazard affects only the opponent and their creatures. Existing
player-only damage IR handles all-player clauses. Source-reference handling is
shared with targeted damage, including last-known sources that left the field.

The compiler accepts only exact complete clauses and existing complete tails,
modes and cycling abilities. Exact sentence-start card-name self-references now
normalize after an earlier sentence too, admitting Cast Away Doubt and Extended
Absence without dropping their damage clauses. Fumigate and Death Begets Life's
result instructions compile only with their exact preceding destruction body.
Unsupported variable costs, choices, replacements, conditional tails and residual
text remain gated. In particular, Dragonback Assault and Splatter Technique stay
unsupported: damage to a planeswalker removes loyalty, and that kernel is tracked
in [issue #93](https://github.com/altf4/rustmage/issues/93). Their creature portions
are not admitted alone.

Observation schema **37** adds `group_move_count`, the public result while a group
resolution remains suspended. It reveals no hidden cards. Optional control schema
2 and ABI 1 are unchanged. Counts are owned by the resolution, without adding a
per-card mutable allocation or a second game-state graph.

Author checks pass 596 Rust tests (587 unit plus nine integration), 104 compiler/
data tests, 71 release Python tests, catalogue consistency, rustfmt and Clippy with
warnings denied. Focused native fixtures execute every admission/mode and both
cycling abilities. They cover simultaneous dying observers, indestructible versus
hexproof, token counts/cessation, creature/enchantment deduplication, exact result
tails, no intermediate SBAs, empty draws plus damage, linked returns, opponent
entry choices, result-count visibility and capacity rollback after departures.
Synthetic fixtures explicitly identify future Spacecraft objects, valid linked
states and spell keyword grants; they admit no unsupported body. Public Python
play verifies actual Fumigate counts/life and Tectonic Hazard's opponent/creature
batch, with clone/restore/stale-index checks.

One hundred `standard_battlefield_groups` games use two pairs of supported 60-card
Standard decks and end naturally. Independent public checks cover affected moves,
surviving marked damage/history, lethal damage deaths, life changes and exact
counted draws: 85,507 decisions, 1,654 batch facts, 18,872 cloned/replayed boundaries,
and 19,980 maximum initialized state bytes. All modes/other bodies are covered by
focused tests; this mixed rollout is not full format acceptance or a Java speed
comparison. Byte counts exclude spare capacity, allocator/reference-count metadata,
external snapshots and immutable definitions. The full Standard/RL goal remains
incomplete.

A local release `benchmark 1000` smoke run produced 571,049 decisions at 404,681
native decisions/second, clone 182 ns, restore 88 ns, and 17,616 initial logical
state bytes. This is one unpinned short run with native smoke policy/observations,
without JSON, encoding or inference; it is not a before/after speedup estimate.
The atomic-checkpoint tradeoff recorded in [numeric-capacity.md](numeric-capacity.md)
still applies and needs further profiling as complete Standard decks become runnable.
