# Continuous combat restrictions

Issue #308 implements five complete bodies on actual main b390849:
Pacifism, Mouser Mark III, Tiger-Dillo, Technodrome and The Lion-Turtle.
Composed coverage is 1,903/5,164 Standard identities, 3,261 gated, schema89.
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
body/seat cases pass in 19.449 seconds. The new fixed, unretried 100-game run passes in 198.991 seconds: 93,895 owned frames and 187,790 actor/revision denials, with all 827 execution inputs and the library hash unchanged.
All of these policy revisions leave production Rust, card programs, decks, seeds,
schema and optimized library unchanged. Every frozen execution file and library
hash is verified per gate; no workload failure is relabeled as a pass.

Final frozen composition `69943a1bb2cdc54bf0dece2dc8610debd8a1aedd` is based
on actual main `b39084973f5e4d1d347f99322080c315df2fc7bc`. It admits exactly the
five complete bodies and preserves all 5,326 other definitions, all 882 previous
helpers, colors, aliases, flags, 94 token recipes and the emblem. Only the five
combat helper rows and Pacifism's Aura row are added. The five complete programs,
read-only combat runtime file and two new extraction/authentication functions
are unchanged from the original production baseline. Every inherited return,
Role, reflexive and temporary-grant rule and raw guard remains present.

Fresh composed validation passes 1,812 unit plus 12 integration tests, all-target
Clippy with denied warnings and formatting (40.520 seconds); all 35 affected
compiler/provenance methods (9.415 seconds); and seven public API methods
(77.453 seconds), without failures/errors/skips. The public methods exercise all
ten combat body/seat cells, four return cells, six temporary-grant cases, local/foreign
Werefox and all six Role semantics bodies plus cheaper-normal-cast timing.
The fixed unretried 100-game Standard workload passes in 198.364 seconds with
ten complete vectors, 93,895 owned frames and 187,790 actor/revision denials.
All 852 frozen execution files and the immutable library hash remain unchanged.

The final optimized library is
`/home/altf4/.cache/rustmage-immutable/combat309-69943a1-995c-schema89-e4f7906b304c.so`,
SHA256 `e4f7906b304ce5d4abecf0feeb2c2a875d14a3f362c51f709029c13213a05d5e`,
schema89, fingerprint
`995c10d2b1e8c94c6d014d4cfb3e083ee681433bb8b246d1833f648539c0fb92`.
Historical parent-preservation tests retain their exact delivered checkpoints
and independently compare their complete programs with the current catalogue.
No failed workload is relabeled passing, and no full broad compiler/API suite is
claimed for this batch. The earlier scope and fixed-seed policy diagnostics above
remain explicit.

Two independent complete-batch preliminary reviews report no findings. Their
novel paid controls include foreign ownership/current control, opponent versus
controlled enabling creatures, real power changes/cleanup, and Aura removal plus
same-physical-card recast. Original and current probe scopes remain distinct.
All gate jobs are terminal and non-library targets cleaned, preserving every
shared-library path/hash. Final exact-head nonauthor confirmations remain required
before merge. Full Standard and training acceptance remain incomplete.
