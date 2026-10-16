# Locked generic spell-cost reductions

Issue: [#110](https://github.com/altf4/rustmage/issues/110).
Twenty complete Standard bodies become executable, with every other clause
compiled alongside its cost rule:

| Card | Complete admitted behavior |
| --- | --- |
| Arcane Epiphany | Controlled Wizard discounts self by one; draw three |
| Archmage of Runes | Instant/sorcery discount; completed instant/sorcery cast draws |
| Ballyrush Banneret | Kithkin or Soldier spell discount, once per source |
| Dragonlord's Servant | Dragon spell discount |
| Focus the Mind | Prior spell this turn discounts self by two; draw three then discard |
| Geist of Saint Thalia | Flying; noncreature spell discount |
| Geyser Drake | Flying; spell discount during other players' turns |
| Gigastorm Titan | Prior spell this turn discounts self by three; 4/4 |
| Highspire Bell-Ringer | Flying; the second spell each turn is discounted by one |
| Honest Rutstein | Entry returns an owned graveyard creature; creature spell discount |
| Mistmeadow Council | Controlled Kithkin discounts self by one; entry draw |
| Mocking Sprite | Flying; instant/sorcery discount |
| Orysa, Tide Choreographer | Owned creatures' total toughness at least ten discounts self by three; entry draws two |
| Pearl of Wisdom | Controlled Otter discounts self by one; draw two |
| Saved by the Shell | Controlled Turtle discounts self by one; owned target counter plus temporary trample/hexproof/indestructible |
| Stormcatch Mentor | Haste, prowess; instant/sorcery discount |
| Uthros Psionicist | The second spell each turn is discounted by two |
| Venom's Hunger | Controlled Villain discounts self by two; destroy creature then gain two |
| Visions of Villainy | Controlled Villain discounts self by one; draw two then lose two |
| Voyager Quickwelder | Artifact spell discount; artifact creature |

[spell-reduction-admissions.json](spell-reduction-admissions.json) records complete
Oracle bodies and Java hashes at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Coverage is 1,063/5,164 Standard
identities, six regression definitions, 48 token factories and one emblem.
Unknown history, target-dependent costs, colored reductions, cost increases,
variable reductions, affinity and residual unsupported bodies remain gated.

Immutable rules describe self/global scope, a type/subtype union, fixed generic
amount, turn relation and a live predicate. Only owned battlefield sources apply
global reductions. The shared source index tracks entry, departure and restore.
A union matches once, including a changeling that is both Kithkin and Soldier.
Multiple sources add, generic costs stop at zero, and colored requirements and
printed mana values remain unchanged. Saturating the accumulated reduction at
`u32::MAX` is exact for all representable generic requirements. No recursive cost
or stat cache is introduced; fixed zero-generic costs skip reduction scans.

The second-spell predicate uses completed casts this turn, including countered
spells. Declaring a spell and playing a land do not increment that count. Self
conditions use current controlled permanents or prior casts; Orysa uses live
battlefield toughness before it enters. Subtype predicates retain changeling and
Kindred behavior. Cost rules never change an activation or cycling cost.

The staged path chooses the nonhybrid equivalent, declares targets, then locks a
separate owned total before additional-cost selection, mana abilities and payment.
Funding and `PayCosts` use that total even after a reducer leaves. The committed
base equivalent remains in `announcing_cost`; the existing `payment_request`
projects the actual total. Before locking, that request is a live estimate for
the admitted target-independent reductions. Prefunded shortcuts evaluate the same
rules and deduplicate hybrid allocations. All compiler equivalents, funding,
submitted-action checks and payment paths agree. These distinctions follow
CR 601.2b and 601.2f–i in the
[official Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt).

The new optional locked total belongs to the existing owned announcement and is
included automatically in clone/snapshot equality. Initial logical state grows
by 32 bytes to 19,680; no per-object field, new dynamic buffer or public schema
field is added. Observation 41, ABI 1 and protocol 2 remain unchanged. One older
synthetic reservation test now explicitly replaces its synthetic locked total
alongside the base requirement; real declarations cannot reprice a locked spell.

Author checks pass 740 Rust tests (728 unit and twelve integration), 133
compiler/data tests, 95 fresh release Python tests, consistency, rustfmt and
Clippy with warnings denied. Twenty-one focused native tests cover every new
body, filter/controller/turn/ordinal boundaries, tribal overlap, multiple-source
floors, source-index restore, printed value, hybrid allocation deduplication,
locked costs, legal additional sacrifice, modal ordering, private discard,
completed-cast triggers and numeric payment validity. Three Python methods cover
reduced fixed/two-brid payment, sacrificing the reducer during payment, and
private self-discount draw/discard through the actual native library with reusable
snapshots. Compiler tests prove exact extraction and full residual-body gates.

The public `standard_spell_reductions` workload completes 100 legal Standard
games: 68,396 decisions, 2,025 staged payments, 15,493 independent public checks,
26,470 cloned/replayed boundaries and 21,248 maximum initialized state bytes.
All twenty cards are cast, every reduction rule qualifies on a paid spell, and
all new spell/entry/trigger bodies are sampled. The checker separately evaluates
known Oracle predicates from public objects/counts, freezes its expected total
before payment, validates every offered fixed-cost cast allocation, then checks
actual payments and resolution facts. It preserves pre-draw observations across
private discard choices and skips exact draw-body checks on insufficient libraries;
focused tests and existing decking coverage handle that boundary.

The policy prefers staged new cards, delays attacks until turn eighteen, leaves
blocks available and holds one Arcane Epiphany response window to sample Geyser
Drake's other-turn discount. It stops holding after that rule has been used.
These workload choices do not change engine legality. Four supported decks and
this policy do not establish complete Standard conformance or training readiness.

An unpinned 1,000-game baseline records 571,049 decisions, roughly 395k native
decisions/s, 692 games/s, 156 ns clone and 114 ns restore. It includes native
policy/observations and excludes inference, tensor encoding and JSON. Byte
estimates exclude capacity, allocator/COW metadata, static definitions and
external snapshots. These timings establish no speed change or Java parity claim.
Full Standard and broader native RL/search acceptance remain incomplete, with
4,101 unsupported identities; the full goal remains active.
