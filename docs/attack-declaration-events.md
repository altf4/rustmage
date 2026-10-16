# Aggregate attack-declaration events

Issue: https://github.com/altf4/rustmage/issues/192

Coverage is **1,372/5,164** pinned Standard cards, with **3,792 unsupported**.
Six non-Standard regression identities, **58 token factories** and one emblem remain
separate. The complete Standard migration remains in progress, pinned to October 15, 2026.

This stage implements fourteen complete bodies: Ancestor Dragon, Armasaur Guide,
Battlesong Berserker, Boggart Prankster, Jolene, Plundering Pugilist, Lionheart Glimmer,
Malamet Brawler, Marshal of the Lost, Meddling Youths, Most Valuable Slayer,
Nori, Teller of Tales, Razorkin Hordecaller, Seasoned Consultant and The Chief Warg.
Nori retains both `{1}{R/W}` payment alternatives; Jolene retains its complete
`{1}{R}`, sacrifice a Treasure activation; all intrinsic keywords, legendary status,
types, costs and other rules remain part of admission. Razorkin adds the exact 1/1 red
Gremlin factory. Clue and Treasure factories retain their complete activated abilities.

## Event timing and rules

`AttackDeclared` is an immutable condition with a minimum qualifying-attacker count,
optional minimum attacking power and a public declaration-time guard. The library
indexes battlefield observers, then dispatches exactly one event per completed,
nonempty declaration. An observer does not have to attack. Tapping, vigilance and
all selected attackers are committed before either individual or aggregate attack
observers inspect the event. Merely selecting an attacker does not trigger anything.
Attacks directed at planeswalkers count too; empty or illegal declarations do not.

The three-creature threshold and Jolene's power qualification are checked when the
attack is declared. Chief Warg's “while” clause similarly checks at declaration,
including nonattacking creatures. These qualifications are not intervening-if clauses
and are not checked again at resolution. Ordinary source-attack triggers remain
restricted to the attacking source.

Ancestor Dragon and Marshal instead count **current attacking creatures at resolution**;
creatures leaving combat reduce that amount. Their quantity is not frozen at declaration.
This matches the pinned XMage AncestorDragonEffect and PermanentsOnBattlefieldCount.
Shared target predicates enforce attacking status, Goblin subtype and control as
applicable at both announcement and resolution. Targets retain exact incarnations;
leaving and returning does not preserve a target. A required target that becomes
illegal causes the existing resolution rules to discard the effect. Counters persist;
temporary growth and keywords expire in cleanup. Captured abilities survive source departure.

## Library and rollback

One snapshot-owned battlefield-source vector avoids scanning the whole object arena
for every declaration. Its entries are removed on departure and restored by existing
clone/restore ownership. There is no new observation or stack-context field. Observation
schema stays **52**, C ABI **1**, and control protocol **2**. Initial logical state is
**19,880 bytes**, up 32 bytes including alignment. Logical accounting excludes capacity,
allocator overhead, immutable definitions, observations and separately retained snapshots.

`StatValue::AttackingCreatures` reads the current combat membership, shared by scaled
life gain and growth. The compiler recognizes only complete, closed event/target/effect
shapes. Unknown qualifiers, tails, pronouns, modes and residual rules remain gated.
There is no per-card runtime name dispatch.

The manifest `attack-declaration-admissions.json` records each full Oracle body and **87 SHA-256 entries across 58 unique Java files** for
all pinned Java constructors/direct supporting rules and token logic, at revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Recorded blobs match both the pinned
revision and checkout. All 5,183 ordinary card IDs remain unchanged. After replacing
numeric token references with their immutable factory identities, exactly these 14
ordinary definitions change; previous definitions and recipes retain their semantics.
Adding Gremlin changes sorted token factory indices and the catalogue fingerprint;
factory indices are scoped to that fingerprint. Replay compares games using the same
catalogue, and no cross-catalogue snapshot compatibility is claimed.

## Acceptance

Eleven new native tests cover both players, all complete casts, both Nori costs,
real Jolene sacrifice/damage, nonattacking observers, declaration count/power boundaries,
source departure, live quantities, target rejection/rechecking, planeswalker attacks,
index removal/reentry, snapshot isolation, stale indices and atomic capacity failures.
The Python test runs all 14 bodies in 15 complete-cast/effect scenarios with clone/restore
and stale-revision rejection through the native C ABI. Compiler tests reject unsupported
prefixes, residuals and target qualifiers and verify all 14 whole-body admissions.

`examples/standard_attack_declarations.rs` uses legal 60-card, four-copy-limit Standard
decks. A public-observation ledger independently predicts each declaration trigger
and its exact source incarnation, then checks resolution quantities, tokens, counters,
temporary statistics and keywords. It checks both players' public views. Some seeds
hold observers back to witness triggers from nonattacking sources; Nori uses both
hybrid alternatives. Every relevant action is replayed on a clone and after restoration,
with rejection of the stale indexed action.

Two 100-game release runs reproduced 75,548 decisions, casts
`[63,85,81,97,88,79,102,72,76,94,130,94,97,85]`, triggers
`[239,66,341,347,33,328,128,253,75,396,210,349,79,103]`, verified effects
`[239,66,341,162,33,328,128,253,75,396,210,349,79,102]`, Nori cost choices
`[74,56]`, 1,022 declarations, 2,134 nonattacking-observer witnesses, 14 triggers with
no legal targets, 665,113 public stat/keyword/counter checks, 67,161 replay boundaries
and 25,428 maximum initialized logical bytes. Counts follow the card order above.
Differences between trigger/effect counts include illegal-target removal and terminal games.
Catalogue: `4584b51cb98cfbf397b205df5aa8f0306fa20604a8b73ea905483fbbf90a3baf`.

The 1,000-game smoke benchmark measured 571,049 decisions, about 333,115 decisions/s,
583 games/s, 137 ns clone and 135 ns restore on this host. Measurements include smoke
policy/observations, exclude model inference, tensor encoding and JSON, vary with
concurrent host work, and are not an XMage parity comparison.

Validation: all **1,188 Rust tests**, **228 compiler/data tests**, **150 release Python tests**, formatting,
all-target Clippy with warnings denied, catalogue consistency and diff checks pass.
