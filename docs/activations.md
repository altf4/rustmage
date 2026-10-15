# Activated abilities and permanent programs

Battlefield abilities have immutable cost, target, effect-program and restriction
definitions. An activation creates an independent stack identity and captures its
source incarnation, controller and keywords. Removing or sacrificing the source
does not remove its ability; a returned source is a new incarnation. Abilities
cannot be targeted by effects that target only spells.

Legal-action generation checks controller, battlefield presence, exact colored
and generic mana, tap status and creature summoning sickness, life/counter
availability, sorcery timing, and admitted once-per-turn restrictions. Haste
bypasses the sickness restriction for tap costs. Abilities without tap costs
ignore sickness. Fixed counter-removal costs enumerate distributions by kind,
including mixed payments, rather than subsets of individual counters.

`Activate` includes an optional single target, exact mana payment and typed
`counter_payment`. Bounded multi-target and targeted counter-cost abilities use
`BeginActivate`, followed by the shared target-selection continuation. Costs
remain uncommitted until all targets are selected. The continuation owns pending
mana/counter payments, source context and bound targets. No other game action
can intervene during an announcement. Invalid submissions leave state and
revision unchanged.

Costs commit before priority and before the ability resolves. Cost-generated
triggers are queued and placed above the activated ability. Sacrificed objects
therefore are not newly available when initial targets are selected. Resolution
uses the same partial/all-invalid target rules, program continuations and
snapshot semantics as other stack objects.

Simple noncreature artifacts/enchantments now use permanent programs, including
entry triggers and battlefield activations. Unconditional fixed entry counters
are initialized before entry triggers and state-based checks, and reset on later
zone changes. Moonlit Lamenter and Gnarlbark Elm exercise entry counters and
removal of counters of any supported kind.

Observation schema **9** adds activation actions, the `ActivatedAbility` stack
kind and `ObjectView.used_abilities`. The latter is a per-ability bit mask,
owned by the object and cleared each turn or zone change. Catalogue definitions
remain shared immutable data. Snapshot state includes all costs, flags and
private choices required to replay an activation.

This milestone adds 29 complete Standard definitions, bringing executable
coverage to 171/5,164. Unknown costs/restrictions, variable costs, mana abilities,
discard/sacrifice-other costs, granted abilities, modes, other activation zones,
and unsupported additional card text remain gated.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 113.7, 118, 601.2b–i, 602 and 122.6.
