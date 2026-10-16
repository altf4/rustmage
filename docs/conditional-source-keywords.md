# Conditional source keywords and shared funding predicates

Issue: https://github.com/altf4/rustmage/issues/168

Nine complete bodies bring executable coverage to **1,272/5,164**, with **3,892
unsupported**. Six regression identities, 56 token factories and one emblem
remain separate. Standard stays pinned to October 15, 2026.

## Complete bodies and provenance

| Card | Whole body |
| --- | --- |
| Barrow Naughty | 1/3 flying; lifelink while another owned Faerie is present; `{2}{B}` gives itself +1/+0 until end of turn |
| Didact Echo | 3/2; entry draws one; flying while its controller has four or more permanent cards in their graveyard |
| Dreadwing Scavenger | 2/2 flying; entry and attack each draw then discard; +1/+1 and deathtouch while its controller has seven or more graveyard cards |
| Echo of Dusk | 2/2; +1/+1 and lifelink while its controller has four or more permanent cards in their graveyard |
| Goblin Tomb Raider | 1/2; +1/+0 and haste while its controller controls an artifact |
| Gravblade Heavy | 3/4; +1/+0 and deathtouch while its controller controls an artifact |
| Kargan Dragonrider | 2/2; flying while its controller controls a Dragon |
| Spineseeker Centipede | 2/1; entry searches for a basic land, reveals it into hand and shuffles; +1/+2 and vigilance with four or more graveyard card types |
| Wildfire Wickerfolk | Artifact 3/2 with haste; +1/+1 and trample with four or more graveyard card types |

`conditional-source-keyword-admissions.json` records all nine Oracle bodies and
91 SHA-256 entries across 38 unique Java files at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. It includes constructors,
conditional continuous/stat/keyword effects, precise count conditions,
controller/other predicates, keyword definitions, draw-discard, basic-land
search and temporary pump costs/effects. Each pinned Git blob was compared
against the read-only checkout. No token factory was added.

## Shared predicates and layers

A positive source keyword grant can now carry an immutable `StatValue`
condition. The existing boolean count-threshold evaluator returns zero or one;
the same predicate controls a compound source stat bonus and its keyword grant.
Every query uses current state. Source departure, losing a qualifying object,
losing a graveyard card/type and restored snapshots immediately change the
result. There is no new mutable game-state field, cache or observation payload.

These are battlefield-only source grants in layer 6, alongside commuting stat
bonuses in layer 7c. Conditions inspect earlier-layer card types, printed
subtypes/changeling, controller, zone and card/token identity. They do not depend
on derived power or effective keywords. Recipients are not selected by effective
keywords. No type-changing or general layer-dependency solver is implied.
[Comprehensive rules CR611.3 and CR613](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)
cover live continuous application and these layers.

Another Faerie excludes the source and accepts a different owned changeling.
Dragon conditions use the same intrinsic subtype facts. Artifact tokens and
artifact creatures qualify as battlefield artifacts. Graveyard conditions count
cards, exclude tokens and use the correct controller; Descend counts permanent
cards, Threshold counts all cards, and Delirium counts distinct card types.
An artifact creature supplies two different card types to Delirium, regardless
of how many such cards exist. Conditional haste changes actual same-turn attack
legality; flying changes blocking; vigilance affects tapping on final attack
declaration; lifelink/deathtouch/trample use the effective damage-source facts.
Printed intrinsic abilities remain when a conditional bonus disappears.

Only exact supported count clauses and positive source keyword lists compile.
Combined stat/keyword clauses retain the keyword clause as a residual that must
compile before the whole body is admitted. Count conditions retain any recognized
life-gain and own-turn requirements as a conjunction; unknown qualifiers remain
rejected. Conditional flash remains gated because it needs permission outside
the battlefield. Conditional changeling,
unknown combat/entry/crime histories, ability removal, recipient dependencies,
new type changes and unknown residual effects also remain gated.

## Counterfactual mana funding

Actual state and funding search now call one earlier-layer evaluator. Concrete
queries iterate actual objects. Funding borrows the immutable original game and
overlays the selectively copied resource objects it has changed. Unchanged
witnesses remain visible without cloning the full game. An actual virtual
sacrifice removes a battlefield artifact and adds its nontoken card to the
appropriate graveyard; token sacrifices do not create a qualifying graveyard
card/type. Virtual actor life overrides the original life when requested.
Existing turn histories remain read-only during pure mana reachability.

Both real and virtual keyword queries use the grant condition. Tests compare a
virtual Omni-Cheese Pizza sacrifice with the actual native mana action: Tomb
Raider loses artifact-based haste while Centipede gains Delirium vigilance.
A Treasure version removes the artifact witness without adding a card type.
The overlay is transient query state; the simulation's owned state and RNG
remain untouched. Existing funding keys/resources and native legality remain
in use. Trigger effects waiting on the stack cannot fund a payment.

Observation schema **51**, C ABI **1** and control protocol **2** are unchanged.
The live static-source index and opaque native snapshots already own all
relevant state. Logical byte accounting excludes capacity, allocators, COW
sharing, immutable definitions, observations and external snapshots.

## Validation and Standard workload

Thirteen native body/rules tests cover both seats and all nine actual casts,
exact stats/intrinsic keywords, full entry draws, loot and revealed land search.
They check live source/controller/zone/token/type/changeling/other distinctions,
same-turn haste attack rejection and restoration, real flying/reach blocking,
lifelink/deathtouch combat, trample assignment, vigilance tapping, paid temporary
pump and expiry, attack loot, illegal/stale choices and atomic rollback when a
conditional lifelink gain overflows during real damage.

Three funding tests compare concrete and virtual keyword state after real
nontoken/token mana sacrifices, preserve the original game, and exercise all
shared quantity variants plus virtual actor life. Four compiler tests protect
exact predicates, compound residuals, whole-body admissions and negative scopes.
The scaling-source inventory now checks all 25 sources. One fresh release
Python method runs nine real-game scenarios to every conditional keyword and
stat bonus, independently calculates both public values, checks private
ownership and replays source/cost/private boundaries including threshold changes.

Two native Standard100 runs use two legal 60-card deck families and reproduce:

- 70,448 decisions; casts `[163,78,85,198,107,253,122,236,228]` in table order.
- Triggers/finished triggers `[0,78,168,0,0,0,0,236,0]`; 57 Barrow pump activations.
- Enabled observations `[17778,4947,5645,10477,2531,17448,2244,783,822]`.
- 200,704 independently checked public keyword/stat pairs, including an
  exact-incarnation/turn ledger for resolved Barrow pumps. The expected values
  use public card definitions, public zones and separate condition arithmetic.
- 68,352 creature-history checks; 833 owned private scry/search/loot choices;
  16,624 native replay boundaries with stale-revision rejection.
- 20,616 maximum initialized logical state bytes in this workload.

Complete checks passed: **1,043 Rust tests** (1,031 unit, 12 integration),
**198 compiler/data tests**, **139 fresh release Python tests**, all-target
Clippy with warnings denied, formatting, catalogue consistency and diff checks.

The separate 1,000-game native Freeform benchmark completed 571,049 decisions at
roughly 376,554 decisions/s and 659 games/s, clone 324 ns and restore 296 ns in
this run. Initial logical state remains **19,784 bytes**. Timing is
host-dependent; the benchmark includes policy and observation construction,
excludes neural inference/tensors/JSON, and is not an XMage parity comparison.
