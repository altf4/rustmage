# Continuous combat restrictions

Issue #308 implements five complete prototype bodies on actual main 8108a6f:
Pacifism, Mouser Mark III, Tiger-Dillo, Technodrome and The Lion-Turtle.
Prototype coverage is 1,891/5,164 Standard identities, 3,273 gated, schema86.
This candidate is not merge-ready; full Standard/RL support remains incomplete.

Immutable CombatRestriction records describe attack/block bans on the source or
its exact attached recipient. Source conditions use existing read-only StepGuard
expressions: another controlled artifact; greatest other creature power; three
or more Lesson cards in the controller’s graveyard; or current source power.
The latter adds a readonly SourcePowerAtLeast guard. Every query uses the current
controller and continuous characteristics. There is no mutable cache or additional
per-game/per-object field.

Attacker declarations for players and planeswalkers and blocker declarations
reject restrictions at the engine action boundary (CR508.1c/509.1b). The effect of
an Aura is kept separate from abilities granted to its recipient. Entering the
battlefield attacking retains its distinct CR508.4 semantics, and restrictions
gained after a legal declaration do not remove existing combat participants.

Whole raw source clauses and authenticated full/short self names are extracted
only from permanent hosts. Unsupported conditions, duplicate restrictions and
incomplete other instructions stay gated. Unknown parentheticals fail closed
before legacy erasure, while exact intrinsic reminders remain valid. Five full
Java records and explicit direct imports are pinned (82 source records); this
is audit evidence, not an execution whitelist or a transitive port.

Initial validation passes eleven compiler/provenance methods including exact
parent comparison: 5,326 of 5,331 definitions preserved, only five whole-body
admissions, five new combat helper rows and Pacifism’s Aura row. Every older
helper/color/alias/recipe/flag is unchanged. Five paid native groups pass both
seats in 0.11 seconds: Aura payment/removal, source versus other artifacts,
current other power changed by real spells, full Technodrome sacrifice/draw/
counter/untap, Lion-Turtle ETB life/mana and three real Lesson spell resolutions.
The tests use explicitly isolated declaration-phase fixtures for attack/block
checks after paid resolution, with per-action owned raw/index forks after parent
drop, restore, both-player observations and actor/revision denials.

The first native run passes four groups and fails the fifth because its fixture
references Disfigure, absent from the pinned catalogue. The corrected fixture
uses supported Stab; no production runtime fix was needed. Terminal targets are
cleaned immediately while every shared-library path/hash is retained.

Frozen production baseline `29a9341a474972f62cdf4b20821c46d19a4304d5`
passes 1,786 native tests, all-target Clippy and formatting. The immutable optimized
library SHA-256 is `5fbe5008c9ccafdb79bd8c0c6995be2278387cd14cc8c72b59e63bea63bb41a2`,
with catalogue fingerprint
`82ba341fbf950464ed4543ce9a64625c7bb83c63d132e7ce1ce61b9af1929212`.
The later test-only 436c88c checkpoint strengthens planeswalker loyalty fixtures;
its five native groups and ten naturally drawn, fully paid public body/seat
cases pass (28.322 seconds for the public cases).

Fixed-workload diagnostics remain failures in their original scopes. The first
100-game attempt at policy 026317d stops after 45 completed games: Tiger-Dillo/P1
seed31202 discarded every enabling Bear Cub before drawing green mana. Policy
51194d1 retains the last enabling creature/artifact and passes that unchanged seed
in 1,490 frames. Its ten public cases also pass, but its new 100-game attempt
fails on Tiger-Dillo/P0 seed30902: the policy spends Fancy Footwork while the source
is summoning sick, then remains in the positive stage after the buff expires.
The engine correctly rejects attacks without a current enabling creature.

Policy 8765b1a waits until Tiger-Dillo can legally attack before casting the
one-turn enabling buff and returns to the enabling stage if the condition ends.
The unchanged seed30902 passes in 1,342 frames with five paid casts; ten public
body/seat cases pass in 19.449 seconds. The new fixed 100-game run is pending.
All of these policy revisions leave production Rust, card programs, decks, seeds,
schema and optimized library unchanged. Every frozen execution file and library
hash is verified per gate; no workload failure is relabeled as a pass.

Composition on the final current main, fresh integration checks and two
independent complete-batch approvals at the final head remain required.
