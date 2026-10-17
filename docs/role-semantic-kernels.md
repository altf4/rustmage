# Role semantic kernels — issue 295 / draft PR 297

The complete bodies are Return Triumphant, Twisted Sewer-Witch, Syr Armont,
the Redeemer and Asinine Antics. The same generic predicates also complete
A Tale for the Ages and Mystical Tether. Both additional admissions have explicit
paid tests and complete source pins. Full Standard remains incomplete.

Return Triumphant stores an owned completion on its entry transaction. Only a
successful battlefield result creates a Young Hero Role, attached to that exact
new incarnation. Skipped moves, replacement destinations and a target that left
and reentered do not attach. The completion captures the Role creator separately
from the returned creature's controller, preserving owner-controlled returns.

Mass Roles freeze actual matching creature references. Rat matching includes
Changeling and actual creature control. One replacement event stores a group for
each recipient; doubling preserves bindings, while added Frogs remain unbound.
Stale recipients never shift later group indices. Roles enter together and use
the existing per-controller timestamp/SBA rules. Public `role_attachments`
exposes these frozen group bindings to both viewers, including unbound additions.
Ordinary token events allocate no recipient vector. Counter privacy is unchanged.

Syr Armont and A Tale use an exact attached-Aura predicate. The recipient must
currently be on the battlefield with an exact attached battlefield Aura. Its
controller determines the lord's recipient set, independently of Aura control.
Equipment does not count. An Aura remains attached until state-based actions
remove an illegal attachment; the predicate does not remove it prematurely.

`AnnounceFlashSurcharge` and `CastInfo.flash_surcharge` declare the optional
timing cost independently of casting method. Normal timing retains the cheaper
route. The extra cost precedes reductions, uses the caster's resources and
respects exact foreign-exile permissions. It does not grant a Flash keyword.
Mystical Tether retains the existing linked exile/return semantics. Unimplemented
combinations of Warp and paid flash remain gated. Observation schema 87 includes
the cast declaration and Role replacement bindings.

The audit-only [source manifest](role-semantic-admissions.json) pins all six full
bodies and resolvable direct imports: 201 records / 99 unique complete Java files
at `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Shared Role factory sources are
included. This does not claim all transitive Java dependencies were rewritten.
No provenance inventory or name grants runtime permission.

Validation remains in progress. Focused native cases cover both seats, actual
payments, foreign ownership/control, timing, reductions, insufficient costs,
stale targets, successful entry results, mass replacement ordering and owned
rollback. Ten focused compiler/provenance methods pass. The exact six-card
comparison preserves all other parent definitions, helpers, colors and recipes.

At production checkpoint `69c16d3`, a focused public policy run passed 16
naturally terminal Standard games: 14,033 owned frames and 28,066 illegal-action
denials. It includes both seats, cheaper normal timing routes, Young Hero attack
counters, Wicked death life loss and paid off-turn Asinine through opposing Royal
hexproof. Immutable library SHA256:
`c3613807780ab01e4e7be2ded4a7ece6edfa7a8c327229f5e77daa0169d2305c`.
This is checkpoint evidence: later Role observation and compiler closure changes
require fresh native/library/public validation before final delivery.

Prototype policy failures were scoring issues: finishing attacker selection
outranked selecting attackers, and mana filtering excluded Three Tree Mascot's
legal conversion. Both were corrected before the 16-game pass. Failed runs are
not final evidence. Full native/Clippy/compiler/API gates and two independent
NONAUTHOR final-head reviews remain required. Every completed build target is
immediately cleaned of non-SO outputs; every shared library retains its exact
original path and hash.
