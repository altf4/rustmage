# Explicit sacrifice events

Issue: https://github.com/altf4/rustmage/issues/158

Seven complete bodies bring executable coverage to **1,235/5,164**, with **3,929
unsupported**. Six regression identities, 56 token factories and one emblem
remain separate. Standard stays pinned to October 15, 2026.

## Complete bodies and provenance

| Card | Whole body |
| --- | --- |
| Biotech Specialist | 1/3; entry creates a Lander; sacrificing an artifact deals two damage to target opponent |
| Crime Novelist | 1/3; sacrificing an artifact puts a +1/+1 counter on itself and adds red mana |
| Experimental Confectioner | 2/3; entry creates a Food; sacrificing a Food creates a 1/1 black Rat that cannot block |
| Gleaming Geardrake | Artifact 1/1 with flying; entry investigates; sacrificing an artifact puts a +1/+1 counter on itself |
| Lightless Evangel | 2/2; sacrificing another creature or artifact puts a +1/+1 counter on itself |
| Pirate Peddlers | 2/2 with deathtouch; sacrificing another permanent puts a +1/+1 counter on itself |
| Tolls of War | Enchantment; entry creates a Clue; sacrificing a permanent during its controller's turn creates a 1/1 white Ally, once each turn |

`sacrifice-event-admissions.json` records all seven Oracle bodies and 47 SHA-256
entries across 30 unique Java sources at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. The manifest includes the common
sacrifice trigger, entry/token effects, counter and mana effects, damage target,
flying/deathtouch, turn condition and complete Lander/Food/Clue/Rat/Ally recipes.
Each hash was computed from the pinned Git object and compared with the
read-only checkout. Existing native token factories implement the corresponding
paid sacrifice/search/draw/life abilities; no factory was added.

## Event boundary and rollback

A transient `ZoneChange.sacrificed_by` records an explicit actor only for actual
sacrifices. Ordinary zone transitions carry no sacrifice cause. Source costs,
mandatory spell/activation additional costs and simultaneous resolution choices
share `sacrifice_batch`. The helper validates battlefield/control assumptions,
freezes all pre-departure object characteristics and observers, moves the whole
batch, marks its cause and dispatches events. A single-object fast path avoids batch
vector allocations for ordinary Food, Clue and Treasure costs. Virtual mana funding never queues
a sacrifice or assumes a future triggered mana payment.

The immutable `Sacrificed` descriptor reuses exact `SacrificeFilter` predicates
and an event-time own-turn flag. The existing departure-source index and trigger
use ledger supply look-back and once-per-turn semantics, including a source
sacrificed alongside another permanent. Ownership means control in the currently
admitted subset; control-changing bodies remain unsupported. A creature-artifact
matches a union once. Token events retain their characteristics after tokens
cease to exist. Source counter effects bind the captured incarnation, while an
independent mana instruction can finish after the source has left.

Crime Novelist always uses the stack. Sacrificing a Treasure during payment adds
the Treasure's mana immediately and queues the Novelist's trigger until casting
finishes. The Novelist's mana cannot fund that same cost. Indestructible stops
destruction but does not stop sacrifice. Hopeful Vigil's source sacrifice stays
a resolution effect; Nutrient Block's source sacrifice stays an activation cost.
Destruction, lethal/zero-toughness state-based departures, bounce, exile, milling
and discard remain separate. Creature-death history retains its existing
creature-only accounting.

The [official September 25, 2026 rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)
cover sacrifice in CR701.21, look-back for sacrifice triggers in CR603.10a,
triggered mana abilities in CR605.1b/605.5a, independent abilities in CR113.7a,
new incarnations in CR400.7 and APNAP ordering in CR603.3b.

No mutable state fields or continuation payloads were added. Snapshot schema 49,
C ABI 1 and control protocol 2 remain unchanged. Initialized logical state stays
19,784 bytes. State-byte figures exclude capacity, allocators, COW sharing,
immutable definitions, observations and external snapshots.

## Compiler boundary

Exact single-event prefixes cover owned artifact/Food/token/permanent sacrifice,
another creature-or-artifact, another permanent and own-turn permanent sacrifice.
The exact counter-plus-red-mana sentence compiles to existing instructions.
The whole residual card body must still compile. Aggregated one-or-more events,
foreign-player events, event-object pronouns/quantities, graveyard watchers,
delayed copies, poison and unsupported residual clauses remain gated. Curious
Cadaver, Esoteric Duplicator, Magnetic Snuffler, Persuasive Interrogators,
Popular Egotist, The Sackville-Bagginses, Camellia and Zhao remain unsupported.

## Validation and native workload

Thirteen native tests cover both seats, complete casts and entry tokens, artifact
and Food filters, multitype uniqueness, wrong actor/origin, indestructible,
self/other predicates, dying observers, APNAP, costs versus effects, mana-ability
payment timing, private draw/search, once-per-turn and new-incarnation behavior.
They replay owned decisions using clone/restore and reject stale revisions and
illegal cost choices. Stack-ID capacity failure restores a sacrificed token,
paid resources and revision; counter overflow restores the pending trigger before
its mana instruction can run. The complete source-index verifier includes the
new descriptor.

Two compiler tests cover exact predicates, the compound mana sentence, whole-body
admissions and negative residual/event bindings. One release Python test exercises
all seven cards through real casts, source or additional sacrifice costs, typed
tokens, targets and cloned/restored native trigger resolution. Nutrient Block can
queue its own independent draw alongside Crime Novelist.

Complete validation: **972 Rust tests** (960 unit, 12 integration), **182
compiler/data tests**, **131 fresh release Python tests**, all-target Clippy with
warnings denied, formatting, catalogue consistency and diff checks.

Two native Standard100 runs reproduced the same vectors in the table's order:

- 101,923 decisions; casts `[105,266,268,141,279,296,190]`.
- Queued sacrifice triggers `[213,317,405,382,824,606,1011]`; finished triggers
  `[212,317,404,382,824,606,1011]`. Games can end with pending frames.
- 99,417 creature-history checks, 200 private choices and 35,507 clone/restore
  replay boundaries; 23,864 maximum initialized logical bytes.
- The workload independently rejects a second Tolls trigger for the same source
  incarnation in the same turn and verifies its controller owns the active turn.

The native 1,000-game Freeform benchmark completed 571,049 decisions at roughly
383,286 decisions/s and 671 games/s; clone averaged 127 ns and restore 113 ns.
The workload includes policy and observation construction, excludes neural
inference/tensor encoding/JSON, and is not an XMage parity comparison.
