# Conditional and count-scaled entry counters

Issue: https://github.com/altf4/rustmage/issues/118

Three complete Standard bodies are admitted:

| Card | Complete implemented behavior |
| --- | --- |
| Goblin Boarders | {2}{R}, 3/2 Goblin Pirate; enters with one +1/+1 counter if its controller attacked this turn |
| Effortless Master | {2}{U}{R}, 4/3 Orc Monk, vigilance and menace; enters with two +1/+1 counters if its controller cast at least two spells this turn, including this spell |
| Luxknight Breacher | {3}{W}, 2/2 Human Knight; enters with one +1/+1 counter per other controlled creature and/or artifact, each qualifying permanent counted once |

The full Oracle bodies and SHA-256 hashes of the corresponding Java files are
in [entry-counter-admissions.json](entry-counter-admissions.json), pinned to
XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. The catalogue
fingerprint is `4e58bbb421e7b44972fdfb15acc1a08c95817ff38c6a2a8cf7bcdb0f6cedacde`.
Coverage is 1,094/5,164 Standard identities, plus six regression definitions,
49 token factories and one emblem; 4,070 identities remain unsupported.

Immutable entry rules reuse the existing controller attack flag, completed-cast
count, typed guards and battlefield count filters. These replacements run before
the entering object exists on the battlefield, for casts and noncasting entries.
Counters and resulting stats are present immediately; they do not use a trigger
or priority window. Selection/announcement alone does not count as a completed
cast. History belongs to each controller and resets each turn; counters remain
until the object changes zones or another rule changes them.

A simultaneous entry transaction computes all incoming quantities before moving
any member. Luxknight Breachers entering together cannot count each other or
other incoming creatures. The lower-level raw batch helper also freezes counter
quantities before all moves, even though current gameplay uses staged entries.
A later separate entry does count the previous batch.
Artifact creatures and artifact tokens each count once, alongside ordinary
creatures and noncreature artifacts. Opposing permanents and ordinary lands do
not qualify. Existing fixed entry counters and starting loyalty share this path.
The entry transaction's existing attachment/life choices remain owned and
snapshotable; no object enters before those choices finish. Rules references:
CR122.6, 400.7, 601.2i, 603 and 614.12 in the pinned
[Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).

The compiler extracts only the three exact clauses, and emits immutable rules
only after every residual line compiles. Cruel Administrator remains gated by
firebending tokens; death/life-loss/departure histories, cast-origin conditions,
unknown counter kinds and general counter-placement replacements stay gated.
This does not claim general replacement-effect ordering, ability/type/control
changes, copy effects or the rest of Standard.

Validation includes 779 Rust tests (767 unit, 12 integration), 143 compiler/data
tests, 101 fresh-release Python tests, rustfmt, Clippy with warnings denied,
catalogue consistency and diff checks. Eight new native regressions cover actual
casts, owner history, live resolution-time counts, union overlap, both batch
orders, noncasting entries/reentry, fixed counters/loyalty and suspended life
choices. Clone/restore/replay checks include stale action revisions. Public
Python tests cover both raid and completed-cast branches, a positive scaled
entry, immediate stats and exact native replay.

`cargo run --release --example standard_entry_counters -- 100` runs two legal
60-card Standard decks through 100 complete deterministic games. Its public
pre-entry observations independently predict counters and immediate stats. It
checks all six zero/positive entry branches and yields 61,586 decisions, 286
checked entries, 28,781 clone/restore/replay boundaries and 20,564 maximum
initialized state bytes. Alternate policies reserve mana for postcombat casts;
other policies exercise precombat entries. This is a semantics workload, not a
claim of deck strength or broad card-format acceptance.

No persistent state, observation, control protocol or ABI field is added.
Observation 42, control protocol 2 and ABI 1 remain unchanged; initial logical
state remains 19,720 bytes. Entry rules are shared immutable catalogue data.
Batch counter values are temporary during commit. Logical initialized byte
accounting excludes capacity, allocator/COW metadata, static data, observations
and externally held snapshots. The full Standard and native RL/search goal
remains incomplete.

A fresh 1,000-game Freeform baseline produced 571,049 decisions at 402,832.99
decisions/s and 705.43 games/s; clone 122.30 ns and restore 125.26 ns. This
unpinned local run includes the smoke policy and observation construction,
excludes neural inference/tensor encoding/JSON, and establishes neither a speed
change nor XMage parity.
