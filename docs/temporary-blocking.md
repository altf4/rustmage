# Temporary blocking prohibitions

Issue: https://github.com/altf4/rustmage/issues/154

Six complete bodies bring executable coverage to **1,218/5,164**, with **3,946
unsupported**. Six non-Standard regression identities, 55 token factories and
one emblem remain separate. The target stays pinned to October 15, 2026.

## Complete bodies

| Card | Rules implemented |
| --- | --- |
| Brambleback Brute | 4/5, enters with two -1/-1 counters; sorcery-only {1}{R} and remove a counter to prevent target creature blocking this turn |
| Sandstorm Verge | Desert; tap for colorless; sorcery-only {3}, tap to prevent target creature blocking this turn |
| Sower of Chaos | 4/3; instant-speed {2}{R} to prevent target creature blocking this turn |
| Summit Intimidator | 4/3 reach; entry prevents target creature blocking this turn |
| War Squeak | Enchant creature, +1/+1 and haste; entry prevents target creature an opponent controls blocking this turn |
| Zog, Triceraton Castaway | Legendary 5/4 reach/trample; entry prevents target creature blocking this turn; Mountaincycling {2} |

`temporary-blocking-admissions.json` retains each full pinned Oracle body and
23 SHA-256 entries across seventeen unique Java sources at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. The six card bodies, common
restriction effect, counter/entry costs, sorcery timing, colorless mana,
attachment bonus/haste and Mountaincycling/Cycling sources were checked.

## Compiler and runtime

The compiler recognizes exact single-creature clauses with existing unrestricted,
controller, another-controller or opponent target predicates and the exact
duration “this turn.” It emits the existing temporary `CANT_BLOCK` flag. It adds
no mutable state, instructions or keywords. All target ownership, hexproof,
Ward, exact-incarnation binding, source funding reservations, counter payment,
illegal block rejection and expiry use existing engine authority.

The restriction affects blocking; it does not tap a creature or prevent attacks.
Repeated grants are idempotent and expire together during cleanup. A later
incarnation of the same physical target cannot inherit the grant. Pending old
effects cannot follow the creature into a new incarnation. Brambleback's removed
counter is a committed activation cost. Sandstorm cannot tap itself for mana
while reserving the same tap for its activated restriction. Zog's cycling works
only from hand, discards it as a cost, and reveals the Mountain card found.

Unspecified, next-turn and conditional durations, ambiguous pronouns and plural
target wording remain gated here. Flashback, reflexive payments, redirection and
unsupported residual bodies still reject complete cards such as Duel Tactics,
Shriek, Ty Lee and Untimely Malfunction. No partial bodies were admitted.

The [official rules](https://magic.wizards.com/en/rules), September 25, 2026
text, describe blocking restrictions in CR509.1b and simultaneous temporary
effect expiry in CR514.2. Counter payments, targeted triggers, independent stack
objects and sorcery timing retain their existing shared implementations.

## Validation and native workload

Eight new Rust tests cover both seats, complete normal casts and entries,
actual block declarations and rejected illegal blocks, real cleanup expiry,
Aura attachment and opponent-only trigger selection, all four supported counter
payment kinds and zero-counter rejection, sorcery restrictions versus Sower,
repeated grants, old target references, reserved land funding, hexproof, paid
and declined Ward, and Zog's private typed search/public reveal. Clone/restore
replays and stale action indices are checked at owned decision boundaries.

Two compiler tests protect exact phrases, durations, full six-body admissions
and unknown residual rejection. A fresh-release Python test performs actual
casts/land play, owned target/payment choices, all six effects, observed flags,
cloned/restored resolution and cleanup expiry through the native library.

Complete checks passed: **946 Rust tests** (934 unit, 12 integration), **178
compiler/data tests**, **129 fresh release Python tests**, Clippy on all targets
with warnings denied, formatting, catalogue consistency and diff checks. All
23 Java hashes were independently recomputed from the pinned Git objects.

Two native Standard100 runs reproduced exactly:

- 59,079 decisions; casts/land plays `[137,184,131,178,195,182]`.
- Resolved restrictions `[83,86,185,178,156,182]`, in the table's order.
- 1,844 forbidden block pairs; 283 rejected illegal actions preserving state
  and revision; 762 exact-incarnation cleanup expiry checks.
- 14,841 clone/restore replay boundaries; 21,728 maximum initialized logical
  state bytes.

The complete Standard60 deck uses twenty basic lands, four Sandstorm Verges,
four copies of each other admitted body, and playsets of Bear Cub, Llanowar Elves,
Dragon Fodder and Abrade. The deterministic fixture delays attacks until turn
ten and retains mana sources for owned payment windows to exercise every body.

The native 1,000-game Freeform smoke benchmark completed 571,049 decisions at
404,960 decisions/s (709 games/s); clone 148.61 ns, restore 101.42 ns. It includes
policy and observations, excludes a neural model/tensor encoding/JSON, and is
not an XMage parity benchmark. Timings are measurements, not performance guarantees.

Initial logical state remains **19,784 bytes**. Schema 49, C ABI 1 and control
protocol 2 are unchanged. Snapshot state already owns the temporary flags and
all continuations. Logical byte accounting excludes allocator/capacity overhead,
immutable definitions, external observations and separately retained snapshots.
Catalogue:
`cfc113b9de5c1e736368d9530e5522749ea5910b4fd245ed7a28ed68f75f5b2b`.
