# Selected source quantities and blocker subtype exclusions

Issue: https://github.com/altf4/rustmage/issues/406

This is a core dependency change. Drake Hatcher, Ravenous Amulet and Juggernaut
remain gated; it admits zero complete cards. Their complete pinned Oracle bodies
and the audited Java source hashes are in `fdn-card-quantity-sources.json`.
XMage is pinned to `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

`AmountValue::DamageEventAmount` reads the damage actually saved in the triggering
packet. `PutCountersScaled` can therefore put that many counters on the exact
source incarnation. Damage prevention, combat qualification and individual
player-damage event capture remain responsibilities of the existing damage and
trigger machinery. A departed source's replacement incarnation is not a recipient.

`AmountValue::SourceCounterQuantity(kind)` reads the current exact source object,
including sources outside the battlefield. When that incarnation departs, the
packet retains its last counter count. Only kinds referenced by its immutable
program are stored. Ordinary packets contain no allocated counter history.
Local activation packets refresh after counter and loyalty payment, immediately
before a source sacrifice, discard or exile. Packets already in state refresh at
zone departure; triggers created from a departed object use its frozen view.
The object table and damage-fact records gain no counter payload.

`StaticAbilityFragment::CannotBeBlockedBySubtype` rejects a blocking pair using
the shared effective subtype query, including changeling. The source fragment
must remain available after ability removal. It participates in the existing
blocking completion checks rather than creating a second declaration procedure.
Juggernaut's gated fixture carries its real Wall restriction alongside the
previous attack requirement.

The bounded core scenarios use ordinary typed instructions and the actual shared
payment routine. They do not install synthetic printed abilities on supported
cards. Final validation and performance measurements follow integration of the
counter-sidecar and subtype-layer parent cores. Complete consumer bodies and
independent reviews remain required before any support count changes.
