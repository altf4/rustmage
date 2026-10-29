# Radiation source-less stack audit

Radiation is an inherent player-triggered ability with no card or object as its source. Its owned stack packet stores `None` in `SpellView::source()` and records the affected player in `StackKind::InherentRule`. It does not expose a fabricated card or object identity. See [the source migration](tam-typed-source-migration.md) for the Rust and schema 190 API changes.

At the active player's precombat main phase, a player with rad counters gets the source-less trigger. Its fixed target-free program checks the current rad count on resolution. It mills that many cards and counts nonland cards actually milled to public zones, including exile under a replacement; it then causes that much life loss and removes that many rad counters. Stack copies retain the affected player. Countering the ability prevents its resolution.

Milling uses a simultaneous zone-change batch and replacement planning. The pending trigger, replacement choices, copies and rollback snapshots are owned. Linked rules tests cover both seats, source-less observations, copied and countered triggers, current counters, public-zone replacements, ordering and snapshot replay. Player-counter and proliferate tests cover separate choices, replacement ordering and mixed permanent/player batches.

Source-specific instruction executors still require a real object source. The radiation program does not use those operations; future inherent programs must audit their own paths and must not manufacture a source to satisfy a physical-source requirement. Tam remains gated pending its remaining branch evidence and independent final review.
