# Once-per-turn trigger limits and typed entry observers

`TriggerCondition::OncePerTurn` wraps a complete supported event. Matching the
underlying event uses the same source indexes, captured event contexts and
intervening-if checks as unrestricted abilities. The central enqueue operation
claims `(ObjectRef, ability index)` before allocating a trigger. Later occurrences
are suppressed even while the first trigger waits through a resolution, target
announcement, private choice or APNAP ordering. Failed target announcements and
unfulfilled effects do not refund the limit. Different abilities and different
source incarnations have separate limits; all limits reset at the next turn.

The owned `State.triggered_this_turn` ledger retains original references after a
source leaves, including last-known-information sources that die simultaneously.
This avoids both following a returned incarnation and losing a limit during a
simultaneous death batch. Only limited triggers allocate ledger entries. The
small list is searched on limited events; unrestricted events do not scan it.
State-size accounting includes initialized records, excluding spare capacity and
allocator overhead. Schema 31 exposes the public ledger in observations, without
hand identities or private selections. Native clones and snapshots own the entire
ledger, and restores continue to invalidate stale outward action indices.

New entry predicates cover exact owned subtypes (including land and Aura
subtypes), with or without “another,” and owned artifacts/enchantments. Existing
permanent incarnation, controller and legal-target rules still apply. In the
current engine controller equals owner; control-changing cards remain rejected.

Exact once-limited “one or more” artifact/enchantment entries and nontoken deaths
can share individual-event dispatch because only the first occurrence can claim
the limit and their compiled effects do not reference batch members/quantities.
The multi-player discard batch spelling admits only the complete event-independent
investigate effect. Unrestricted aggregate entry/death events and ambiguous
multi-player amount/player references remain rejected.

The implementation follows CR603.2–603.6 and object identity in CR400.7 in the
[September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
The pinned Java FlyingOctobot and ElvishArchivist implementations provide the
reference source vocabulary. Different limit meanings, including “Do this only
once each turn,” remain gated.

Sixteen complete Standard identities are added:

| Card | Complete admitted semantics |
| --- | --- |
| Avatar Enthusiasts | Counter on source when another owned Ally enters |
| Bloombrute | Limited life-gain draw; targeted trample/lifelink activation |
| Donatello, Way with Machines | Flying; owned artifact entry grows source |
| Elvish Archivist | Separate limited artifact-entry counters and enchantment-entry draw |
| Fateful Discovery | Draw for every owned artifact entry |
| Flying Octobot | Flying; limited other-owned-Villain entry growth |
| Homicide Investigator | Limited owned nontoken death batch investigate |
| Hostile Investigator | Target opponent discards on entry; limited multiplayer discard investigate |
| Koth of the Homestead | Landfall life gain and separate Plains-entry targeted counter |
| Mechan Assembler | Limited other-owned-artifact entry Robot token |
| Mm'menon, Uthros Exile | Flying; owned artifact entry targeted counter |
| Obyra, Dreaming Duelist | Flash/flying; another owned Faerie entering drains opponent |
| Rimefur Reindeer | Owned enchantment entry taps a targeted opposing creature |
| Tanglespan Lookout | Owned Aura entry draws |
| Tidus, Blitzball Star | Artifact-entry growth; named attack taps opposing creature |
| Youthful Valkyrie | Flying; another owned Angel entry growth |

Coverage is 816/5,164 Standard identities, six regression definitions and 40 token
factories. No former admissions were removed. Mary Jane Watson remains gated by
hybrid mana; Crossbones by its compound source-pronoun damage, Enduring Innocence
by entry qualification/type-changing return, and many other cards by missing
conditions, events, copies, layers, replacements or additional costs. A supported
shared event never admits only part of a card.

Validation: 451 Rust tests (443 unit and eight integration), 79 compiler tests,
49 rebuilt release Python tests, catalogue consistency, rustfmt and Clippy with
warnings denied. Eighteen focused native fixtures cover separate sources/abilities,
source departure/reentry, simultaneous deaths/tokens, APNAP, multiplayer discards,
private discard continuation, typed entry targets, Aura entry, actual attack
commitment and full replay/accounting. The failed-target-announcement fixture is
explicitly synthetic and adds no printed definition.

One hundred supported Standard games completed naturally: 52,855 decisions,
335 consumed trigger limits, 1,537 turn transitions and 8,126 cloned/replayed
boundaries. Maximum initialized state was 13,232 bytes (excluding allocator
metadata, spare capacity, external snapshots and static definitions). This is
workload evidence for the supported decks, not full Standard acceptance or an
XMage performance comparison. Batched training/model interfaces and information
set search remain incomplete.
