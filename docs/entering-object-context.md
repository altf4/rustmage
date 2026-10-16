# Owned entering-object references

Issue: https://github.com/altf4/rustmage/issues/186

Three complete bodies bring executable coverage to **1,356/5,164** pinned
Standard identities, with **3,808 unsupported**. Six non-Standard regression
identities, 57 token factories and one emblem remain separate. The target is
pinned to October 15, 2026; the full Standard migration remains incomplete.

| Card | Complete body |
| --- | --- |
| Good-Fortune Unicorn | 1GW 2/2 Unicorn; another owned creature entering puts one +1/+1 counter on that permanent |
| Serra Redeemer | 3WW 2/4 flying Angel Soldier; another owned creature entering with power at most two receives two +1/+1 counters |
| Wartime Protestors | 3R 4/4 haste Human Rebel Ally; another owned Ally entering receives a persistent +1/+1 counter and haste until end of turn |

`entering-object-admissions.json` records the three complete Oracle bodies and
**34 SHA-256 entries across 19 unique Java files** at revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All constructors were read in full,
including event dispatchers, fixed zone-bound pointers, counter and haste
implementations, power/another/controlled filters and intrinsic keywords.
Recorded blobs match both the pinned revision and checkout.

## Owned context and rules

`SpellView.entering_object: Option<ObjectRef>` owns the exact incarnation that
caused an entry trigger. It is independent of the observer's source reference and
of the target vector. Every queued entry trigger captures it, including existing
self-entry triggers; ordinary spells, activations and non-entry triggers use
`None`. The field moves with its packet through APNAP ordering, announcement,
stack and private resolving continuations and is cloned/restored with game state.
It never borrows an arena object or points into a derived cache.

`Subject::Entering` resolves only while that exact incarnation remains on the
battlefield. Source departure does not cancel the effect. Entrant departure,
reentry or token slot reuse makes the old reference inapplicable; resolution
continues without redirecting the effect. These instructions use no targets,
request no target selection and do not globally fizzle an otherwise target-free
ability. Other admitted instructions continue normally.

Serra's power condition uses post-entry continuous characteristics and counters
at the event. Two Redeemers can both trigger for the same two-power entrant and
each later place two counters, regardless of the resulting power. Wartime's
counter persists after haste expires. Suppression applies before capturing any
new trigger; an already captured trigger remains valid when a suppressor enters.
See the [official Serra Redeemer release notes](https://magic.wizards.com/en/news/feature/dominaria-united-release-notes-2022-08-26)
for entry-time power checks, and CR 400.7 for distinct zone incarnations.

The compiler binds only the complete explicit clause “put ... +1/+1 counters on
that creature,” optionally followed by the exact haste clause. It is available
only under an entry-only event condition. Mixed entry/death or entry/attack
headers stay gated for this binding. Unknown pronouns, targets, counter kinds,
keywords and trailing effects are rejected. Static scaling modifiers cannot use
the event subject. Previously executable definitions and card/token IDs are unchanged.

## Observation and memory

Observation schema is now **52**, exported as the Rust constant
`rustmage::OBSERVATION_SCHEMA`. Current native examples and tests use the constant;
Python assertions follow the new schema. C ABI **1** and control protocol **2**
are unchanged. The JSON `entering_object` field is a reference or `null`.
Older milestone documentation retains its original schema and measurements.

The optional reference occupies **24 bytes inline**, without a separate allocation.
On this build `SpellView` occupies **256 bytes**. Alignment in the state's embedded
announcement/resolution slots adds **64 initialized bytes** to a new 120-card game:
**19,848 bytes**, previously 19,784. Stack elements and continuations account for
their actual struct sizes through the existing logical-byte calculation. Counts
exclude allocator overhead, capacity, immutable definitions, observations and
separately retained snapshots. Checkpoint identity and revision semantics are unchanged.

## Validation and Standard workload

Full checks passed: **1,165 Rust tests**, **222 compiler/data tests**, **148
fresh release Python tests**, catalogue consistency, formatting, all-target
Clippy with warnings denied and diff checks.

Fifteen native tests cover complete casts in both seats, exact references and
empty target plans, source independence, entrant departure/reentry, token reuse,
simultaneous sources and entrants, multiple Redeemers, static power conditions,
controller/subtype restrictions, suppression, haste expiry, captured references
waiting through private surveil, non-entry packets, target-action rejection and
atomic counter overflow without haste leakage and normal +1/+1 versus -1/-1
counter annihilation after captured placement. Public actions check clone and
restore equality and stale action-index rejection. Two compiler tests cover exact
bindings, mixed-event and residual negatives and complete bodies. Three fresh
release Python scenarios verify captured references, empty targets, resolved
counters/haste and cloned/restored observations.

Run the legal 60-card/four-copy native workload:

```sh
cargo run --release --example standard_entering_objects -- 100
```

Three families mix these bodies with First-Time Flyer, Merchant of Many Hats,
Swab Goblin, Bear Cub, Snarling Gorehound, Invasion Reinforcements, Helpful Hunter,
Torpor Orb or Warleader's Call, with 24 basic lands. An independent public ledger
keys each expected trigger by **observer incarnation, entrant incarnation and
ability index**, then checks the packet's exact reference and empty targets.
Separate effect accounting checks persistent counters, temporary haste,
damage, drawing, token creation, exact private surveil destinations
and stat/keyword values from manual bases plus a Warleader's Call count. This
witness reads no engine trigger conditions or instruction programs.

Two deterministic runs reproduced:

- **44,320 decisions**, casts **[62, 79, 116]** in table order;
- triggers and independently verified effects both **[27, 22, 30]**;
- **309 exact entry contexts**, **316 suppressed would-be triggers**;
- **140,198 public stat/ability pairs**, **138 private choices**;
- **23,686 replay boundaries**, **21,112 maximum initialized logical bytes**.

The 1,000-game native smoke benchmark measured 571,049 decisions, approximately
383,000 decisions/s and 671 games/s, 181 ns clone and 100 ns restore on this host.
Timing is host dependent and includes policy and observation construction; it
excludes model inference, tensor encoding and JSON. It is not an XMage parity
or speed comparison.

Catalogue fingerprint:
`deda95097088fd61e88d9fe7720b7da18d54fdcb5f0e0c47444e6eaf81eda9f6`.
