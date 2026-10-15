# Architecture and migration

## Library boundary

The engine is a synchronous state machine. One `apply` commits one primitive
decision and runs forced work until the next decision. Its inputs are typed
actions, not transport messages, threads, callbacks, or English prompts. Full
state remains inside the simulator. A policy gets a deliberately projected
observation. Rewards and training algorithms belong outside the rules engine.

The Rust API is the primary interface. `rlib` serves Rust callers and native
search. `cdylib` serves Python/other languages through a C ABI. The JSON control
plane is useful for a first binding and diagnostics; it is outside native
rollouts. Networking can be added as an adapter if deployment eventually needs
it, with no server behavior embedded into game state.

## State and branch costs

Card definitions are immutable enum/static data. Objects occupy a contiguous
arena indexed by small integer IDs. Libraries are vectors of IDs with the top at
the end. The initial slice has no ability object graphs, UUIDs, per-card strings,
reference cycles, serialization-based copies, or per-game asynchronous tasks.

An owned clone is the first implementation. It copies mutable object fields,
zone ordering, stack, combat declarations, damage assignments, pending decisions,
mana, turn bookkeeping, mulligan state, result flags, and a per-game SplitMix64
state. The RNG's bounded sampling uses rejection to avoid modulo bias.

`Snapshot` is a typed, reusable state copy. `restore` is rules-state restoration;
the outward revision continues advancing to reject stale action indices.
Native `Game::clone` creates an isolated branch. Python cloning forks the current
game but deliberately does not copy the session's saved-checkpoint map.

Measure before switching representations. As the rules kernel grows, consider
page-granular copy-on-write arenas and separating rarely changed from frequently
changed fields. An undo journal may reduce write volume but makes every new
mutation and suspended continuation responsible for inversion. Retain owned
snapshots as a correctness baseline if journaling is introduced.

The benchmark reports initialized state payload, not process RSS, allocator
overhead, spare capacity, or a whole search tree. It measures clone and restore
separately, plus complete limited-card games with setup, smoke-policy observation
encoding, action generation, validation, and natural terminal outcomes. This is
not a comparison with the existing Java benchmark: the rules/card workloads and
decision decomposition differ. Expand benchmarks as mechanics are migrated.

## Rules expansion requirements

The simple effect enum is intentionally limited to the accepted catalogue. The
next kernel needs explicit events, replacement/prevention choice continuations,
trigger queues with APNAP ordering, SBA fixed points, and continuous-effect
layers/dependencies. Avoid a growing list of card-name exceptions in `apply`.
Target/cost/mode choices should share the same validation logic for masks and
submission, with intermediate selection kept private until committed.

Current IDs are stable because no accepted card can reenter the battlefield or
be recast from a graveyard. Before adding those mechanics, replace target IDs
with index + zone-change generation references and implement last-known
information. Before adding control-changing effects, distinguish owner from
controller everywhere. Before adding damage triggers/prevention, represent
simultaneous damage as an event batch carrying source and target information.
These are prerequisites for card admission, not optional compatibility details.

The current rules reference is Wizards' [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
Fixtures cover starting/turn flow, priority, payment, stack, combat, and losses.
The test suite explicitly permits every current multi-blocker damage division;
do not reintroduce the obsolete damage-assignment ordering rule during migration.

## Tracked milestones

- [#1 Full rewrite](https://github.com/altf4/rustmage/issues/1): remains open until
  supported formats/card coverage and migration acceptance are established.
- [#2 Library foundation](https://github.com/altf4/rustmage/issues/2): first PR;
  deterministic branchable games, legal actions, private views, native/Python API,
  regression tests and reproducible benchmarks.
- [#3 Rules kernel](https://github.com/altf4/rustmage/issues/3): event processing,
  triggers, replacements, layers, object generations, costs and casting.
- [#4 Card migration](https://github.com/altf4/rustmage/issues/4): inventory the
  32,000+ Java implementations by required mechanics, establish a card IR and
  machine-readable support manifest, and port card-specific fixtures.
- [#5 Training/search integration](https://github.com/altf4/rustmage/issues/5):
  batched encoding/inference, worker pools, hidden-information search, and the
  existing Python RL pipeline. Search cutoffs are truncations, not losses.

Each implementation milestone should arrive in a reviewed PR with tests and an
updated coverage manifest. Do not mark the migration complete based on the
initial subset's throughput. No general-format or unsupported-card training
should proceed on approximated rules.
