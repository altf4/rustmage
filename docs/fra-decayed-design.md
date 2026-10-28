# Decayed counters and attack triggers

This is a design and test checklist, not an implementation. The Mabel draft
can represent and remove decayed counters, but it must not be treated as proof
that a permanent with one plays correctly.

The [June 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260619.pdf)
identify decayed as a keyword counter (122.1b). Decayed grants both a
blocking prohibition and an attack-triggered ability (702.147a). Each
instance of an ability functions independently (113.2c); a Boolean indicating
that any decayed counter exists cannot represent multiple attack triggers.
Counters of the same kind share the newest such counter's timestamp (613.7c).
They are interchangeable, but the number of independently granted abilities
still matters. Printed or separately granted instances must also remain
independent of the counters.

The existing `engine_keyword_counters::sync_effects` deliberately coalesces
one counter kind on an object into one timeline row. Any extension needs to
preserve that row's refreshed timestamp and the number of applicable grants
through layer-six evaluation, including later removal of abilities and later
counter additions. Merely adding `CANT_BLOCK` to the row loses the triggered
ability; directly checking raw counters during combat bypasses ability loss and
timestamp order. Speculative characteristic worlds and copied objects must
use the same evaluation, and counters are not copied characteristics.

At attack declaration, enqueue one ordinary attack trigger per surviving
instance, with its source incarnation and controller. A creature entering
already attacking has not been declared as an attacker. Resolving an attack
trigger creates an owned delayed trigger for the next end-of-combat event;
countering that trigger creates none. Losing decayed, removing the counter or
changing characteristics after the original trigger is queued does not undo
it. The delayed trigger is placed on the stack, rather than sacrificing
immediately at step entry, and its controller gets the normal priority and
ordering opportunities. The delayed ability follows its particular object,
not a later incarnation; the normal sacrifice path and replacement effects
must apply, including control changes. Rules 603.7a–e cover delayed-trigger
creation, timing, identity and controller. If an attack trigger resolves after
an end-of-combat event, the next such event is relevant.

`engine_mobilize::queue_delayed_sacrifices` currently handles end-step
sacrifices and is not a substitute for an end-of-combat event. Any new pending
state must participate in observations, memory accounting, cloning,
raw/index branch and restore, and schema reconciliation with the other drafts.

Required rules tests include:

- One and multiple decayed counters, separately granted or printed instances,
  and counters added at different times around an ability-removal effect.
- Legal blocking, declaration versus entering attacking, both stack priority
  windows, countering the initial or delayed trigger, and multiple combats.
- Removing the counter or the ability before and after declaration; removal
  after the trigger is queued must not erase that trigger.
- Source departure and reentry, type changes, control transfer and return,
  sacrifice replacement effects, copied characteristics and counter movement.
- Transactional failed actions, private observations where applicable, and
  equivalent outcomes after owned raw/index branch and restore.

The same official counter list includes exalted and shadow; review their
representation and behavior before claiming complete coverage of keyword
counters. This checklist does not establish whether any particular pinned
Standard card can create those counters.
