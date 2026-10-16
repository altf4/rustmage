# Count-scaled spell reductions and affinity

Issue: [#112](https://github.com/altf4/rustmage/issues/112).
This stage extends the shared locked generic cost path from
[spell-cost-reductions.md](spell-cost-reductions.md). Fourteen complete Standard
bodies bring coverage to 1,077/5,164, with six regression definitions, 49 token
factories and one emblem. Full Standard/native RL/search acceptance remains
incomplete; 4,087 identities are still unsupported.

| Card | Complete admitted behavior |
| --- | --- |
| Chitin Gravestalker | Discount per artifact or creature card in own graveyard, counted once; cycling two |
| Claws Out | Affinity for Cats; owned creatures get +2/+2 this turn |
| Gargantuan Leech | Discount per own battlefield/graveyard Cave; lifelink |
| Huskburster Swarm | Discount per owned creature card in graveyard or exile; menace, deathtouch |
| Memory Guardian | Affinity for artifacts; flying |
| Salt Road Packbeast | Affinity for creatures; entry draw |
| Sami, Wildcat Captain | Double strike, vigilance; grants affinity for artifacts to own spells |
| Scales of Shale | Affinity for Lizards; target gets +2/+0, lifelink and indestructible this turn |
| Thrumming Hivepool | Affinity for Slivers; own Slivers gain double strike and haste; own upkeep creates two colorless 1/1 Slivers |
| Tolarian Terror | Discount per own graveyard instant/sorcery card; ward two |
| Travel the Overworld | Affinity for Towns; draw four |
| Valkyrie Aerial Unit | Affinity for artifacts; flying; entry surveil two |
| Wildvine Pummeler | Vivid counts distinct colors among own permanents; reach, trample |
| Witherbloom, the Balancer | Affinity for creatures; flying, deathtouch; own instant/sorcery spells gain affinity for creatures |

[scaled-reduction-admissions.json](scaled-reduction-admissions.json) pins every
complete Oracle body and Java source hash at revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All fourteen Java hashes were verified
against that checkout. No residual clause is discarded to admit a card.

The immutable reduction amount is now fixed or a typed characteristic quantity
with a fixed multiplier. Existing count evaluators provide ownership, zone,
real-card exclusions, type unions, subtype matching, distinct colors and sums.
Artifact creatures match a union once. Creature-card counts in exile/graveyard
exclude tokens; battlefield affinity includes matching tokens. Changeling matches
creature types, without treating lands as creature types. Cave/Town vocabulary
uses actual land subtypes. Vivid unions color bits, ignoring colorless objects
and duplicate colors. Proposed spell cards remain outside battlefield/graveyard
counts; hand-dependent clauses remain gated.

All quantities are evaluated before additional costs and mana abilities, then the
existing owned total remains locked through payment. Both staged and prefunded
paths use the same reduction evaluator; multiple affinity instances add, and
colored components and printed mana value remain unchanged. Nonnegative amounts
and products saturate at u32::MAX before accumulating: this is exact for every
representable generic cost, whose reduction floors at zero. Cycling and other
activation costs do not use spell discounts. Only owned battlefield sources
provide global grants, using the existing shared source index.

Rules: CR 601.2a, 601.2f–h and 702.41a–b in the pinned
[official Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).
Unknown attack/death/sacrifice history, differently named counts, target-dependent
reductions, temporary grants, variable printed costs, cost increases and remaining
body dependencies stay gated. This includes Rowdy Research, Witchstalker Frenzy,
Hellspur Brute, Eddymurk Crab, Sunderflock, Sapling Nursery and Rottenmouth Viper.
No new mutable state, observation field or ABI field is introduced. Initial logical
state remains 19,680 bytes; observation 41, ABI 1 and protocol 2 are unchanged.

Validation passes 752 Rust tests (740 unit, twelve integration), 136 compiler/data
tests, 97 release Python tests, catalogue consistency, rustfmt and Clippy with
warnings denied. Twelve focused native regressions cover counts, scope, type
unions, changeling, zones/card exclusions, distinct colors, floors, colored
requirements, stacked affinity, locked sacrifice, entry/keyword/token bodies,
cycling and replay. Compiler tests require all fourteen complete bodies, exact
metadata and retained unsupported clauses. Two new public Python tests check
native affinity payment/entry and private surveil choices through clone/restore.

`cargo run --release --example standard_scaled_reductions -- 100` completes
100 legal sixty-card Standard games over four decks: 68,986 decisions, 1,560
staged payments, 10,262 independent public checks, 25,679 cloned/replayed
boundaries and 21,504 maximum initialized state bytes. All fourteen cards are
cast and every new cost rule qualifies on a paid spell. Public cost checks use an
independent model of types, zones, ownership, color unions and locked totals.
The policy prefers staged casts and delays attacks until turn eighteen; private
surveil choices are replayed. Body checks cover creature entry stats, draw,
Claws/Scales effects and Hivepool tokens/grants, allowing queued triggers to
survive source removal. Focused tests separately cover cycling and ward metadata.
Exact draw checks skip insufficient libraries. This workload samples these decks;
it is not a complete rules proof or a competitive training acceptance result.

A fresh release baseline over 1,000 Freeform games reports 571,049 decisions,
391,448.72 decisions/second, 685.49 games/second, 117.55 ns clone and 116.55 ns
restore. It uses the existing limited native smoke policy with observation
construction; it excludes neural inference, tensor encoding and JSON. Timing is
unpinned and is not an attributed speed change or a Java parity comparison.
Logical bytes exclude unused capacities, allocator/COW metadata, immutable
catalogue storage, observations and externally retained snapshots.
