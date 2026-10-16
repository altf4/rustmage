# Committed creature-death history and morbid rules

Issue: https://github.com/altf4/rustmage/issues/120

Five complete Standard bodies are admitted:

| Card | Complete implemented behavior |
| --- | --- |
| Cackling Prowler | {3}{G}, 4/3 Hyena Rogue, ward {2}; at the beginning of its controller's end step, if a creature died this turn, put a +1/+1 counter on itself |
| Cackling Slasher | {3}{B}, 3/3 Human Assassin, deathtouch; enters with one +1/+1 counter if a creature died this turn |
| Dreaded Bat-Cloud | {4}{B}, 4/2 Bat, flying and deathtouch; costs {3} less if a creature died this turn |
| Needletooth Pack | {3}{G}{G}, 4/5 Dinosaur; at the beginning of its controller's end step, if a creature died this turn, put two +1/+1 counters on target creature it controls |
| Scorpion, Seething Striker | {3}{B}, legendary 3/3 Scorpion Human Villain, deathtouch; at the beginning of its controller's end step, if a creature died this turn, target creature it controls connives |

[death-history-admissions.json](death-history-admissions.json) records complete
Oracle bodies and all five Java SHA-256 hashes at XMage revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`, plus the MorbidCondition and
CreaturesDiedWatcher supporting sources. Catalogue fingerprint:
`f53f514bcff942b217c133db63adadaf8d27d59da2fbe171ba3f19cef1d68618`.
Coverage is 1,099/5,164 Standard identities, plus six regression definitions,
49 token factories and one emblem; 4,065 identities remain unsupported.

The state owns one checked `u64` counter for actual creature transitions from
battlefield to graveyard during the current turn. Either controller's creatures
and tokens qualify. Discard, mill, countering a creature spell, bouncing, exile
and noncreature permanent deaths do not. A dead token ceasing to exist does not
count again; a reentered creature or reused token slot can have a new actual
death event. The history survives later zone changes and is reset by begin-turn,
independently of graveyard contents. It participates in clone/snapshot/restore
and atomic action rollback, including numeric capacity failure.

The common transition boundary records deaths before observers or linked returns
run. Every member of a simultaneous departure batch commits before its zone
observers are queued. Consequently, a Slasher returned by a departing exile
source can see a creature dying in the same batch, in either movement order.
Existing immutable pre-entry rules, locked spell costs and intervening-if guards
share the typed `CreatureDied` predicate. Entry counters are present immediately;
spell discounts retain colored requirements. End-step guards apply at the step's
beginning and at resolution; a first death after the step begins creates no
retroactive trigger. These abilities use the controller's end step and legal
controlled targets. Ward is an independent ability, not a morbid trigger.
Rules references: CR400.7, 601.2f, 603.4, 614.12, 700.4 and 704 in the pinned
[Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).

The compiler recognizes only the exact global predicate; it does not infer
controller-specific or nontoken histories. Whole-body admission still rejects
plot on Blacksnag Buzzard/Rictus Robber, Old Flitterfang's additional activation
cost, Wardens of the Cycle's triggered modes, Tragic Banshee's remaining body
and other missing semantics. General zone replacements, control/type/copy
changes and wider histories remain gated. Existing future syntax that shares
the typed predicate is checked rather than replaced with unconditional effects.

Validation passes 789 Rust tests (777 unit and 12 integration), 145 compiler/data
and 103 freshly rebuilt release Python tests, catalogue consistency, rustfmt,
Clippy with warnings denied and diff checks. Ten new native tests cover false
zone transitions, both controllers, real combat/damage/sacrifice/SBA/destruction,
token slot reuse, reentry/reset, same-batch linked returns, immediate entry
counters, exact staged/illegal payments, guarded step creation and ownership,
private connive continuation and simultaneous-action capacity rollback. Public
Python regressions replay a morbid cast/payment and a private connive discard
without repeating the already completed draw.

`cargo run --release --example standard_death_history -- 100` uses two legal
60-card Standard decks through 100 complete games. The decks intentionally have
no tokens, bounce, exile or graveyard replacements; public before/after object
transitions independently predict actual deaths, including legend-rule deaths.
The workload checks both viewers, both Bat payment and Slasher entry branches,
all five new creature entries and all three guarded step-trigger kinds. It
separates Prowler's ward ability from its morbid ability. Results: 60,021
decisions, 180,063 public history checks, 26,092 clone/restore/replay boundaries
and 20,416 maximum initialized state bytes. Alternate policies reserve mana for
postcombat entries. This is a rules/replay workload, not trained-bot acceptance.

Observation schema **43** exposes `creatures_died_this_turn` as public history to
both viewers. ABI 1 and control protocol 2 are unchanged. The `u64` field fits
existing State padding on the measured platform; initial logical state remains
19,720 bytes. Logical initialized accounting excludes capacity, allocator/COW
metadata, statics, observations and external snapshots. A fresh 1,000-game
Freeform baseline gives 571,049 decisions, 337,705.00 decisions/s and 591.38
games/s; clone 130.59 ns, restore 126.84 ns. The unpinned local run includes the
smoke policy and observation construction, excludes model inference/tensor
encoding/JSON, and establishes neither a speed change nor XMage parity.

Full Standard semantics and native RL/search acceptance remain incomplete. The
full goal remains active.
