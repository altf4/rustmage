# Live source stat bonuses from count thresholds

Issue: https://github.com/altf4/rustmage/issues/166

Nine complete bodies bring executable coverage to **1,263/5,164**, with **3,901
unsupported**. Six regression identities, 56 token factories and one emblem
remain separate. Standard stays pinned to October 15, 2026.

## Complete bodies and provenance

| Card | Whole body |
| --- | --- |
| Basking Capybara | 1/3; +3/+0 while its controller has at least four permanent cards in their graveyard |
| Billowing Shriekmass | 2/3 flying; entry mills three; +2/+1 while its controller has at least seven cards in their graveyard |
| Brightspear Zealot | 2/4 vigilance; +2/+0 while its controller has cast at least two spells this turn |
| Cloudsculpt Technician | 1/4 flying; +1/+0 while its controller controls an artifact |
| Frilled Cave-Wurm | 2/5; +2/+0 while its controller has at least four permanent cards in their graveyard |
| Gaelicat | 1/3 flying and vigilance; +2/+0 while its controller controls at least two artifacts |
| Gigantoad | 4/4; +2/+2 while its controller controls at least seven lands |
| Mind Drill Assailant | 2/5 with two blue/black hybrid pips; +3/+0 while its controller has at least seven cards in their graveyard; `{2}{U/B}` surveils one |
| Scorpion Sentinel | Artifact 1/4; +3/+0 while its controller controls at least seven lands |

`count-threshold-admissions.json` records all nine Oracle bodies and 68
SHA-256 entries across 27 unique Java sources at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. It includes constructors,
`ConditionalContinuousEffect`, `BoostSourceEffect`, exact Descend/Threshold and
controlled-permanent conditions, the spell-cast watcher, keyword definitions,
mill entry and hybrid private surveil. Each pinned blob was compared against
the read-only checkout. No token factory was added.

## Live rules and rollback

Immutable `StatValue::AtLeast` evaluates an existing earlier-layer quantity and
returns zero or one. The existing source-only scaling modifier multiplies its
fixed bonus by that boolean. More qualifying cards do not multiply the bonus.
`SpellsCastThisTurn` reads the existing owned cast counter. There is no new
mutable field, cache, invalidation ledger, trigger payload or snapshot format.

The controller's battlefield artifacts/lands count current objects, including
artifact-creature objects once and artifact tokens. Graveyard thresholds count
cards, exclude tokens, and distinguish all cards from permanent cards. Instant
and sorcery cards count for Threshold but do not satisfy Descend. Basic and
nonbasic lands count alike. Other players' objects and other zones do not count.
A count change immediately adds/removes the bonus. An ability activation is not
a spell cast; an actually cast spell counts even if countered. Earlier casts can
precede the source's entry, and the existing next-turn reset clears both players'
cast histories.

These are source-only battlefield bonuses in layer 7c. They stack with existing
counters, temporary growth and commuting anthems. They do not change printed
base characteristics in hand, library, stack, graveyard or exile. Losing a
toughness bonus can cause the existing damage-based state action to put a
creature into the graveyard. Each source uses the existing live static-source
index, already owned by clone/restore. Observations and legal-action generation
remain pure.

[Comprehensive rules CR611.3a–c and CR613.4c](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)
cover continuous static application and the modifying power/toughness sublayer.
These predicates inspect counts and existing turn history, with no recursive
derived-power or effective-keyword dependencies. The compiler accepts exact
leading/trailing "as long as" source-stat clauses and recognized ability-word
labels. Unknown conditional keywords/type changes, activation/cost permissions,
replacement effects, combat/entry/crime histories and mixed unimplemented bodies
remain gated. A recognized bonus never admits an unknown residual body.

Observation schema **51**, C ABI **1** and control protocol **2** are unchanged.
State-byte accounting excludes capacity, allocators, COW sharing, immutable
definitions, observations and external snapshots.

## Validation and full Standard games

Thirteen native tests cover both seats and all nine actual casts, exact stats,
flying/vigilance, mill entry, hybrid cost and private surveil. They check live
crossings in both directions, controller/zone/type/token distinctions, artifact
multitypes, nonbasic lands, non-scaling above a threshold, a shared state-based
death after toughness drops, committed/countered and earlier spells, actual
next-turn history reset, stacking, source departures and observation purity.
Real combat uses current threshold power. Owned decisions replay through native
clone/restore, reject stale/illegal choices and preserve a failed cast with a
maximal spell-history counter.

Three compiler tests cover exact clauses, fixed signed bonus/threshold bounds,
positive/negative scopes, duplicate clauses and whole-body residual rejection.
One fresh release Python method runs nine real-game scenarios to every enabled
bonus, independently calculates the public expected stats, checks private
surveil ownership and replays source/cost/private boundaries including threshold
crossings. The existing source-index inventory test now covers all 19 scaling
sources.

The native `standard_count_thresholds` workload uses legal 60-card Standard
decks, four copies each of nine new bodies and six land identities. No external
state mutation or Freeform configuration is used. Both 100-game runs reproduce:

- 56,600 decisions; casts `[134,25,73,103,83,117,103,132,147]` in table order.
- 25 mill-entry triggers and 25 finished triggers; 394 Mind Drill activations.
- Enabled-bonus observations `[2366,1394,329,8534,1979,1829,5098,2161,5891]`.
- 221,018 independently checked public stat observations; 54,815
  creature-history checks; 394 private surveil choices; 10,679 native replay
  boundaries with stale-revision rejection.
- 20,872 maximum initialized logical state bytes in this workload.

Complete checks passed: **1,027 Rust tests** (1,015 unit, 12 integration),
**194 compiler/data tests**, **138 fresh release Python tests**, all-target
Clippy with warnings denied, formatting, catalogue consistency and diff checks.

The separate 1,000-game native Freeform benchmark completed 571,049 decisions at
roughly 400,107 decisions/s and 701 games/s, clone 439 ns and restore 413 ns in
this run. Initial logical state remains **19,784 bytes**. Timing is
host-dependent; the benchmark includes policy and observation construction,
excludes neural inference/tensors/JSON, and is not an XMage parity comparison.
