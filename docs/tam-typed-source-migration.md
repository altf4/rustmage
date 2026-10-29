# Source-less stack identity migration (draft)

An inherent player-rule trigger has no source card, object or source facts.
A max-value `Card` or `CardId` cannot safely represent that absence in the
in-process library: metadata and object lookups expect real values.

The staged `SpellView::source()` accessor returns `Option<StackSourceView>` so
consumers can migrate before the old fields are removed. It is a transition,
not a complete implementation: the current packet still contains sentinels.

The final owned packet should store a single typed source, for example
`Option<StackSourceView>`: `Some` contains the exact object incarnation,
source card and last-known facts; `None` represents an inherent rule. Spell and
activated-ability constructors must populate `Some`; the radiation constructor
must populate `None`. Generic target, copy, ordering, rollback, observer and
serialization paths must branch on that value. Physical-source-only instruction
executors should require a real source at their typed boundary, while inherent
programs should never acquire a fake source merely to satisfy those helpers.
Stack copies clone the option and retain the original affected player.

The current public `SpellView.card`, `object`, `source_card` and `source_facts`
are non-optional; removing or changing them is a Rust API break. Migrate every
example and in-repository consumer before removing the fields, using `source()`
to handle both cases rather than unwrapping for arbitrary stack packets. For
consumers requiring an actual spell or an activated ability, first prove the
kind and then handle the source explicitly. Migrate documentation and external
clients as well. The normal-spell path's historical cast/source distinctions
must remain intact.

A public source field or removal of existing fields changes the observation wire
schema. Integrate one schema bump with concurrent work, reject prior-version
projection when a source-less packet or source-dependent hidden continuation
cannot be represented, preserve older version projectors and verify nested
packets, public views from both seats, copies, snapshots and rollback. Do not
silently serialize absence as a valid catalogue card.

Until the internal source representation, all consumers, examples, schema
projection and guarded tests are finished, this work and Tam remain gated.

## Schema 190 trace audit

The schema 190 source representation changes the public packet shape. The recorded schema 189 projection audit at `/home/altf4/.cache/fra-tam190.prior-mana-audit.json` matched all 2,928 prior supported-card traces. The schema 190 trace baseline was recorded from the same frozen integration inputs at `/home/altf4/.cache/fra-tam190.json`. This comparison does not establish full radiation or proliferate rules correctness; those require separate rules tests and review.
