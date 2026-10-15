# Damage batches and combat continuations

Damage records retain source object ID, controller, keyword characteristics,
recipient, amount and combat/noncombat origin. These attributes are captured
when the event is constructed, rather than recomputed from a source that might
leave its zone. The current catalogue admits no control-changing or zone-return
cards, so controller equals owner; general control layers, generation references
and last-known-information lookup remain separate prerequisites.

All assigned combat damage is applied as one batch, followed by state-based
actions. Lifelink life gains occur inside that batch, before loss checks. Damage
from one source to multiple recipients is grouped into one life-gain event;
distinct sources remain distinct. Source grouping uses a deterministic map to
avoid a quadratic scan when many lifelink creatures deal damage simultaneously.
Life totals use signed 64-bit integers. Zero damage creates no damage/life-gain
event. Deathtouch marks damage for the next SBA check and resets afterward.

First/double strike introduces `FirstStrikeDamage` and a normal `CombatDamage`
step with priority between them. The state remembers who had a strike ability at
the start of the first step, including across snapshots. Normal damage uses that
history and current double strike rather than letting first strike hit twice.
Creatures killed in the first batch do not participate in normal damage.

Trample assignments are incremental and preserve every legal complete division.
Blockers can receive any distribution when no damage reaches the player. Damage
may reach the defending player only after all blockers have lethal assignments,
accounting for marked damage and deathtouch. The final blocker choice leaves any
legal remainder for the defending player automatically. This avoids dead-end
partial choices without enumerating all complete partitions. Trample does not
overflow while blocking, and a blocked trampler with no remaining blockers deals
its damage to the defending player.

Current cards can block only one attacker. Before admitting an additional-block
ability, extend assignment legality to account for simultaneous assignments from
multiple attacking sources to the same blocker. Before admitting prevention,
replacement, damage triggers or protection, extend event processing accordingly.
The existing kernel does not approximate those mechanics.

## Interface and verification

Observation schema **3** represents `damage_assigned` as structured `DamageEvent`
records and exposes the pending `damage_source` to the choosing actor. An
`AssignDamage` action now has `target: Target` and `amount` instead of `blocker`.
Other players cannot see partial damage assignments. Native C function signatures
are unchanged; JSON consumers should use the observation schema/fingerprint.

Tests include first-strike kills and retaliation, double-strike timing, instant
responses between damage steps, source attributes after departure, lifelink
preventing a simultaneous loss, per-source life-gain grouping, wide life totals,
marked damage, dead blockers, zero damage, exact restore during strike choices,
and all small two-blocker trample partitions compared with independent complete
assignment constraints, including deathtouch. Rust and Python supported Standard
self-play now use decks exercising these keywords. The smoke policy preserves
mana for main phases so it can cast creatures costing more than one mana.

Rules reference: Wizards' [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt),
sections 510, 702.2, 702.4, 702.7, 702.15 and 702.19. Tests establish this kernel's
implemented interactions, not full Standard semantics or all-card acceptance.
