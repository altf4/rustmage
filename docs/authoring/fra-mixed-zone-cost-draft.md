# Mixed-zone cost draft limitations

CR 614.17b prevents choosing an impossible cost; 614.17c allows only a true
self-replacement effect to change an impossible event, before other replacement
or prevention effects. CR 118.11 treats a cost modified by a replacement as
paid. The current preflight handles ordinary prohibition before the existing
ordinary replacement provider, but does not model a resolving spell's or
ability's self-replacement; do not use it for an event with one. A static
replacement printed on a departing object is not automatically a self-replacement
under CR 614.15.

Non-entry commits revalidate every source's exact incarnation and zone before
any movement. Staged entry now carries an owned State checkpoint captured
before replacement planning, the selected component incarnations including
prevented ones, and an explicit unpaid-program counter. If a component changes
while entry choices are suspended, the batch restores that complete checkpoint
and resumes the unpaid continuation. Synthetic tests inject stale incarnations
while a real life-payment entry choice is pending, including a prevented
component; they are unbuilt and do not yet demonstrate a real nested move.
The prepayment checkpoint must be captured at the correct caller boundary and
replacement-choice replay, mana replay, trigger ordering, and failure semantics
must be reviewed and tested end to end before connecting this mechanism.

The staged helper is not connected to Sphinx or Emrakul. Tests are synthetic,
unlinked, and unbuilt. This draft must stay gated until end-to-end entry rollback, true
self-replacement, provider layering, full caller integration, and independently
reviewed guarded tests are complete.
