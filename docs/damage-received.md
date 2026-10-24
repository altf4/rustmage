# Actual creature damage packets

`TriggerCondition::CreatureDamaged` observes positive actual damage to a creature,
with a controller relation qualified at the event. `DamageReceivedView` captures
one exact recipient reference and the checked sum of damage from every source
and assignment fragment in that simultaneous batch. Separate creatures and
separate batches, including first-strike and regular damage, remain separate
occurrences. `AmountValue::DamageReceivedAmount` reads the frozen packet.

Dispatch occurs after damage replacements, prevention and damage results including
lifelink, before state-based actions. Current post-event types, controllers and
available abilities qualify the event (CR 603.10). Lethally damaged creatures and
observers still exist at that boundary. Later removal, transformation or control
changes do not requalify a queued packet. The optional boxed packet adds no heap
allocation to ordinary spell rows; initialized packet bytes are counted by the
existing owned-state accounting.

Jennifer's back uses an ordinary required any-target plan, followed by
`MayOncePerTurn`, `RecordOncePerTurn` and `DamageScaled`. Declining, canceling a
packet or losing all legal targets does not record use. Accepting records use
before outgoing damage or its replacement choices; subsequent events and queued
siblings cannot reuse it, even if the outgoing damage is prevented. Uses belong
to the exact source incarnation and ability slot, independent of controller.
Source damage reuses current exact source characteristics or the latest saved
battlefield departure facts and keywords, even after that card returns as a new
front-face incarnation.

The pinned Java Jennifer class passes `SetTargetPointer.PERMANENT` to its reusable
received-damage helper, which installs a fixed pointer to the damaged victim on
its effect while the class also adds `TargetAnyTarget`. Rust follows the frozen
Oracle's any-target choice: the packet's recipient records event identity and
never becomes an implicit damage target. The [official release notes](https://magic.wizards.com/en/news/feature/marvel-super-heroes-release-notes)
confirm per-victim summation, damage after source departure, and suppression of
queued siblings after acceptance. Post-event qualification follows the
[Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260619.pdf),
603.10.

This core's first whole representative is Jennifer Walters // The Sensational
She-Hulk. Bruce Banner and Red Hulk remain gated until this prerequisite is
independently reviewed and integrated. Bruce additionally requires its XX
activation cost and current-or-latest-departure attacking status for extra combat.
This change does not implement separately acquired in-place copy ability identity
or expand dependent cards. The full 281-card MSH acceptance goal remains open.

The focused boundary regression combines a paid King back face's prevent-all-self
replacement with preventable 5 and unpreventable 2 damage in one batch: exactly
one received packet holds 2. Its lifelink source is equipped with the admitted
Aettir and Priwen, so the same event changes its real base power from 20 to 22;
the queued source snapshot must contain 22. The current admitted pool and typed
providers have no life-dependent creature-type or controller effect. Direct
post-lifelink type/controller qualification therefore has no honest live-card
reproducer here; its ordering is established by inspection of the central commit
path, not a claimed synthetic-card test.
