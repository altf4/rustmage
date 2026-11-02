# Named activation increase engine contract

This engine-only cluster supplies the remaining cost dependency for Skyseer's
Chariot. It does not admit or edit ordinary DFT definitions. The fourteen
`dft_activation_tax` scenarios are authored but have not been run in this
isolated worktree; the root integration owner runs Cargo once the engine batch
is assembled. No passing tax outcome is claimed by this document.

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
