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

Card definitions are immutable generated static data. Two-byte card identities
index those definitions. Objects occupy a contiguous
arena indexed by small integer IDs. Libraries are vectors of IDs with the top at
the end. The initial slice has no ability object graphs, UUIDs, per-card strings,
reference cycles, serialization-based copies, or per-game asynchronous tasks.

Clones and snapshots share the object arena and library buffers through
`Arc<Vec<T>>`; the first mutation detaches a shared buffer. Other mutable state,
including zone ordering, stack, combat declarations, damage assignments, pending
decisions, mana, turn bookkeeping, mulligans, result and local RNG, is copied.
Branches remain isolated. Cloning is not constant time in all state dimensions.
The RNG's bounded sampling uses rejection to avoid modulo bias.

`Snapshot` is a typed, reusable state copy. `restore` is rules-state restoration;
the outward revision continues advancing to reject stale action indices. Restore
returns a recoverable error without mutation if that revision cannot advance.
Every action has an atomic checkpoint for checked numeric capacity failures; see
[numeric-capacity.md](numeric-capacity.md) for the contract and performance cost.
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

Card bodies and immutable instruction programs share static catalogue storage.
Owned resolving continuations retain a program counter, source context and pending
choices. Trigger instances use distinct stack identities, queue until priority,
and are placed in APNAP/controller order. See [programs.md](programs.md) for exact
admission scope. Wider events, replacement/prevention continuations, remaining SBA types and continuous layers remain prerequisites for dependent cards.
The implemented SBA rules repeat to a fixed point, including token cessation.
Target predicates now share validation between masks, submission and resolution;
[owned announcements](targeting.md) retain partial target choices and pending
fixed mana payment. [Activated abilities](activations.md) also retain deferred
mana/counter costs, independent source context and per-object usage flags. Wider
cost/mode choices still need the same treatment.
Intermediate selections remain private until committed.

Physical card slots remain stable, while `ObjectRef` pairs a slot with a zone-change
generation. All current zone transitions share reset, library-index and combat-removal
logic; spell targets capture their incarnation when cast. Damage sources retain their
captured incarnation and attributes. Transition results own the departing object's
last-known view. Fixed [token factories](tokens.md) share immutable storage;
ceased token slots are recycled with new generations and an owned free list.
See [object-identity.md](object-identity.md) for scope and schema.

Zone transitions now dispatch the admitted entry/death triggers; attack declarations
dispatch admitted attack triggers. Simultaneous death sources are captured before
any object moves. Zone-following exceptions and wider event conditions still need
implementation. Before control-changing effects, distinguish owner from controller
everywhere. Before prevention/replacement cards, extend damage/zone event processing.
These are prerequisites for card admission.

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

The user selected **current Standard first**. The
[Standard target](standard-target.md) pins the initial format date and describes
the next acceptance gates. Prioritize #3/#4 around that card pool. The initial
Freeform benchmark is not a Standard acceptance test.
