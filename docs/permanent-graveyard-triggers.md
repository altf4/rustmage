# Permanent battlefield-to-graveyard triggers

Issue: https://github.com/altf4/rustmage/issues/156

Ten complete bodies bring executable coverage to **1,228/5,164**, with **3,936
unsupported**. Six regression identities, 56 token factories and one emblem
remain separate. Standard stays pinned to October 15, 2026.

## Complete bodies and provenance

| Card | Whole body |
| --- | --- |
| Ashiok's Reaper | 3/3; an owned enchantment going from battlefield to graveyard draws a card |
| Hopeful Vigil | Entry creates a 2/2 white vigilant Knight; its battlefield-to-graveyard trigger scries two; {2}{W} ability sacrifices the enchantment on resolution |
| Knight of Doves | 1/3; an owned enchantment going from battlefield to graveyard creates a 1/1 white flying Bird |
| Long Feng, Grand Secretariat | Legendary hybrid 2/3; another owned creature or an owned land going from battlefield to graveyard puts a +1/+1 counter on target owned creature |
| Mephitic Draught | Artifact; entry or battlefield-to-graveyard departure draws a card and loses one life |
| Nutrient Block | Indestructible Food artifact; {2}, tap, sacrifice as a cost to gain three life; battlefield-to-graveyard departure draws a card |
| Reach for the Sky | Flash Aura, enchant creature, +3/+2 and reach; battlefield-to-graveyard departure draws a card |
| Savior of the Sleeping | 2/3 vigilance; an owned enchantment going from battlefield to graveyard puts a +1/+1 counter on itself |
| Warehouse Tabby | 1/1; an owned enchantment going from battlefield to graveyard creates a 1/1 black Rat that cannot block; {1}{B} grants itself deathtouch until end of turn |
| Wicked Visitor | 2/2; an owned enchantment going from battlefield to graveyard makes each opponent lose one life |

`permanent-graveyard-admissions.json` records all ten full Oracle bodies and
41 SHA-256 entries across 33 unique pinned Java sources at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. This includes the card constructors,
shared self/all/entry-or-departure trigger classes, draw/life/counter effects,
sacrifice effect, Food ability, Aura bonus and keyword effects, scry and all three
token recipes. The new factory is the white 1/1 flying Bird. All source hashes
were recomputed from pinned Git objects and compared with the read-only checkout.

## Event model and compiler boundary

Shared zone dispatch now examines every actual battlefield-to-graveyard event.
Self and entry-or-departure triggers apply to the exact captured source object.
Existing creature-death watchers retain an explicit creature check. Creature
death history still increments only for creatures; artifacts and enchantments
do not inflate it. A multitype creature remains one event for each matching
ability, even when multiple type bits match.

The new immutable `PermanentToGraveyard` descriptor stores a kind union, ownership
relation and source-exclusion kind mask. Long Feng excludes its source from the
creature branch while preserving its printed land branch. Exact source and
global artifact/enchantment/printed-union clauses compile into these shared
events. Unknown residual clauses, from-anywhere, leaves-battlefield, nontoken
and aggregated “one or more” variants remain gated in this extension. The
compiler does not infer an event-card antecedent or sacrifice from a departure.

The existing death-source index supplies frozen object views for the entire
pre-event battlefield. Simultaneously departing observers still see matching
departures. Bounce, exile, milling and discard are distinct events. An Aura
that falls off a dead creature gets its own subsequent state-based departure
batch; observers that died in an earlier batch are already absent.

Costs and effects retain their independent timing. Nutrient Block sacrifices as
an activation cost, queues its draw above its life-gain ability, and can gain life
after a later incarnation enters. Hopeful Vigil stays on the battlefield after
payment and sacrifices during resolution; an old ability cannot consume a new
incarnation. Source and target counter effects likewise cannot follow reused
physical card IDs. A counter-capacity failure restores the pending frame,
registries, snapshot state and revision without undoing earlier completed costs.

The [official September 25, 2026 rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)
define battlefield-to-graveyard departure in CR700.4, source look-back and
departure triggers in CR603.6/603.10, independent stack abilities in CR113.7a,
new objects on zone changes in CR400.7 and Aura state-based cleanup in CR704.5.

Wider type/control changes, event-card targeting, copy effects, additional counter
types and unsupported residual rules remain whole-body gated. Bitter Chill,
Demonic Ruckus, Krenko Baron of Tin Street, Tarrian's Soulcleaver, Teysa, Ygra
and Zoetic Glyph remain unsupported. No partial bodies were admitted.

## Validation and replay workload

Thirteen native tests cover both seats, all ten bodies, complete casts and
activations, typed token recipes, real destroy versus indestructible/sacrifice,
counter/tap costs versus resolution effects, private scry, ownership and type
filters, APNAP ordering, dying observers, multitype creatures, source/target
incarnations, Aura cleanup and counter-capacity rollback. Clone/restore and stale
action revisions are checked at owned decision boundaries. The existing complete
source-index test now classifies the new descriptor too.

Two compiler tests protect exact aliases and kind-scoped exclusions, all ten
whole-body admissions, unknown residuals, ambiguous event-card antecedents,
ownership and unsupported event variants. One fresh release Python test exercises
all ten bodies through actual casts, owned costs/targets, self-sacrifice and
destruction, library choices, observer privacy and cloned/restored resolution.

Complete checks passed: **959 Rust tests** (947 unit, 12 integration), **180
compiler/data tests**, **130 fresh release Python tests**, all-target Clippy
with warnings denied, formatting, catalogue consistency and diff checks.

Two native Standard100 runs reproduced exactly, with vectors in the table's order:

- 106,571 decisions; casts `[321,316,124,229,347,405,259,132,368,140]`.
- Queued graveyard triggers `[80,275,45,238,75,401,210,45,46,39]` and finished
  triggers `[79,275,45,236,75,401,206,45,46,39]`. Games can end with queued frames.
- 103,697 creature-history checks, 1,068 private library choices and 38,469
  clone/restore replay boundaries.
- 21,656 maximum initialized logical state bytes.

Two complete Standard60 deck families exercise owned enchantment sacrifice,
artifact destruction, Food costs, enchanted-creature sacrifice/Aura cleanup,
and Evolving Wilds land sacrifice with Long Feng. The history checker's explicit
fixture assumptions are no exile/bounce or zero-toughness incoming creatures;
departed creature tokens cease before observation. Attacks wait until turn twelve;
mana sources are used in owned payment windows. This is an integration workload,
not a competitive deck or neural policy.

The native 1,000-game Freeform benchmark completed 571,049 decisions at 396,986
decisions/s (695 games/s), clone 127.95 ns and restore 116.69 ns. It includes smoke
policy and observations, excludes a model/tensors/JSON, and is not an XMage parity
comparison. Timings are measurements rather than guarantees.

No mutable fields or new continuations were added. Initial logical state stays
**19,784 bytes**; schema 49, C ABI 1 and control protocol 2 are unchanged. Existing
registries and owned stack/library choices are included in snapshots. Logical
accounting excludes allocator/capacity overhead, immutable catalogue definitions,
external observations and separately retained snapshots. Catalogue fingerprint:
`951574a8bf42f1addd3fcef1e18b37d9d7c8d4c161ae4bf2227acd8ba934fc37`.
