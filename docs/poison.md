# Player poison counters and state-based loss

Issue: https://github.com/altf4/rustmage/issues/162

Four complete bodies bring executable coverage to **1,246/5,164**, with **3,918
unsupported**. Six regression identities, 56 token factories and one emblem
remain separate. Standard stays pinned to October 15, 2026.

## Complete bodies and provenance

| Card | Whole body |
| --- | --- |
| Fynn, the Fangbearer | Legendary 1/3 with deathtouch; each owned deathtouch creature dealing combat damage to a player triggers two poison counters for that damaged player |
| Persuasive Interrogators | 5/6; entry investigates; sacrificing an owned Clue gives target opponent two poison counters |
| The Serpent Society | Legendary 3/4 with deathtouch; Ward costs getting five poison counters; another owned deathtouch creature dying makes each opponent sacrifice a nontoken creature |
| Virulent Silencer | Artifact 2/3; each owned nontoken artifact creature dealing combat damage to a player triggers two poison counters for that damaged player |

`poison-admissions.json` records all four Oracle bodies and 27 SHA-256 entries
across 21 unique Java files at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. It includes the constructors,
combat-damage and death triggers, poison effects and Ward cost, deathtouch,
investigate and complete Clue recipe, sacrifice effect, targets and the shared
poison-loss check in `GameImpl`. Each pinned Git blob was compared against the
read-only checkout. No token factory was added.

## State, binding and rules

Each player now has a persistent checked `u64` poison count. Both observers see
`players[].poison_counters`; turns and source departures do not reset it. Native
snapshots, clones and restore include the value through the existing owned state.
The shared state-based batch declares a loss at ten or more counters. Concurrent
poison/life/failed-draw losses use the existing simultaneous-loss handling.
Poison does not change life, life-loss history or damage totals.

`AddPoison` applies to an explicit player set. `DamagedPlayer` binds the recipient
from the already-owned damage event, rather than inferring that the recipient is
an opponent. A qualifying hit triggers a fixed two counters regardless of damage
amount. Separate dealers trigger separately; fragments of one simultaneous hit
remain one occurrence. The actual poison effect uses the stack. Positive combat
damage, controller, effective deathtouch and nontoken/artifact qualifications use
frozen damage-source facts. Unknown event-object or recipient bindings remain
unsupported.

The new immutable filtered creature-death descriptor reuses `CreatureFilter` and
frozen departing keywords. Dying observers see the entire pre-event battlefield.
Society excludes itself from its death condition and its sacrifice effect accepts
only the opponent's nontoken creatures. That choice is not targeting and can
sacrifice a hexproof/indestructible creature. Clue sacrifice uses the existing
explicit actor-tagged event; an artifact with another subtype does not qualify.

Poison Ward uses an owned opponent optional-choice frame. Accepting gets exactly
five counters and lets the associated stack item continue; declining uses the
shared counter helper, including its uncounterable protection and exact stack
identity. A player can accept even when doing so causes a poison loss. Source
removal/reentry does not undo Ward or redirect it. An overflow restores the whole
current action and pending frame; earlier completed costs stay paid.

The [official September 25, 2026 rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)
cover poison in CR104.3d/122.1f/704.5c, Ward in CR702.21, simultaneous state-based
checks in CR704, frozen sacrifice/death look-back in CR603.10a and independent
abilities in CR113.7a. [Fynn's official release notes](https://magic.wizards.com/en/news/feature/kaldheim-release-notes-2021-01-22)
confirm that the poison-loss rule does not require Fynn to remain in play. The
[Marvel Super Heroes release notes](https://media.wizards.com/2026/downloads/MSH_Release_Notes_N6hMZoes90/zfGb4BJ8Bf_EN_MTGMSH_ReleaseNotes_20260513.pdf)
likewise identify Society's poison loss as a state-based action.

## Observation and memory boundary

Observation schema is now **50** for the new public player field. Current Rust
and Python schema assertions are updated. C ABI 1 and control protocol 2 remain
unchanged. Snapshots are opaque native owned state, rather than a serialized
schema-49/50 file format.

On the measured build, initialized logical state remains **19,784 bytes**: the
new counters fit within the existing player-state alignment. No trigger-event or
optional-choice payload fields were added. State-byte accounting excludes
capacity, allocators, COW sharing, immutable definitions, observations and
external snapshots. Poison prevention/removal, infect/toxic, counter replacement,
proliferation and multiplayer rules remain outside the admitted bodies.

## Validation and Standard workload

Thirteen native tests cover both seats and all four bodies, actual casts and
complete stats/keywords/Clue entry, real combat, frozen effective deathtouch,
controller/type/token/positive-damage filters, damaged-player binding, fixed
poison amounts, per-dealer multiplicity and source-independent lethal poison.
They check targeted Clue poison above its independent private draw, Society's
frozen dying-observer and nontoken sacrifice effect, owned Ward acceptance and
decline at zero/four/five/nine counters, losing cost acceptance, uncounterable
Eject, source reentry, persistence and simultaneous poison/life losses. Owned
decisions replay through clone/restore and reject stale revisions/illegal
choices. Capacity cases preserve a pending Ward payment or a targeted poison
frame after the Clue cost has already been paid.

Three compiler tests protect exact poison/recipient/Ward clauses, combat and
death filters, complete-body admissions and negative scopes. Three fresh release
Python tests replay Fynn and Silencer combat games through ten-counter losses,
Interrogators above its Clue draw, and Society's decline versus two accepted Ward
payments through a poison loss.

Complete checks passed: **998 Rust tests** (986 unit, 12 integration), **188
compiler/data tests**, **135 fresh release Python tests**, all-target Clippy with
warnings denied, formatting, catalogue consistency and diff checks.

Two native Standard100 runs reproduced the same vectors in the table's order:

- 95,747 decisions; casts `[352,290,361,382]`.
- Poison/death triggers `[104,328,149,204]`; finished triggers
  `[101,315,144,175]`. Society's vector counts its death trigger; Ward is separate.
  Games can end with pending frames.
- Ward declined/paid `[32,22]`; 90 games ended at lethal poison, all with the
  losing player still above zero life. The workload verifies the winner.
- 95,747 checks of public persistent poison and hand privacy; 92,848
  creature-history checks; 29,590 clone/restore replay boundaries.
- 22,180 maximum initialized logical state bytes. These decks have no private
  library-choice instructions; their private draws are checked through hand
  visibility, while optional Ward ownership is checked separately.

The separate 1,000-game native Freeform benchmark completed 571,049 decisions at
roughly 404,433 decisions/s and 708 games/s; clone averaged 130 ns and restore
106 ns. It includes policy and observation construction, excludes neural
inference/tensors/JSON, and is not an XMage parity comparison.
