# Target predicates and owned announcements

Spells and triggered abilities share immutable `TargetSpec` definitions: a rule
plus minimum/maximum counts for one homogeneous occurrence of “target.” Predicates
cover current damage targets, creatures, actual spells, players/opponents, and
object-zone/type/controller/self-other filters. Physical card slots become bound
`ObjectRef` values when chosen. “Other” compares the original source incarnation,
not its physical card slot; a source that leaves and returns is a different object.

A required trigger with no legal announcement is removed. A trigger with a “may”
effect still needs its required targets before going on the stack; the optional
effect is chosen during resolution. APNAP/controller ordering suspends for each
ability's target announcement and resumes before granting anyone priority.

For one-target spells, `Cast` remains a fully validated combined action. Spells
with multiple targets use `BeginCast { card, payment }`, incremental
`ChooseTarget { target }`, then `FinishTargets` once the minimum is satisfied.
Choosing the maximum commits automatically. Target groups prohibit duplicate
choices. No subsets or Cartesian products of target sets are eagerly generated.
Completed homogeneous sets are canonicalized to avoid order-dependent search state.

A multi-target spell moves from hand to its stack zone during announcement; its
validated mana payment is retained until targets commit. No other player receives
priority and mana cannot be spent elsewhere during this continuation. There is no
cancel action; the initial action is offered only if completion is possible. Broader
costs, modes and casting observers remain gated. [Payment windows](payment-windows.md)
support mana abilities during the admitted fixed casting/activation costs.

Resolution rechecks each target's incarnation, zone and predicates. Illegal members
are removed. If every originally chosen target is illegal, none of the effect
resolves, including non-targeted life gain. Choosing zero targets for an optional
specification permits its non-targeted instructions to resolve normally. Surviving
targets receive the effect when other members become illegal. Instructions also
preserve references within resolution, so an object moved by an earlier instruction
cannot be confused with its new incarnation later in the program.

Targeted draw/discard/life effects use their selected players; multi-player draws
follow APNAP order. Untapping, targeted temporary symmetric stat changes, and
graveyard-to-battlefield reanimation share the interpreter. Reanimation clears old
object state, starts summoning sickness and dispatches the new entry triggers.
Multi-object zone changes capture death sources before movement. Source departure
does not remove an already announced ability.

Observation schema **6** replaces `SpellView.target`/`target_object` with
`targets: [{target, object}]` and adds `announcing`. Uncommitted target selections
are visible to their choosing actor; the pending source remains public. Committed
targets are public. Clone/restore includes the announcement, selected references,
resume context, future IDs and pending payment. State-size accounting includes all
initialized nested target buffers. C signatures and protocol response schema are
unchanged. Neural policies must handle the three new actions.

Exact complete-text definitions add 26 pinned Standard identities, reaching
106/5,164. Supported examples include Skeleton Archer, Vault Plunderer,
Reclamation Sage, Fancy Footwork, Thistledown Players, Zombify and Emerge from the
Cocoon. Instant/sorcery subtypes such as Lesson preserve ordinary casting behavior;
extra card types such as Kindred still fail these templates. Unknown instructions,
unequal stat changes, additional target groups/conditions, costs or keywords reject
the whole template. No ward/protection/hexproof source cards are admitted yet.

The framework handles bounded “up to” specifications, with synthetic regression
fixtures for zero-target versus all-targets-invalid behavior; this does not claim
that every card with those words is implemented. Modes, multiple distinct target
groups, distributed damage, most target restrictions, general costs, replacements,
continuous layers and other dependent mechanics remain gated. General reanimation
semantics for unsupported creatures or control-changing effects are not established.

Tests enumerate every complete one-or-two-creature target set on small boards,
exercise partial/all-invalid references, private pending choices, atomic rejection,
APNAP announcement resumes, source departure, optional effects with required
targets, and reanimation entry triggers. Native `standard_targeting` and Python
self-play replay both spell and trigger target choices through cloning/restoring.

Rules basis: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
115.1–115.6, 601.2, 603.3–603.5 and 608.2b.
