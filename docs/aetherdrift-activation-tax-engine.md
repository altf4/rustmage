# Named activation increase engine contract

This engine-only cluster supplies the remaining cost dependency for Skyseer's
Chariot. It does not admit or edit ordinary DFT definitions. The root integration
owner has run the eighteen `dft_activation_tax` scenarios: seventeen pass at
`db13683e8`; the initial-parent deferred grant scenario is the remaining case
addressed by the ManaOutput repair below. The eight new `dft_mana_output`
scenarios await the root owner's guarded run. No Cargo runs in this isolated
worktree.

## Pinned rule evidence

The full Oracle body and structured-fact hash are in
[`aetherdrift-quantities-combat-sources.json`](aetherdrift-quantities-combat-sources.json):
Skyseer's Chariot, Oracle ID `92d28545-4403-4c24-8547-efab4dde6a53`, facts SHA-256
`ed52aeaeb3e26e2a769eb73bc34f8c38e3ba14d62b628ac6b5ba2fdbe49c3ec4`.
The full Java class at XMage revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7` is
`Mage.Sets/src/mage/cards/s/SkyseersChariot.java`, SHA-256
`2018997694a4827146adf0f5c93d54b4b5da5cf7a9e873e2b5a8aa543a8c8ce6`.
Its static effect asks for the actual activated source object, matches its chosen
name, and uses `CostModificationType.INCREASE_COST` without excluding mana
abilities, opposing controllers, crew, saddle, or nonbattlefield activations.

The full reusable `CostModificationEffectImpl`, `ChooseACardNameEffect`, and
`NamePredicate` bodies were read. The relevant complete `CardUtil` increase,
adjustment and name-matching methods were also read; their containing-file
hashes are recorded in the same JSON. XMage supplies source evidence; the typed
implementation follows CR601.2f/602.2b increase-before-reduction ordering and
CR605.3c's exact active-instance restriction rather than adopting XMage's joined
display-name parsing or automatic multi-option payment shortcuts.

## Cost determination and origin

`NamedActivationGenericIncrease(u32)` is a generic immutable static fragment.
`CharacteristicWorld::named_activation_generic_increase` uses the activated
object's effective current name and exact chosen-value origin, sums independent
active fragments from either controller, and tests each fragment's own source
group under removal-aware characteristic evaluation. Providers remain available
to projected worlds even when currently suppressed. The ability's immutable
defining card is separate from the actual source's name.

The cost projection includes raw base, X and increases before typed and generic
reductions, including excess colored reduction applied to generic mana. An
explicit alternative choice owns a cold `ActivationCostChoice` receipt with its
raw base, typed reduction and route. Final cost determination resamples current
increases/reductions after the last target/X choice, then writes the existing
`total_cost` field before any mana payment. Saturating a reduction never destroys
the operands needed for that lock. Subsequent provider departure or source-name
change does not recompute this parent total. Replacement equip routes replace
only the printed base, then undergo the same increases and reductions.

Ordinary indexed activations, acquired self templates, source-max templates,
external attachment/planeswalker templates, inherited donor slots, hand and
graveyard origins use this projection. Nonbattlefield starts use the owner and
exact source zone. An inherited packet clears unrelated same-index self-template
provenance before installing its immutable donor definition and donor slot.

## Mana and nested payment

Untaxed intrinsic and pure layer-six mana retain their direct actions. Taxed
intrinsic mana uses the existing generic intrinsic templates. Taxed layer-six
mana uses two generic immutable tap/tap-sacrifice templates and
`AnnounceGrantedMana { card, grantor, production }`. No card/name table is added
to engine code. The grantor and selected concrete production identify the actual
grant; quality is captured at announcement from that production's original
source and stays owned if the provider leaves before payment.

Resolution-granted templates have an explicit
`AnnounceSelfGrantedMana { source, grant_timestamp, counter_payment, production }`
route. Funding retains their definition and acquisition timestamp independently
of indexed and donor slots. Existing sparse granted-use rows retain independent
once-per-turn/exhaust use; a printed ability's use bit is not borrowed by a
separate acquired instance. Intrinsic reserved indices do not occupy printed
use bits.

Actual mana windows and speculative funding share immutable activation packet
and payment constructors. Source-zone mana participates using owner authority,
exact origin, legal exile/discard costs and actual departure incarnation. New
starts reject every active ability instance, including all ancestors. Completion
uses the frozen child packet, exact selected external cost and, when supplied,
exact physical payment vector before lot allocation. Projected completion removes
only that resolved child from the active chain before examining its parent; a
fresh activation of that now-resolved ability may then help fund the parent.
No candidate search clones a Game or enumerates contributor powersets.

## Crew and saddle costs

A selected group is a non-targeting cost receipt containing exact source,
controller, selected incarnations and completion state. A positive tax opens the
ordinary payment window after selection; selected creatures remain untapped
until every cost has been paid. The funding requirement checks current signed
power or available toughness substitution, the qualifying two-power bonus, and
current eligibility in each exact final projected payment world before the
simultaneous contributor tap. A partial selection uses positive unselected
contributions only as a bound at that same leaf, never as a chosen hidden subset.
Every selected prefix and Finish action must still have a complete joint payment.
An untaxed zero-mana-component activation does not acquire a mana window merely
to change contributor characteristics.

Paid taps and participant events retain their earlier contracts: all
contributions are checked before taps, participant abilities are frozen from
post-tap facts under CR603.10, and countering the resulting crew/saddle ability
cannot refund costs or undo those event packets. Existing zero-cost completion
and replacement suspension routes remain available. Completed actor-only group
views survive nested payment, and their owned state cannot be erased into a
schema198-compatible snapshot projection.

## Authored scenario contracts and limits

The fourteen focused scenarios cover both controllers, zero printed mana costs,
raw alternative receipts, X and reduction floors, provider removal after locking,
Hand/GY owner authority, taxed intrinsic mana, a restricted-quality granted mana
ability whose provider leaves during nested payment, inherited source names and
donor slots, independent acquired once-per-turn mana instances, coupled crew and
saddle prefix/Finish/mana reservations, static suppression/restoration, max-speed
graveyard mana with atomic exile, target-to-cost-lock timing, current copied
names, and both structured halves of an external split source. Rejections retain
snapshot/revision; raw/indexed/restore branches own their receipts after disposing
of their parent Game. These are outcome contracts until root validation consumes
the committed inputs.

Current admitted battlefield/stack name projections, including copy/name changes
and selected faces, are authoritative. External split names come from the pinned
structured name table. Battlefield Room doors and their unlocked-name sets are
an existing gated engine blocker: this cluster does not introduce door state,
parse display separators, admit Rooms, or claim full Standard interactions with
unsupported Room battlefield behavior. All DFT members are single-name sources.
The whole-pool goal remains open through final integration, card implementation,
independent reviews, broad rules checks and measured simulation throughput.

## Post-integration exact-source and quantity review

Independent scoped review found two contribution edges, repaired without adding
Game state. `crew_saddle_subject_matches` uses current subtype only for the exact
live source incarnation. After departure, projected `payment_departures` and the
already refreshed active/suspended `SpellView.source.facts` provide the latest
card/subtype witness. A preflight fallback is used only without a departure
witness. Final paid sums can use the paid item's facts after its announcement
header has left State. A new incarnation in the same CardId slot cannot supply
the old qualifier. Copied-Vehicle sacrifice plus slot reuse and a later copy/type change before departure and a new Vehicle incarnation have contrasting scenarios.

The signed base and assistance bonus are now separate exact PowerSum terms.
A positive base at i128::MAX plus two is never saturated before cancellation
with a negative selected contribution. A Crew3 scenario selects MAX(+2) and
-MAX+2, producing the exact legal total4. Any test-only i128 convenience
projection fails closed if the individual value cannot be represented.
The appended projected power-up scenario also verifies the shared world query
for entry-turn reduction after a new projected incarnation. These follow-up
scenario outcomes remain pending the integration owner's guarded run.

The initial integrated fourteen-case run passed twelve cases. It exposed an old
self-grant creation assertion which still rejected mana, once-per-turn and
exhaust templates despite the new exact acquired-use and mana paths. The guard
now permits those supported lifecycles, while retaining its explicit unsupported
source-zone/power-up lifecycle boundary. The retained self-granted nested/once
scenario verifies the repair; its outcome is pending the next root run. The
other failed case revealed initial-parent funding's missing deferred-grant
snapshot route and remains open while the shared ManaOutput completion repair
is authored. Neither failing expectation was weakened.

## Deferred initial funding and post-cost ManaOutput

A cold optional owned `ManaOutput` is shared by the actual announcement and the
speculative funding move. It contains the immutable production recipe, exact
ability instance, original controller, captured quality, optional color or exact
Combination allocation, source-power fallback, speculative deferred cost, and
an optional owned resume Pending for mana started outside a parent announcement.
It adds no public action or view. Active and suspended announcement byte accounting
includes the receipt and its resume state; old schema projection rejects it.
Ordinary fixed direct mana keeps its original route.

A currently available source can be frozen before mana to pay its own activation
cost exists. Funding starts its exact child payment context, excludes every active
ancestor instance, visits exact reachable cost payments and lot allocations,
commits source and external costs, and only then evaluates the output. It restores
all parent cost context and resumes the enclosing goal without cloning a Game.
The already announced recipe, definition, grantor and spending quality survive
provider departure. A source departure never applies unpaid use/counter/tap effects
to a later incarnation. Parent contributor reservations and donor/use history
remain independent of the temporary child context.

SourcePower, tapped-source power, subtype counts, distinct creature powers and
monocolored permanent colors are read after all costs. Controller expressions
use the original activation controller. Exact projected departure facts extend
the existing sparse card/subtype witness with sampled power, including its memo
key; actual completion uses the refreshed paid packet's departure facts. A new
incarnation cannot replace either witness. This follows the current-information
and exact last-known-information rule in CR608.2h; mana activation and immediate
resolution follow CR605.3.

For a variable color choice, an all-zero announcement is an unselected late route,
not a chosen color. If cost payment increases a zero quantity, the existing owned
EffectMana/ChooseMana window chooses from the final domain. Early-selected colors
remain an optional atomic shortcut and are applied to the final quantity. Fixed
Combination allocations stay exact. A completed rollback event records actual
concrete production and captured quality once; retained replay consumes this frozen
result rather than sampling the recipe again.

The new outcome contracts cover post-counter quantities in actual and initial
parent funding, positive and rejected stale previews, nested sacrifice changing
an Elf count, zero-to-positive late choice, exact latest departure power versus a
new incarnation, original controller after departure/control change, fixed late
Combination allocation, and completed late mana retained after a foreign Aura
entry strands the cast. Late receipt and retained-replay branches compare raw,
indexed and restored owned states. These outcomes are pending integrated validation.

The root owner has separately confirmed all acquired-use, projected power-up,
exact copied-source subtype/LKI and signed MAX-cancellation follow-ups pass;
those expectations were preserved. The source-subtype fixture uses a legitimate
copy transition rather than treating Vehicle as a creature subtype. The sparse
static-provider index now opts in named activation increases, avoiding a scan of
all library/hand objects in ordinary mana funding.

## Dynamic pruning review follow-up

Fresh nonauthor review found an optimistic color bound still sampling dynamic
production before costs, and a granted-mana bound passing an unevaluated recipe
to the concrete-output visitor. All total/color pruning now uses the same
conservative recipe ceiling. Source power and distinct-power quantities retain
the representable ceiling; subtype counts may count every represented object;
chosen or monocolored colors overcount all five colors. Only the exact post-cost
visitor determines legal production. The contrasting printed and granted
scenarios remove a negative counter to raise power from one to two, pay an actual
{1}{G} parent, and reject the wrong color or insufficient fixed quantity without
mutation. These two new scenarios await root validation.

The ordinary fixed-output fast path adds its result directly to the already
owned post-cost FundingState. It no longer clones that state a second time;
dynamic choices continue to branch for distinct concrete outputs.
