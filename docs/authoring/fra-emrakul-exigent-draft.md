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

### Funding termination blocker

A prevented, non-tapping sacrifice-mana cost can be paid repeatedly while its
source remains in play. A forward search over exact mana counts can therefore
visit arbitrarily many states without enabling the enclosing cost. In
particular, artifact-spell-only mana cannot pay an ordinary cost; the bounded
`restricted_repeatable_mana_produces_distinct_but_unpayable_projected_states`
fixture demonstrates successive unpayable states without running an unbounded
probe. A missing-color upper bound solves only the simplest case.

A candidate exact solution for a *proven finite-control, monotone* subset is
backward coverability using minimal antichains. Its resource vector must track
ordinary mana (the aggregate pool less all tagged lots) and each distinct
mana-quality/color lot independently. Its control node must preserve exact
object incarnations, live characteristics, replacement choices, ability uses,
linked and copy-source facts, and the complete nested payment phase. Each
backward transition must preserve actual selectable payment allocations and
quality restrictions; successful nodes need predecessor witnesses so public
actions, provenance and real mana amounts remain exact. Dickson's lemma yields
termination only if the vector has a fixed finite dimension, the non-mana
control graph is finite, and all transitions and the success condition are
monotone in that vector.

Those properties have not been proved for the current engine. In particular,
entry/reentry and counter effects can change control facts, dynamic production
can change quality or quantity, and nested cost continuations expand the phase
space. An arbitrary cap, forward pruning of a larger mana state, or treating all
colored mana as interchangeable can discard the only legal route. Keep this
mechanic and Emrakul gated until the supported transition subset is formally
bounded and checked against actual payment and rollback behavior.

The current draft also normalizes a spell-only lot in a standalone fixed-purpose
search when that lot is ineligible for the goal. Such mana cannot fund any mana
activation, and the frozen spell/activation purpose cannot spend it; all other
quality, non-mana state and exact pools are retained. This does not generalize to
an eligible lot. For example, an artifact-only green producer with a prevented
non-tapping sacrifice and a generic-to-blue converter can generate arbitrarily
many distinct useful artifact-spell lots even when a blue artifact-spell goal
cannot be reached: the converter cannot spend those lots, but the color upper
bound can count its blue production. Dropping or arbitrarily capping the green
lots would need a proof that no later reachable goal or cost can use them.

The fixed-purpose test is unlinked and unbuilt. Full termination still requires
a finite-control argument and a sound coverability treatment across nested
payment choices, conversions, and quality-changing or source-changing events.

### Candidate spendability reachability (not implemented)

A finite, conservative color/restriction closure can reject a subset of missing
color goals without changing exact payment state. Begin with the colors available
in each actual mana quality. Abstract any positive amount to an unbounded amount
of that same quality and color. Enumerate a conservative superset of mana
abilities and their possible production qualities/colors. Add an ability's
outputs only when its colored and generic mana cost is satisfiable from classes
whose restrictions permit spending on that activation; ignore all non-mana
costs, timing, limited uses, and required amounts beyond positivity. Repeat to
fixed point. If no class eligible for the fixed final purpose can supply a
required color, and no color-substitution or non-mana payment applies, no exact
payment is possible. On the bounded artifact-only example this excludes the
converter: its activation cannot spend the only available class.

This argument relies on an exhaustive transition inventory. The current
`mana_activations` and `granted_mana` inspect one world: sacrifices can alter
control, copy identity, layer-six abilities and graveyard borrowing; linked
returns, entry replacement and nested payment can introduce more providers.
An implementation must either enumerate all such providers and qualities as an
overapproximation or guard the optimization with independently established
invariants and fall back whenever any provider might change. In particular, an
unavailable activation must not be omitted solely because it is unavailable in
the starting state. Bonus/source provenance remains exact in the actual search;
the abstraction is only a negative reachability test. This remains a proposal,
not a complete termination proof or a production pruning rule.
