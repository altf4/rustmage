# Derived-power and distinct-name spell reductions

Issue: [#114](https://github.com/altf4/rustmage/issues/114).
Seven complete Standard bodies become executable. Coverage is 1,084/5,164,
with six regression definitions, 49 token factories and one emblem. Full Standard
and broader native RL/search acceptance remain incomplete: 4,080 identities
still lack complete rules.

| Card | Complete admitted behavior |
| --- | --- |
| All-Fates Scroll | Tap for one mana of any color; seven mana plus tap/sacrifice draws cards equal to distinct controlled land names |
| Fungal Colossus | Generic discount per distinct controlled land name; 5/5 |
| Ghalta the Unstoppable | Discount by greatest controlled creature power; trample; other controlled creatures have trample |
| Ghalta, Primal Hunger | Discount by total controlled creature power; 12/12 trample |
| Hellspur Brute | Affinity for outlaws; 5/4 trample |
| The Lord of the Eagles | Discount by total controlled flying-creature power; flash, flying; 8/8 |
| The Scarlet Witch | Controlled instant/sorcery spells with printed mana value at least four are discounted by its live power; 2/3 |

[derived-reduction-admissions.json](derived-reduction-admissions.json) pins each
complete Oracle body and Java hash at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All seven hashes were verified against
that checkout. Unknown residual text continues to reject the entire card.

Derived power is a dedicated cost amount rather than a characteristic expression.
It reads effective creature stats, with owned battlefield and effective flying
filters. Signed values are summed before applying a minimum of zero: a creature
with power -2 and another with power 5 contribute total power 3, not 5. Greatest
power uses the maximum, or zero with no qualifying creature. This evaluation
remains separate from the earlier-layer characteristic evaluator, avoiding a
stat-definition recursion path. Source-power reductions query the actual source
object. The receiving spell's immutable type and printed mana value determine
eligibility, including the maximum printed contribution of two-brid symbols.
Choosing a cheaper equivalent or reducing its total cannot change its mana value.

Characteristic expressions gain exact distinct-name and filter-union counts.
A union counts each object once, so an outlaw with several of the five qualifying
creature types (or changeling) contributes one. Distinct names use rules names,
including token names; internal factory keys do not make otherwise identical
names distinct. The existing interpreter reuses this expression for All-Fates
Scroll's resolution-time draw. Only matching owned battlefield lands contribute;
duplicate names, graveyard lands and opposing lands cannot inflate either rule.

All amounts feed the shared generation, funding and submitted-action checks.
The total still locks before mana abilities/additional costs and cannot reprice
when a source departs or a counted creature changes. A regression casts Ghalta
using a Treasure whose sacrifice reduces Regal Bunnicorn's power after cost
locking. Generic discounts floor at zero; colored costs and printed values
remain unchanged. Derived totals use the existing checked i128 numeric model,
then cap nonnegative discounts at u32::MAX, which is exact for all representable
generic costs. There is no new mutable state, observation field or ABI field.
Initial logical state remains 19,680 bytes; observation 41, ABI 1 and protocol 2
are unchanged. Distinct-name evaluation uses a temporary sorted/deduplicated
vector, outside retained game state.

Rules follow CR 107.1b, 111.4, 202.3, 601.2 and 702.41 in the pinned
[official Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
Unimplemented attack/sacrifice history, target/hand dependencies, temporary cost
grants, cost increases and printed variable costs remain gated. Ghalta the
Immovable, The Pride of Hull Clade, The Skullspore Nexus, Rowdy Research and
Sunderflock still have unsupported body dependencies.

Validation passes 761 Rust tests (749 unit and twelve integration), 138
compiler/data tests, 98 release Python tests, catalogue consistency, rustfmt and
Clippy with warnings denied. Nine new native tests cover signed totals/maxima,
live counters/flying, ownership, distinct names/token-name identity, outlaw unions,
printed mana-value/type filtering, source power, locked Treasure sacrifice,
two-brid/source-departure replay and every admitted body. Two compiler methods
check exact metadata, source-name/bound validation, layer separation, all seven
complete bodies and retained unsupported text. A public Python method exercises
both source-power and distinct-name payments, resolution and clone/restore.

`cargo run --release --example standard_derived_reductions -- 100` completes
100 legal sixty-card Standard games over four decks: 71,845 decisions, 1,761
staged payments, 14,142 independent public checks, 28,251 clone/replay boundaries
and 20,944 maximum initialized state bytes. All seven cards are cast, all six new
discount rules qualify on paid spells, and All-Fates Scroll's draw ability
resolves. An independent public-observation cost model checks effective powers,
names, types, subtype unions, receiving values and locked totals. The policy
prefers staged new spells and delays attacks until turn eighteen. Body checks
cover all seven entries and the Scroll's draw; native tests additionally verify
keyword grants and every mana color. Exact draw checks skip insufficient
libraries. This is a deterministic supported-deck workload, not complete rules
proof, competitive training acceptance or a Java parity comparison.

Logical byte counts exclude spare capacity, allocator/COW metadata, immutable
catalogue storage, observations and externally retained snapshots. The existing
Freeform smoke benchmark excludes neural inference, tensor encoding and JSON;
its timings are unpinned and do not establish an attributed speed change.
