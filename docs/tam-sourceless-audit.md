# Tam draft: source-less radiation audit (unvalidated)

This is an isolated draft and not an admission claim. Radiation is an inherent
player-triggered ability with no object or card as source. The prototype uses
sentinels internally and serializes source fields as null; it remains gated.

## Reached paths

* `begin_step(Main1)` queues an `InherentRule::Radiation` packet only if the active
  player has rad counters; ordinary APNAP ordering and announcement operate on
  its fixed target-free program. `resolve_top` rechecks the affected player and
  executes only `ResolveRadiation`.
* `target_spec`, `modes`, and announcement cost/mana-choice methods explicitly
  handle `InherentRule`, with no immutable source-card lookup. Stack ability
  copy preserves kind, affected player, source marker and program; explicit
  countering removes a nonspell stack item. Spell-only filters short-circuit
  before source-card metadata; ability-source-kind matching uses a bounded
  lookup and zero source facts.
* `mill_radiation` uses actual zone-change events, and the action checkpoint
  suspends competing zone replacements before committing the mill batch. The
  pending trigger, replacement choices, copies and rollback snapshots are owned.
  The isolated radiation tests cover both players, current counters, source-less
  wire fields, copied/countered triggers, APNAP ordering and competing choices.
* Generic source-capture passes that run during milling either match exact real
  object references first, check `TriggeredAbility` before definition access,
  or use bounds-checked battlefield lookup. Source-specific instructions such as
  grant-self ability, temporary trigger creation and source-return never occur
  in the fixed `ResolveRadiation` program.

## Blockers

* The public Rust `SpellView` fields still expose the `CardId::MAX` object and
  `Card::SOURCELESS` sentinel, even though serialization emits null and the
  `source_identity()` accessor returns `None`. A caller using the historically
  public source card's normal `Card` metadata methods can index the catalogue
  with the sentinel and panic. The object ID is similarly not a real object.
  Replace public identity with an optional or typed representation and migrate
  all consumers before declaring this feature safe for the in-process API.
* The full source-field scan contains hundreds of direct accesses. Reached-path
  checks above are not a substitute for exhaustive migration to typed sources;
  adding any new source-less program or copying one into a new execution path
  must not silently inherit a physical-source assumption.
* New tests and schema guard are isolated and unbuilt; the latter must be merged
  with concurrently developed schema changes. APNAP player-counter split and
  replacement behavior also needs independent review and guarded validation.
