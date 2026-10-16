# Entry-time power and union predicates

Issue: https://github.com/altf4/rustmage/issues/184

Nine complete bodies bring executable coverage to **1,353/5,164** pinned
Standard identities, with **3,811 unsupported**. Six non-Standard regression
identities, 57 token factories and one emblem remain separate. The target is
pinned to October 15, 2026; the full Standard migration remains incomplete.

| Card | Complete body |
| --- | --- |
| HYDRA Assault Robot | 1R 1/3 artifact Robot Villain; another owned Villain and/or artifact entering deals one damage to a targeted opponent |
| Impact Tremors | 1R enchantment; each owned creature entry deals one damage to each opponent |
| Marketwatch Phantom | 1W 2/2 Spirit Detective; another owned creature entering with power at most two grants the source flying until end of turn |
| Neighborhood Guardian | 1W 2/2 Unicorn; the same entry condition gives a targeted owned creature +1/+1 until end of turn |
| Snarling Gorehound | B 1/1 menace Dog; the same entry condition surveils one privately |
| Valley Mightcaller | G 1/1 trample Frog Warrior; another owned Frog, Rabbit, Raccoon or Squirrel entering adds a source +1/+1 counter |
| Vial Smasher, Gleeful Grenadier | BR legendary 3/2 Goblin Mercenary; another owned outlaw entering deals one damage to a targeted opponent |
| Vicious Clown | 2R 2/3 Human Clown; another owned creature entering with power at most two gives the source +2/+0 until end of turn |
| Warleader's Call | 1RW enchantment; owned creatures get +1/+1 and each owned creature entry deals one damage to each opponent |

`entry-predicate-admissions.json` records all nine complete Oracle bodies and
**89 SHA-256 entries across 37 unique Java files** at revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All nine constructors were read in
full, together with the event dispatchers, relevant filters and effects. The
manifest includes intrinsic keywords, targets, controlled filters, power and
another predicates, union definitions and `MageObject`'s outlaw classification.
Every recorded blob matches both the pinned revision and checkout.

## Semantics and compilation

A small immutable `EntryPredicate` vocabulary covers creature power at most a
fixed bound, any of a fixed subtype list, and a kind-or-subtype union. Its event
excludes the source and requires the entrant's controller to match the observer.
The plain “a creature you control” prefix uses the existing `CreatureEntersYou`
event, including creature sources if applicable.

Power uses the post-event continuous characteristics, including simultaneously
entering static sources and entry counters. This condition is tested only when
the creature enters; a later power change does not cancel an already triggered
ability. See the official [Serra Redeemer release notes](https://magic.wizards.com/en/news/feature/dominaria-united-release-notes-2022-08-26)
for the same entry condition. Actual intervening-if clauses keep their separate
trigger-time and resolution-time guard checks.

Subtype unions have no implicit creature requirement (CR 109.2). The outlaw
union is Assassin, Mercenary, Pirate, Rogue or Warlock, consistent with CR 700.12
and the official [Outlaws of Thunder Junction release notes](https://magic.wizards.com/en/news/feature/outlaws-of-thunder-junction-release-notes).
A permanent matching multiple members still produces one event match. Each
member of a simultaneous batch produces its own event. Torpor Orb, Doorkeeper
Thrull and Karn suppress matching entry events before any of these conditions
can allocate triggers or use a once-per-turn limit.

Compiler parsing is singular and closed: the exact power clause, canonical
subtype lists, the exact outlaw noun and a known subtype “and/or artifact”
clause. Unknown subtypes, relation changes, grouped multiplicities, extra
predicates and unsupported effect tails stay gated. Explicit source aliases
still authenticate existing source effects; event-subject pronouns acquire no
new inferred binding. Good-Fortune Unicorn, Serra Redeemer and Wartime Protestors
remain gated until an owned exact entering-object reference is implemented.
Authority of the Consuls and Terror of the Peaks also retain their remaining
unsupported body gates.

There are no new mutable state fields, trigger packet fields, observation
schema or ABI changes. Schema remains **51**, C ABI **1**, control protocol **2**.
Previously executable definitions and all card/token identities are unchanged.

## Validation and independent workload

Full checks passed: **1,150 Rust tests**, **220 compiler/data tests**, **147
release Python tests**, catalogue consistency, formatting, all-target Clippy
with warnings denied and diff checks.

Twelve native tests exercise full casts for all nine bodies in both seats,
post-batch continuous stats and entry counters, entry-only power tests, negative
power, self/enemy/noncreature exclusion, union uniqueness, token multiplicity,
suppression, source reincarnation, end-of-turn expiry, persistent counters,
target rejection and fizzle, private surveil and atomic capacity rollback.
Public actions verify clone and snapshot/restore equality with stale index rejection.
Three compiler tests exercise exact predicates, wrappers and negative bodies.
Nine release Python scenarios cast each complete body, resolve a matching entry
effect, verify private surveil and replay cloned/restored public observations.

Run the legal 60-card/four-copy workload:

```sh
cargo run --release --example standard_entry_predicates -- 100
```

Three deck families use these bodies plus Swab Goblin, Gigantoad and Torpor Orb,
with 24 basic lands. An independent ledger reconstructs entry matches from
public names, subtypes, object transitions and manually calculated continuous
stats. It checks missing and unexpected triggers by exact incarnation, and
separately witnesses damage, counters, temporary boosts, flying, exact selected
surveil destinations and library counts, and
suppression. Public stat checks use manual bases, a Warleader's Call count, a
Gigantoad land threshold and the independently maintained resolved-effect ledger.
No engine trigger conditions, suppression masks or instruction programs are read.

Two deterministic runs reproduced:

- **43,826 decisions**;
- casts **[59, 94, 145, 80, 157, 136, 66, 163, 69]**, in table order;
- triggers **[5, 237, 84, 46, 126, 17, 11, 142, 61]**;
- verified effects **[5, 233, 83, 45, 125, 17, 9, 137, 61]**;
- **219 suppressed would-be triggers**, **130,736 public stat/ability pairs**;
- **125 private choices**, **39,316 replay boundaries**;
- **20,932 maximum initialized logical bytes**.

The 1,000-game native smoke benchmark measured 571,049 decisions, approximately
366,183 decisions/s and 641 games/s, 130 ns clone and 118 ns restore on this host.
Initial logical state remains **19,784 bytes**, excluding allocator/capacity,
immutable definitions, observations and separately retained snapshots. Timing is
host dependent and includes policy and observation construction; it excludes
model inference, tensor encoding and JSON. It is not an XMage parity or speed comparison.

Catalogue fingerprint:
`0ac521c3fa1ebcc2943528fe61a1004c40ddc01d8843444ed0befa7f2e56315b`.
