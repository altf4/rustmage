# Declared-attacker history and complete raid bodies

Issue: [#116](https://github.com/altf4/rustmage/issues/116).
Seven complete Standard bodies bring coverage to 1,091/5,164, with six regression
definitions, 49 token factories and one emblem. Full Standard/native RL/search
acceptance remains incomplete; 4,073 identities are unsupported.

| Card | Complete admitted behavior |
| --- | --- |
| Chart a Course | Draw two; discard one unless its controller attacked this turn |
| Fire Nation Raider | 4/2; raid entry creates a Clue |
| Gorehorn Raider | 4/4; raid entry deals two to any target |
| Rowdy Research | Discount per creature that attacked this turn; draw three |
| Skyship Buccaneer | 4/3 flying; raid entry draws one |
| Storm Fleet Spy | 2/2; raid entry draws one |
| Witchstalker Frenzy | Discount per creature that attacked this turn; deal five to target creature |

[attack-history-admissions.json](attack-history-admissions.json) pins all seven
complete Oracle bodies, Java hashes and the two supporting Java history sources
at `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Each hash was checked against that
revision. The Java watcher stores object references in a set on attacker
declaration; the dynamic value reads that set's size. Departures cannot erase a
prior declaration, and another declaration by the same object cannot inflate it.

The engine retains a COW vector of distinct ObjectRefs for this turn. Selecting an
attacker leaves history unchanged; finishing declaration records the complete
batch before processing any attack triggers. History survives combat ending,
death/exile and token cessation. Object generations distinguish reentries and
reused token slots; repeated declaration of one incarnation counts once. A new
turn clears history. The two discounts count all declared attackers, including
another player's attackers, and use the same locked generation/funding/validation
path as other reductions. Generic amounts floor at zero; colors stay required.

The existing per-player attacked flag remains separate: raid checks the source
controller, and empty/illegal declarations cannot enable it. The four new entry
triggers retain typed intervening-if checks at creation and resolution. Only the
raid label is removed; the explicit condition and entire residual body must
compile. Chart a Course draws once, then branches around the discard instruction
when raid is true. Without raid, the existing private discard continuation is
required and owned by the controller; snapshot replay never repeats the draw.

Rules: CR 400.7, 508.1, 508.4, 603.4, 601.2 and 702.41 in the pinned
[official Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
Putting a creature onto the battlefield attacking does not declare it. Such
entry mechanics remain unsupported, as do extra combats, prepare, subtype/entry
history and conditional entry-counter replacement rules. Goblin Boarders,
Cruel Administrator, Avengers Assemble!, Full Throttle, Hexhaven Dueling Arena
and Strongbox Raider retain whole-card gates. Native isolated declaration
fixtures test generation/deduplication without admitting extra-combat cards.

Observation schema is now **42**. The new public root field
`creatures_attacked_this_turn` is the same for both viewers, including while a
private choice is pending. Existing hidden-card/action projections are unchanged.
C ABI 1 and control protocol 2 remain unchanged. Historical schema assertions in
existing tests now expect 42. The retained history adds 40 initial logical bytes
on this build, including struct alignment and the COW vector header: initial
state is 19,720 bytes. Each initialized history reference adds 16 bytes. No new
per-object or per-player field is added; clone/restore owns the complete history.
Logical bytes exclude capacity, allocator/COW metadata, immutable definitions,
observations and externally retained snapshots.

Validation passes 770 Rust tests (758 unit and twelve integration), 140
compiler/data tests, 100 release Python tests, consistency, rustfmt and Clippy with
warnings denied. Nine new native tests cover commit timing, empty/illegal attacks,
departure/combat persistence, turn reset, deduplication/reentry/token-slot reuse,
COW branch isolation, both spell controllers, exact/color-preserving staged and
prefunded payments, all entry bodies and private Chart replay. Two compiler tests
check precise history scope, raid guards, branch instructions, complete provenance
and residual-text gates. Two new Python methods cover both discounted spells and
both Chart branches, public counts and private discard rollback. Their policies
reserve mana for actions after combat; passive opposing decks isolate these paths.

`cargo run --release --example standard_attack_history -- 100` completes 100
legal sixty-card Standard games over four decks: 67,476 decisions, 1,837 staged
payments, 349,957 public checks, 27,151 clone/replay boundaries and 20,628 maximum
initialized state bytes. An independent set built from public committed
declarations checks counts and turn resets after every action; per-player raid
flags are also tracked independently. All seven cards, both cost rules on paid
spells, all four raid trigger bodies, both raid cast states and both Chart branches
are required by assertions. Separate public cost/body checks verify payment,
draw, Clue creation and damage. Half the policies reserve resources for raid
entries after combat; attacks start at turn ten. Exact draw checks skip
insufficient libraries. This workload is not a complete rules proof or
competitive training acceptance.

A fresh release baseline over 1,000 Freeform games reports 571,049 decisions,
409,689.42 decisions/second, 717.43 games/second, 103.30 ns clone and 111.81 ns
restore. It includes the existing native smoke policy and observation
construction, excluding neural inference, tensor encoding and JSON. Timings are
unpinned and are not an attributed improvement or a Java parity comparison.
