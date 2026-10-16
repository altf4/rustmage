# Exhaust limits per permanent incarnation

Issue: https://github.com/altf4/rustmage/issues/138
Standard pool: October 15, 2026. This stage admits ten whole bodies, reaching
**1,153/5,164** executable Standard cards; **4,011 remain unsupported**. The catalogue
also has six regression identities, **51 token factories** and one emblem.
The engine remains an in-process Rust library with complete owned snapshots.

## Whole bodies and provenance

`exhaust-admissions.json` records the exact pinned Oracle texts and Java SHA-256
hashes at `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All ten source files, the
shared ExhaustAbility and the three concrete token classes were read.

| Card | Complete body |
| --- | --- |
| Camera Launcher | Exhaust counter plus colorless artifact Thopter with flying |
| Draconautics Engineer | Independent other-creature haste/source counter and flying red Dinosaur Dragon token abilities |
| Greenbelt Guardian | Repeatable targeted trample, separate exhaust three counters |
| Hazard of the Dunes | Reach/trample, exhaust three counters |
| Keen Buccaneer | Vigilance, exhaust draw then private discard then source counter |
| Loot, the Pathfinder | Double strike/vigilance/haste; independent tapped mana, draw-three and damage abilities |
| Pacesetter Paragon | Exhaust source counter then temporary double strike |
| Prowcatcher Specialist | Haste, exhaust two counters |
| Skystreak Engineer | Flying, exhaust two counters |
| Stampeding Scurryfoot | Exhaust source counter plus green Elephant token |

These are thirteen distinct exhaust abilities plus Greenbelt Guardian's ordinary
repeatable ability. Afterburner Expert, Elvish Refueler, Boom Scholar, Greasewrench
Goblin, Hog-Monkey, Mai, Mindspring Merfolk, Rangers' Refueler, Skyserpent Seeker,
Sita Varma and Winter retain their unsupported body dependencies. Exhaust activation
observers, reset exceptions, copies/control changes, X costs, keyword counters,
Vehicles/animation and wider mechanics remain gated. Exhaust combined with a
loyalty cost is also rejected: persistent exhaustion and per-turn loyalty use
would need separate tracking before that composition can be admitted.

## Rules and state layout

Exhaust is a keyword on an activated ability, represented by immutable ability
metadata. An immutable generated mask identifies its slots within the existing
64-bit activation-use field. On turn start, the engine keeps only exhaust bits;
ordinary activation/loyalty/per-turn bits clear as before. A zone identity change
clears all uses. This adds **no field or bytes to mutable GameState/Object**.

Legality checks test both ordinary per-turn and exhaust limits. A completed
activation spends its exhaust bit before the effect resolves. Countering or
untapping cannot refund it, and each slot has an independent limit. Starting a
staged announcement does not itself spend the bit; costs still have to complete.
Exceptions that care about having begun activation need their own history and
remain unsupported, including Elvish Refueler (CR 702.177b).

Loot's mana ability resolves through the existing immediate mana path and is
subject to the same limit. Counterfactual funding checks the spent bits, includes
them in its existing resource keys, and bounds exhaust production to one use.
Neither virtual search nor untapping/reaching another turn can reuse a spent
source. Re-entry creates a fresh incarnation and enables the ability again.

The compiler recognizes only exact exhaust activation lines, compiles every
remaining clause, preserves independent indices, and limits bodies to 64 slots.
Authenticated full/short source names can start a damage clause. A complete
source-counter sentence proves its following keyword pronoun; existing targeted
counter/keyword clauses retain their target binding. Residual text and unknown
pronoun scope remain unsupported. Exact token characteristics and both Engineer
abilities are included; tokens entering after its haste effect do not gain haste.

The [official Aetherdrift release notes](https://magic.wizards.com/en/news/feature/aetherdrift-release-notes)
and CR 702.177a establish the once-only ability limit and re-entry reset. Also see
CR 400.7, 602.2, 602.5 and 605 for object identity, activation restrictions, payment
and mana abilities. No claim is made that reset-exception or exhaust-observer
cards are implemented by this stage.

## Observation and library interface

Observation schema advances **48 → 49**. `ObjectView.exhausted_abilities` exposes
only the persistent exhaust-use bits for that incarnation. The existing
`used_abilities` field contains ordinary current-turn uses plus those persistent
bits. Existing cards' behavior is unchanged. ABI stays 1 and control protocol 2;
card/ability indices remain local to the catalogue fingerprint.

Snapshots own the same use bitset as other state; no external exhaustion registry
or server is involved. Restores preserve every limit and continuation while
advancing revision to reject stale action indices. An old source's delayed effect
cannot place counters on a new incarnation, even though its token creation still
resolves independently.

## Verification

- **869 Rust tests**: 857 unit and 12 integration, including nine new rules tests
  and one funding regression.
- **164 compiler/data tests**, including three new whole-body/limit/loyalty-gate methods.
- **119 fresh release Python tests**. Two new methods exercise all thirteen
  exhaust abilities, payment/body replay, persistence, forged action rejection,
  actual Unsummon/recast reset and private discard through the C ABI.
- Format, all-target Clippy with warnings denied, catalogue consistency and diff
  checks pass.

Native tests cover pre-effect spending, payment-time rejection, all start paths,
turn persistence, independent slots, source generations, exact tokens/counters/
keywords, private continuations, real untaps and bounce/recast, ordinary-repeatable
coexistence, all five single-color mana outcomes and illegal mixed production.
A structural stack-removal fixture models a countered effect without claiming an
ability-counter spell has been admitted. Virtual funding tests isolate feasibility
before use, after later untaps/turns and after re-entry. Rules fixtures and the workload clone/restore/replay decision boundaries with
stale-revision checks.

`cargo run --release --example standard_exhaust -- 100` uses legal 60-card Standard
decks. A separate model tracks committed `(ObjectRef, ability)` uses from actions,
never clearing them at turn boundaries. It compares public masks and every offered
exhaust action, checks resolved source counters, counts all thirteen distinct
exhaust slots and the repeatable slot, and requires all ten entries, three token
bodies, private discard, later-turn persistence and actual reset returns.
External ordinary-activation preferences keep the smoke policy finite without
limiting legal engine actions.

Two native runs reproduce **58,634 decisions**, **245,381 mask/body checks**,
**904 exhaust activations**, **4,401 persisted turn-boundary checks**, **40 private
discards**, **83 reset returns**, **26,959 clone/restore replays**, and **22,324 maximum
logical initialized state bytes**, across 100 completed games.

Initial state remains **19,784 logical bytes**, excluding capacity/allocator/COW
metadata, statics, observations and externally held snapshots. A fresh unpinned
1,000-game Freeform sample yields 571,049 decisions, 397,668.66 decisions/s,
696.38 games/s, clone 166.01 ns and restore 121.51 ns. This includes smoke policy
and observation construction, excludes model inference/tensor encoding/JSON, and
establishes neither a speedup nor XMage parity.

Full Standard and broader native RL/search acceptance remain incomplete and active.
