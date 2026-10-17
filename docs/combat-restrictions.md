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

Naturally drawn Standard public acceptance, fresh final native/Clippy and
independent complete-batch reviews remain required before delivery.
