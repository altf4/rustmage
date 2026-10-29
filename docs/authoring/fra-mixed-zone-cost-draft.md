# Mixed-zone cost draft limitations

CR 614.17b prevents choosing an impossible cost; 614.17c allows only a true
self-replacement effect to change an impossible event, before other replacement
or prevention effects. CR 118.11 treats a cost modified by a replacement as
paid. The current preflight handles ordinary prohibition before the existing
ordinary replacement provider, but does not model a resolving spell's or
ability's self-replacement; do not use it for an event with one. A static
replacement printed on a departing object is not automatically a self-replacement
under CR 614.15.

Non-entry commits now revalidate every source's exact incarnation and zone
before any movement. Staged battlefield entry supports frozen plans and existing
entry choices, but it does not yet own a pre-cost checkpoint or a failure
continuation. Should an entry choice or nested replacement change another chosen
cost component, a safe implementation must restore the complete prepayment
state (including life payments, pending choices, observer queues, RNG, and
replacement replay) and resume the caller's unpaid branch; merely checking a
reference or restoring one object cannot provide that guarantee. Presently
supported basic entry choices select a value/attachment or pay life, and none
identified directly moves another chosen object, but that is not a generic proof.

The staged helper is not connected to Sphinx or Emrakul. Tests are synthetic,
unlinked, and unbuilt. This draft must stay gated until entry rollback, true
self-replacement, provider layering, full caller integration, and independently
reviewed guarded tests are complete.
