# Hybrid activation costs and committed equivalents

Issue: https://github.com/altf4/rustmage/issues/124

Three complete Standard bodies are admitted:

| Card | Complete implemented behavior |
| --- | --- |
| Cindering Cutthroat | {2}{B/R}, 3/2 Lizard Assassin; conditional entry counter if an opponent lost life this turn; {1}{B/R} grants itself menace until end of turn |
| Hovel Hurler | {3}{R/W}{R/W}, 6/7 Giant Warrior; enters with two -1/-1 counters; {R/W}{R/W}, remove a counter: another controlled creature gets +1/+0 and flying until end of turn, only as a sorcery |
| Trostani, Three Whispers | {G}{G/W}{W}, legendary 4/4 Dryad; {1}{G} grants target creature deathtouch, {G/W} grants vigilance, {2}{W} grants double strike, each until end of turn |

Full Oracle bodies and Java SHA-256 hashes are in
[hybrid-activation-admissions.json](hybrid-activation-admissions.json), pinned
to XMage revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Catalogue
fingerprint: `a3e6a4f1ba6a95eb4e0afac839b5f2d5a7cda9ac1fd29b6530714f863533c8fa`.
Coverage is 1,105/5,164 Standard identities, plus six regression definitions,
49 token factories and one emblem; 4,059 identities remain unsupported.

Activation costs now hold immutable alternate mana equivalents, rather than
representing each equivalent as a different ability. One printed ability keeps
its index, once-per-turn/use bit, counter/tap/sacrifice/life requirements, target
plan and source identity across every cost route. Direct prefunded actions
include every exact allocation and deduplicate allocations shared by multiple
equivalents. Announcements are offered if any equivalent is reachable. They
commit an affordable equivalent before targets, mana abilities and payment,
using existing owned announcement state. Selected colored and generic quantities
remain fixed; an unselected or unaffordable equivalent cannot be submitted.
Counter-cost reservations remain pending until payment; timing and target rules
are unchanged. Source-independent mana-only costs can finish after source
departure using the captured definition, while source-dependent costs retain
existing incarnation checks. Rules references: CR107.4, 601.2b/f/h, 602.2 and
605.3 in the pinned [Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).

Counterfactual pure-mana funding enumerates the same immutable cost equivalents,
including a cheaper two-brid side, and deduplicates physical allocations before
applying virtual production. Source reservation, recursion exclusion, counters
and life remain shared obligations; queries do not mutate the game. No new
complete card in this slice has a hybrid pure-mana ability or a two-brid
activation. Explicit future-cost IR tests cover those helper paths and selected
generic-cost locking without admitting a partially implemented card.

The compiler parses fixed, ordinary hybrid and two-brid activated mana symbols.
Unknown/Phyrexian/X symbols, hybrid cycling/ward, generic discard activation
costs and residual mechanics remain gated. For resource-free pure mana
production, the profit check compares output against the cheapest equivalent;
a two-brid generic-side cost cannot conceal an unbounded cheaper conversion.
Infinite positive resource production still requires a separate solver. Ordinary
fixed costs retain their existing behavior. Deduplication sets do not allocate
for ordinary single-equivalent costs.

Observation schema **45** adds the `activation_cost` decision and
`ChooseActivationCost { cost }` action. Existing public `cost_choices` and
`announcing_cost` fields carry the alternatives and committed choice. The acting
controller owns cost choices/payment requests; the opposing viewer does not
receive private choices. Spell-cost and activation-cost actions cannot be
interchanged. ABI 1 and control protocol 2 remain unchanged. No persistent state
field is added; initial logical state remains 19,752 bytes. Immutable alternatives
are shared catalogue data. Initialized logical accounting excludes capacity,
allocator/COW metadata, static definitions, observations and external snapshots.

Validation passes 810 Rust tests (798 unit and 12 integration), 150 compiler/data
and 107 freshly built release Python tests, catalogue consistency, formatting,
Clippy with warnings denied and diff checks. Ten new native tests exercise both
colors, overlapping allocation deduplication, staged cost/target ordering,
unfundable/forged choices, ownership/stale revisions, exact payments, reserved
counters, sorcery restrictions, all five ability bodies, source departure,
shared once/tap restrictions, selected two-brid future IR and counterfactual
mana-cost alternatives. Python exercises five native cost-choice/payment/body
replay cases across all three cards.

`cargo run --release --example standard_hybrid_activations -- 100` runs three
legal 60-card Standard decks through 100 complete games. It checks all three
entries, all five resolved abilities, all seven hybrid payment routes and the
two fixed Trostani routes. Results: 61,512 decisions, 1,063 activation payments,
1,063 body checks, 29,480 clone/restore/replay boundaries and 21,016 maximum
initialized state bytes. Every payment checks the selected request, total and
colored requirements; resolved bodies check effective keywords and Hovel's
+1/+0. Policy limits repeated activations per printed ability per turn to keep
this a finite smoke workload; legal repeat activations remain available.

A fresh 1,000-game Freeform baseline produces 571,049 decisions at 392,415.01
decisions/s and 687.18 games/s; clone 235.60 ns and restore 183.07 ns. This
unpinned sample includes smoke policy and observation construction, excludes
model inference/tensor/JSON work, and establishes neither a performance change
nor XMage parity.

Full Standard and native RL/search acceptance remain incomplete and active.
