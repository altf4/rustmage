# Resolution-time damage and life quantities

[Issue #100](https://github.com/altf4/rustmage/issues/100) reuses immutable amount
expressions for targeted damage and controller life gain. Coverage is now
1,006/5,164 pinned Standard identities, with 4,158 unsupported, six regression
definitions, 48 token factories and one emblem factory. The
[admission record](scaled-effect-admissions.json) contains the eight complete
Oracle bodies and hashes checked against the pinned Java revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

| Complete admission | Behavior |
| --- | --- |
| Archway Angel | Flying; entry gains twice the live controlled Gate count as one gain event |
| Cat-Gator | Lifelink; entry deals live controlled Swamp-count damage to any target |
| Coordinated Maneuver | Choose creature-count damage to a creature/planeswalker or destroy an enchantment |
| Jet, Freedom Fighter | Hybrid-cost creature; entry deals live controlled creature-count damage to an opposing creature, death puts counters on up to two distinct creatures |
| Rumbling Rockslide | Controlled-land-count damage to a creature |
| Slash of Light | Sum of controlled creatures and controlled Equipment, then damage to a creature |
| Smaug the Magnificent | Flying, haste; attack deals controlled-Treasure-count damage to any target, upkeep creates a real Treasure |
| Volley Veteran | Entry deals controlled-Goblin-count damage to an opposing creature, including itself and applicable changelings/tokens/Kindred permanents |

The compiler accepts exact complete count/life sentences and their two damage
word orders. A defined X effect amount is independent of X in a mana cost; variable
mana costs remain gated. Supported sums independently count each term (an object
matching both belongs to both), and a checked immutable multiplier represents
life per permanent. Canonical plurals for land types, Gates, Deserts and Treasures
map to printed subtypes. An opening it/he/she is normalized only for a proven
self-event; exact legendary shortened-name references also normalize to the
source. Unknown subjects, event history, arbitrary power/toughness quantities,
excess damage, player-wide variable damage, replacements, residual clauses and
other incomplete dependencies remain rejected as whole cards.

Amounts evaluate once when the instruction executes, after responses, rather
than when casting/announcing. This follows CR 608.2h. The existing target plans
still validate legal choices and recheck generation/zone/hexproof at resolution
(CR 608.2b). Damage flows through the ordinary source-aware batch pipeline,
including actual damage histories, loyalty removal, lifelink, death and life-gain
triggers (CR 120 and 702.15). A departed source uses captured source facts and
keywords while its count expression still examines the live board. A zero count
causes no damage or life-gain event. Relevant primary rules are the
[official Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt),
with [Wizards' rules index](https://magic.wizards.com/en/rules) as the entry point.

The existing amount evaluator returns u128. Checked multiplication and conversion
to the signed i128 damage/life representation report recoverable capacity errors;
there is no narrowing to u16/i32, truncation or invented game result. The enclosing
transaction restores all earlier damage, life totals, histories, pending choices,
RNG and revision on numeric failure. No mutable amount cache or continuation is
introduced. Existing snapshots own all inputs, and immutable programs remain
shared. Observation schema 41, ABI 1 and control protocol 2 are unchanged.

Author validation passes 662 Rust tests (650 unit and twelve integration), 118
compiler/data tests, 81 release Python tests, catalogue consistency, rustfmt and
Clippy with warnings denied. Fourteen new native tests cover live counts after
responses, ownership/nonbasic/subtype/changeling/token membership, empty sets,
source departure and lifelink, stale target generations, illegal friendly/mode
targets, both modes, Jet's death counters, Smaug's real upkeep Treasure, distinct
life totals/events and full action/choice clone/restore. Synthetic IR fixtures
check unrepresentable damage/gain and u128 multiplication at a suspended optional
boundary without admitting fictional cards. Real Archway and Cat-Gator fixtures
prove atomic rollback of life-gain/lifelink failures. Compiler tests cover complete
admissions and explicit unsupported/ambiguous clauses. Two new public Python/C ABI
tests independently inspect Cat-Gator's damage/lifelink and Archway's gain/event
totals through native clone and reusable snapshots.

One hundred legal 60-card Standard games complete naturally: 51,176 decisions,
753 independent public checks across 251 scaled effects and 12,834 clone/replay
boundaries, with 25,064 maximum initialized state bytes. All eight admissions'
scaled effects occur. Focused fixtures exercise their additional abilities/modes.
The example delays attacks until turn 16, declines blocks and bounds optional
Krenko/Clachan growth/repeats to give the expensive bodies time to execute. These
are documented smoke policy choices; the engine's legal action space is unchanged.
Checks are derived from public pre-resolution counts and actual post-resolution
life, damage, loyalty, token disappearance and histories, independently of the
internal amount evaluator. This is a supported-deck workload, not full Standard
or neural search acceptance.

An unpinned 1,000-game native benchmark measured 571,049 decisions at about
390,888 decisions/s (684.5 games/s), clone 274.3 ns and restore 255.8 ns. The initial
logical state payload remains 19,648 bytes, excluding spare capacity, allocation/
COW metadata, statics, observations and external snapshots. Host scheduling and
other work can affect these timings; they do not establish a performance change.
The benchmark includes smoke policy/observation work but excludes model inference,
tensor encoding and JSON and makes no XMage throughput-equivalence claim.

Full Standard and broader native RL/search acceptance remain incomplete.
