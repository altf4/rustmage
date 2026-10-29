# Emrakul, the Exigent Doom draft

Pinned Oracle 4421ab7d-6d9b-4edd-b5a0-53a8ed84da6f and XMage revision
492369c7844b8d76479f0393ff9eb6b0b0cf30d7 are recorded in the provenance file.
The partial typed draft implements 12/12, flying, trample and the on-cast
untap-all-controlled-lands trigger, independent of spell resolution. Tests are
unlinked and unbuilt; no completeness or throughput claim is made.

Two printed obligations are not yet implemented. Ward requires a payer-owned,
optional atomic selection of exactly three controlled permanents, validation of
all selected incarnations and sacrifice prohibitions before committing, one
simultaneous sacrifice event with replacement choices and correct unpaid
countering even if the original ward source has left. The choice, payer,
related stack item, partial selection, replacement continuation and rollback
must remain owned, private/public as appropriate and serializable; countering an
uncounterable spell must still allow paying or declining.

The {3} hand activation must exile the source as a mandatory cost, target any
land, and on resolution separately grant that exact land a layer-six {T}: {C}{C}
ability and give the captured player permission to cast the exact exiled card.
The cast permission lasts only while that incarnation remains in exile. The
land grant ends when the linked card is cast from exile, not merely when the
card leaves exile or changes controller; exact timing, departure and return,
ability suppression order, copied abilities, funding projections and rollback
need independent tests. A countered or targetless activation must not create
either resolution effect. Both features need shared mechanic implementation,
compatibility/schema review and guarded validation before this card can be
marked complete.
