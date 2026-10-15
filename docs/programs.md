# Effect programs and trigger execution

Card definitions contain immutable typed instruction slices and trigger definitions.
State copies contain compact source context, stack identities, a program counter and
owned choices; they do not clone definitions, Java objects, closures or rule strings.
Spell programs use typed target rules and the existing timing/payment validator.
Legacy damage, growth, counter, draw and removal spells use the same interpreter.

Current instructions implement targeted damage/growth/counter/bounce/destruction,
non-targeted draw/discard/life gain/loss for a controller, opponent or each player,
and optional blocks. Programs are finite forward sequences: optional blocks skip
forward, instructions never recursively invoke programs, and choices resume the
same owned continuation. Recursive card interactions, loops and arbitrary control
flow have not been admitted.

A resolving spell remains in its stack zone while its stack entry is owned by the
continuation. No player receives priority during resolution, including optional or
discard choices. State-based actions wait until the whole program finishes. A
failed draw or zero life mid-program does not interrupt later instructions. A
completed instant/sorcery then moves to its owner's graveyard; a resolved triggered
ability does not move its source card.

Discard instructions collect choices in APNAP order, without moving or revealing
selected cards until all choices for that instruction are complete. Selected cards
cannot be selected twice. Each viewer can see their own selections. Exhausting a
hand completes that player's mandatory selection. Optional loot checks the choice
to pay, rather than whether a card was actually drawn (CR118.12): a player may choose
to draw from an empty library (CR121.3), must still discard if instructed, then loses
at the next SBA check. Draw replacements/prohibitions are not admitted yet.

Supported trigger conditions:

- This creature entering, dying or attacking; one ability may trigger on entry or attack.
- Another creature, or this/another creature, entering under its controller's control.
- Another artifact entering under its controller's control.
- A land entering under its controller's control.

Entry triggers inspect the post-event battlefield. Self-death triggers capture
source context across the entire simultaneous death set before any member moves.
Abilities queue until the next priority point, after SBAs. Active-player abilities
are placed first; each controller chooses bottom-to-top order when there is more
than one. Distinct stack IDs distinguish repeated abilities from the same source.
An ability survives source departure and preserves its original definition and
controller. Counterspell targets actual spells, not triggered abilities.

The compiler requires complete supported text, normal layout, fixed implemented
mana symbols, numeric stats and a nonlegendary supported creature body. All keyword
lines and every trigger/effect clause must compile. Extra activated abilities,
conditional triggers, unimplemented keywords/costs, or residual text reject the
whole template. The current catalogue supports 106/5,164 Standard identities.
[Target announcements](targeting.md) extend these programs with typed predicates,
multiple targets in one group, player effects, untapping and reanimation. Six additional regression cards
are outside Standard. Unsupported cards remain rejected at game creation.

Observation schema **5** adds stack kind/unique ID/source definition/context,
`resolving`, `waiting_triggers` and private `effect_discards`. New actions are
`PutTrigger { id }` and `ChooseOptional { accept }`; program and cleanup discards
both use `Discard { card }`. Wrong actors, stale revisions and unoffered choices
are rejected before mutation. Clones and snapshots include queues, ordering work,
resolving instructions, pending selections and future stack IDs. C signatures and
protocol response schema are unchanged. Neural encoders must handle these choices. Schema 6 replaces the old singular
stack target fields with bound target vectors and adds pending announcements; see
[targeting](targeting.md).

Still gated: multiple distinct target groups and announcement modes, mana/other optional costs,
intervening conditions, trigger-on-trigger abilities, other death watchers, batch
entry effects, event replacement/prevention, draw/life/discard observers, control
layers, tokens/counters, and the remaining mechanics. The implemented dispatch is
not evidence of general trigger or full Standard conformance.

Rules basis: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
118.12, 121.2c/121.3, 603.2–603.6, 608.2 and 704. Regression tests cover source
removal, ability vs spell targeting, distinct ability indices, simultaneous death
and APNAP ordering, optional decisions, private simultaneous discards, deferred
SBAs, atomic rejection, and cloning/restoring pending choices. Native
`standard_triggers` and Python shared-library games exercise supported Standard
decks. These workloads do not constitute full Standard training acceptance.
