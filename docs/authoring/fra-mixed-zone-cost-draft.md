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

Sphinx now calls the staged planner with an owned failure continuation. The
copy movement change reconciles the existing copy and SBA machinery: a spell
copy can enter exile during a resolving payment and ceases at the next SBA,
while card-only graveyard replacement effects exclude copies. The public-action
copy test is unbuilt. The wider copy-card audit remains incomplete on this
branch: historical copy-aware card-only consumers and zone-event identity from
commits through 239f8bb5 have not been ported. Do not admit Sphinx or treat this
as a generally reviewed lifecycle implementation before that audit, supported
replacement/entry interaction coverage, and guarded validation complete.

The synthetic competing exile redirect fixture is a plumbing test: it redirects
an instant/sorcery card into the battlefield, so it does not establish a
rules-reachable entry event. The separate Watery Grave staged-entry fixture
exercises a real entry-life choice with a constructed replacement plan, not a
real supported replacement provider.
