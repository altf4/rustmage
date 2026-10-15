# Distinct target groups and noncombat creature damage

TargetPlan represents either one TargetSpec or an immutable array of independent
occurrences of "target". Spells, modes, triggered abilities and activated abilities
share the plan type. Each BoundTarget owns a group index and exact object
incarnation. Announcements own the current group, plan and current specification.
A group completes before the next group begins; optional empty groups can be
skipped. Payment follows all target groups.

Every required group must have enough candidates before announcement is offered.
Within an occurrence targets are distinct; separate occurrences may reuse an
object. The admitted plans have no cross-group constraints, so their completion
checks are independent and exact. Printed relationships between occurrences,
including unimplemented uses of "another", stay gated. Group-indexed bindings
survive sorting, partial selection, mana windows, clones and restores.

Resolution revalidates each binding against its own group's predicate. If all
chosen targets are illegal the entire spell/ability fails to resolve. Otherwise
remaining groups execute independently. SelectGroup controls the instruction
interpreter's target focus; that focus is owned by Resolution and survives private
library/optional/discard continuations. Non-targeted instructions retain their
normal player semantics. Fight and power-damage instructions explicitly identify
both participant groups rather than depending on target position or current focus.

Fight captures current power and effective keywords for both creatures before
applying one simultaneous damage batch. It is noncombat damage: first/double
strike, blocking and trample assignment do not apply. Lifelink/deathtouch do.
Nonpositive power deals no damage. Fighting itself deals twice its power in one
event. If either participant is absent, a new incarnation, no longer a creature or
an illegal target, neither fights. One-way power damage similarly requires its
admitted creature participants and deals no return damage. Prior growth/counters
and live additive effects contribute to current power. SBAs follow the complete
resolution, including later instructions.

Schema **16** adds group to BoundTarget and actor-only targeting_group during
selection. Opponents see neither partial targets nor the current group/action
mask. Completed stack bindings remain public. Immutable plans are shared; state
copies contain only indices, bindings and the existing continuation. state_bytes
accounts for their initialized storage through normal struct/element sizes.

Fourteen complete Standard definitions bring coverage to 537/5,164, six legacy
regressions and 31 token factories: Bushwhack, Combat Tutorial, Cost of Brilliance,
Giantfall, Huatli's Final Strike, Kapow!, Knockout Maneuver, Quarrel, Rabid Bite,
Rabid Gnaw, Rocky Rebuke, Tenderize, Troll Negotiations and Vibrant Outburst.
The compiler composes wholly supported independent targeted clauses and rebases
group operands, while preserving the whole-text gate for residual mechanics.
Planeswalker participants, source-sacrifice power/LKI instructions, general
cross-group relationships and remaining card/rules mechanics need further work.

Native fixtures cover exact manual ownership/hexproof masks, illegal declarations,
within/across-group distinctness, optional groups, all/partial revalidation,
incarnations, per-stage replay, buffed power, simultaneous lethal damage,
lifelink/deathtouch/strike behavior, nonpositive power, self-fight, and private
continuation focus. Python exercises the two group declarations and power damage
through the C ABI. standard_groups admits two Standard 60-card decks and exercises
fight, bite, modal spells, independent effects and partial target choices.

Source references include XMage Bushwhack, Quarrel, Kapow, CombatTutorial and
VibrantOutburst. Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 115.3, 601, 608 and 701.14. Full Standard/RL/search acceptance remains open.
