# Loyalty counters and planeswalker damage foundation

First part of [issue #93](https://github.com/altf4/rustmage/issues/93), which remains
open for loyalty activation costs/timing, combat declarations and whole walker
bodies. This stage does not establish playable planeswalker cards.

Five complete damage bodies now compile: Broadside Barrage, Dragonback Assault,
Fulminous Forte, Sear and Splatter Technique. Coverage is 986/5,164 pinned Standard
identities (4,178 unsupported), six regression definitions and 43 token factories.
[The admission record](loyalty-damage-admissions.json) lists their printed bodies,
source paths and digests at XMage revision 492369c7844b8d76479f0393ff9eb6b0b0cf30d7.
Every printed ability/mode remains required for admission. Existing planeswalker
identities, including Professor Dellian Fel and The Aetherspark, remain rejected
in submitted decks until their complete rules and the remaining kernel are ready.

`Counters` adds unsigned 64-bit `loyalty`; `CounterKind::ALL`, `get` and exact u128
`total` include it. Doubling, any-kind removal, typed payment generation, source
obligations, virtual mana-funding keys and optimistic resource counts all account
for it. Typed removals have a direct bounded path; mixed removals enumerate all
valid distributions among four represented kinds. Loyalty on a creature counts
as a counter for modified checks but never adds power/toughness. Other counter
types remain gated with their corresponding unsupported bodies.

Printed starting loyalty is generated as a sparse immutable match table, using
the represented front face. Missing, negative, non-string or oversized numeric
facts fail closed even with a refreshed data checksum. `Card::is_planeswalker`
and `starting_loyalty` expose those facts without admitting printed abilities.
Battlefield entry initializes loyalty; a zone change clears it and a later entry
starts from the printed value. Turn changes and cleanup preserve loyalty.

Positive damage to a planeswalker removes up to that many available loyalty
counters. It does not mark creature damage. The event still retains its full
quantity for lifelink/history; large damage is not narrowed to u64. Zero/negative
events do nothing. Deathtouch is applied only to creatures and does not destroy a
walker with remaining loyalty. A zero-loyalty state-based action moves the walker
to its owner's graveyard regardless of indestructible; it is not a creature-death
trigger. Existing snapshots, object generations, simultaneous batches and atomic
capacity rollback own all loyalty state.

The shared kernel now supports the exact creature/planeswalker target and group
sets used by the new spells, and planeswalkers in the represented `any target`
space. No state-based checks run between ordinary instructions: Broadside Barrage
can leave a zero-loyalty future walker on the field during its mandatory discard
choice, then apply the SBA once resolution completes. Existing target revalidation,
fizzle, source-generation and hexproof rules continue to apply. Battle defense,
planeswalker animation/control layers and other unimplemented bodies stay gated.

Observation schema **38** includes loyalty in visible object counters. Zero
loyalty is omitted from JSON for compact common-case observations; a missing key
deserializes to zero. Native `Counters` always carries the field. Positive values,
including u64::MAX, round-trip exactly. ABI 1/control-response schema 2 are unchanged.

Author verification: 609 Rust tests (599 unit plus ten integration), 106 compiler/
data tests, 73 release Python tests, catalogue consistency, rustfmt and Clippy with
warnings denied. Native fixtures cover entry/reset, indestructible versus zero
loyalty, lifelink/deathtouch/history, huge quantities, every new mode/body, all-kind
doubling and overflow rollback, exact any-counter payment and atomic forgery
rejection. Every pinned starting-loyalty fact is checked. Rust and actual C ABI
fixtures explicitly inject unsupported future walker objects solely to exercise
the kernel; they do not execute those walkers' missing printed abilities or admit
them in production decks. The C ABI fixture verifies loyalty observations, real
Sear damage, clone and restore. Public Python games use supported cards to check
Dragonback's actual entry and landfall programs and Broadside's damage/draw/discard
continuation, including snapshots and stale revisions.

One hundred `standard_loyalty_damage` games use supported 60-card Standard decks
and end naturally: 145,800 decisions, 3,746 independent public checks, 35,870
clone/replay boundaries, 21,276 maximum initialized state bytes. These games exercise
the creature branches; future-walker branches are covered by the explicitly
isolated native/C ABI fixtures. Focused tests cover modes not chosen by the smoke
policy. This is not full Standard or planeswalker acceptance.

A local unpinned release `benchmark 1000` run produced 571,049 decisions at 394,427
native decisions/second, clone 111 ns, restore 108 ns, and 19,552 initial logical
state bytes. The preceding same-deck stage recorded 17,616 initial bytes; adding
loyalty increases dense object/continuation storage by 1,936 bytes in that fixture.
This is an explicit memory cost of the direct representation. COW still shares
the arena until a branch mutates it. Short unpinned timing samples do not establish
an attributable speed change; the atomic-checkpoint overhead remains documented
in numeric-capacity.md. These measurements exclude inference, encoding, JSON,
allocator/reference-count metadata, spare capacity and external snapshots, and do
not establish an equivalent-workload Java comparison. The all-Standard/RL goal
remains active.
