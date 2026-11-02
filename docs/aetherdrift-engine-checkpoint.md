# Aetherdrift engine dependency checkpoint

Current acceptance: all 276 card definitions and whole-card receipts, all 3,777 native cases, the fixed 80 DFT games, and full production release/binding validation pass. See `aetherdrift-acceptance.json`. The inventory below preserves the initial engine audit and later checkpoints.

All 276 pinned Oracle/main XMage implementation bodies and current typed definitions have been read across four disjoint audits. The per-card source pins, obligations, existing primitives, concrete gaps and required observable outcomes are in `docs/aetherdrift-audit/`. This is a dependency audit; actual per-card rules proofs and further reusable-source review remain pending. Starting admission remains 119/276 with no new card admission.

All audited engine dependencies below now have implemented shared support and
passing contract evidence on the independently reviewed frozen baseline. Their
descriptions preserve the initial audit requirements; they do not certify each
card's eventual definition wiring. See `aetherdrift-engine-freeze-initial.json` for the
3329 initial passing cases, trace migration, performance evidence and cache reuse.

The following full dependency inventory deliberately retains each audit ID (overlaps will share one implementation owner). All identified engine changes and their focused/shared contract proofs and independent reviews must finish before ordinary DFT card authorship.

- [x] **max_speed**: Generalize live speed scalar and max-speed qualifications across static stats/keywords, trigger grants, graveyard activation/cast and draw replacements. Preserve layer-six grants rather than merely adding a resolution if.
- [x] **exert_cost**: Add exert as a paid activation cost with exact incarnation and controller next-untap-step skip; it is not an effect and survives countering.
- [x] **participant_events**: Dispatch a crew/saddle payment event per contributing creature with exact Mount/Vehicle as non-targeted event object and declaration phase/controller context.
- [x] **contribution_toughness**: Allow crew/saddle contributions using live toughness instead of power, in funding, incremental selection and commitment; use effective available abilities.
- [x] **saddle_target**: Mark exact target Mount saddled until cleanup as a resolving effect independent of a Saddle activation/payment.
- [x] **static_mount_vehicle_union**: Live controller-qualified Mount OR Vehicle modifier without duplicate bonuses for both subtypes.
- [x] **artifact_spell_or_activation_mana**: Add exact ArtifactSpellOrActivation quality; existing ExceptNonArtifactSpells incorrectly permits Purpose::Other.
- [x] **offturn_creature**: Conditional layer-four addition of creature type when active player differs from source controller; retain intrinsic vehicle P/T.
- [x] **exhaust_activation_event**: Trigger on committing an actual Exhaust activation independent of outcome and source permanent type.
- [x] **controlled_duration_combat_lock**: Linked control effect and cannot-attack/block rule last only while resolving controller continuously controls exact source, irreversibly expire on loss of source control.
- [x] **counter_filtered_grant**: Grant EOT keywords to snapshot of controlled creatures with ANY counters, including sparse named counters, not just +1/+1.
- [x] **captured_defender_attack_group**: One trigger for a declaration attacking observer controller, carrying exact group of attacking creatures for EOT -1/-0.
- [x] **nonland_name_activation_tax**: As-enters nonland card-name choice plus generic activated-ability tax on any source with same rules name, including mana abilities and nonbattlefield activations.
- [x] **remove_keyword_group**: Remove indestructible only from frozen creature OR Vehicle recipient group, then destroy that group with ordinary replacement/indestructible semantics.
- [x] **reduce_target_controller_speed**: After bounce, compare target pre-move controller speed with every other player, decrement strictly leading speed by one with floor one.
- [x] **return_then_animate**: Return target artifact then animate actual resulting incarnation permanently if Vehicle; preserve successful move identity across entry choices/replacements.
- [x] **untap_nonland_group**: Untap all controlled nonland permanents, including artifacts/enchantments/planeswalkers that are not creatures.
- [x] **as_enters_copy**: Optional nontargeted as-enters copy of controlled artifact OR creature, before entry choices/counters/ETB; physical card keeps identity and changes copiable recipe.
- [x] **own_spell_target**: Support unqualified any spell you control target. Existing YourSpellKinds uses u8 mask and requires intersection, so zero never matches and255 misses Battle256-only spells.
- [x] **speed-value**: No StatValue for controller/player speed. Add generic live quantity to actual/speculative worlds and Item AmountValue via Stat. Reuse SourceConditionalKeywords, StatExpr, KeywordGrant, StepGuard::Expression for max-speed effects.
- [x] **graveyard-max-speed**: SourceMaxSpeedActivation enumeration in visit_source_grants currently requires giver.zone == Battlefield; max-speed graveyard activations require zone-aware grant/guard availability, affordability and ownership.
- [x] **exhaust-event**: No ExhaustAbilityActivated trigger qualification. Emit only on actual activation commitment (not failed announcement, copying, resolution).
- [x] **caster-off-turn**: DuringOpponentsTurn checks active player against observer controller; Adrenaline requires actual spell caster != active player, any-player cast, and fixed caster damage recipient. Add distinct event qualification, never change existing wrapper semantics.
- [x] **capped-name-exile-search**: General choose-card-name (all legal names) then optional graveyard/hand/library search capped across all three zones, exiling at most four without replacement draws. Existing CoverUpSearch is based on graveyard choice, unbounded and draws for hand exiles.
- [x] **untargeted-graveyard-kind-subtype**: Need nontargeting mandatory resolution choice of creature OR Vehicle from entire own graveyard after milling (not only newly milled cards); existing RecoverMilled supports Permanent/Land only and selects from milled batch.
- [x] **discard-event-amount**: Discard batch count is already retained for PutDiscardCounters/DamageDiscardAmount but no general AmountValue::DiscardEventCount for CreateTokensScaled. Add captured quantity or specialized generic token-from-discard instruction.
- [x] **embalm-grant**: No embalm: temporary exact-graveyard activation grant priced from printed mana cost, sorcery timing, exile source cost, copiable white/Zombie/no-mana-cost token exceptions. CopyExceptions currently lacks those general exceptions.
- [x] **destruction-result**: Demonic Junker needs simultaneous up-to-one own/one enemy creature destruction and one +2 source counter effect conditional on actual successful destruction of an own creature. Do not infer from selected target or final graveyard result alone.
- [x] **cost-union-kind-subtype**: SacrificeFilter combines kind and subtype conjunctively; cannot express creature OR Vehicle, Mount OR Vehicle. Generalize typed cost/filter predicates with OR while preserving affordability and speculative reservation.
- [x] **after-main-combats**: Existing AdditionalCombat increments count consumed only at EndCombat; Full Throttle needs two inserted combat phases immediately after the current main phase (including Main2), no inserted main phase, and a source-independent begin-each-combat trigger through cleanup untapping all current creatures that attacked this turn.
- [x] **caster-foreign-owned**: CastFilter::NotOwnedByYou compares owner to observer controller; Gonti needs owner != actual caster and token created by actual caster. Add separate filter rather than changing inherited semantics.
- [x] **combat-foreign-face-down-play**: Reuse PlayerDamaged creature filter + captured controller, but add source-independent permanent foreign face-down exile/play permission for exact top-card incarnation, actor-specific look permission and any-mana spending for spells. Existing face-down play primitives are subtype-gated/source controller/self library.
- [x] **capped-rummage**: Existing DiscardAnyDraw supports any number/all and DiscardUpToOneDraw caps one; add bounded optional discard 0..N then draw actual completed count (Greasewrench N=2).
- [x] **delayed-target-dies**: Grim Javelineer creates delayed death trigger controlled by resolving trigger controller, not ability granted to creature. Existing GrantUntilEndTurn queues SelfDies under dying creature controller, so it is not equivalent. Generalize exact target delayed-death registration (ManifestDreadWhenTargetDies has narrow behavior).
- [x] **sacrifice-cost-facts**: Hellish Sideswipe needs frozen subtype of actual additional-cost sacrificed permanent and conditional draw after destruction. Existing scalar sacrifice facts capture power/mana value but no generic paid Vehicle predicate.
- [x] **combat-only-token-requirement**: Howlsquad creates token at begin combat and requires that exact actual token to attack this combat if able, including after source departure. Generic keyword MUST_ATTACK lasts too long unless temporary combat-expiring requirement is added.
- [x] **sacrifice-or-discard-fallback**: Momentum Breaker requires opponent sacrifice a creature/Vehicle if able, otherwise mandatory discard; existing optional ChooseObjects skip means decline not inability and must not supply fallback incorrectly. Add explicit can-pay/failure branch using union predicate.
- [x] **speed-player-damage**: Outpace Oblivion needs damage 2 to each player whose own speed is <4. Add generic per-player speed predicate/scalar or narrow typed effect; do not evaluate only controller speed.
- [x] **filtered-mass-return-delayed-sacrifice**: Push the Limit needs simultaneous own graveyard Mount OR Vehicle return with exact successful entrants remembered for next-end-step delayed sacrifice. Existing ReturnOwnedGraveyardNonlandPermanents and ReturnGraveyardCreatures cannot filter arbitrary subtype union or schedule these delayed sacs; Mobilize only token results.
- [x] **post-mill-target-threshold**: Quag Feast targets creature/planeswalker/Vehicle before milling; compare current target mana value with current controller graveyard size AFTER actual mill then conditionally destroy. Existing target-time max_mana would be wrong. Add conditional typed instruction using target current facts.
- [x] **crew-saddle-participant-event**: No event for this creature paying another object crew/saddle cost during own main. Emit participant event on committed activation/tap payment, freeze exact Mount/Vehicle as untargeted event object, preserving buffs even if crew/saddle ability countered.
- [x] **graveyard-self-cast-additional-cost**: Wickerfolk needs persistent self graveyard cast permission with normal mana plus 2 life and artifact-or-creature sacrifice only when casting through this permission. Existing temporary general permissions and global spell AdditionalCost must not apply costs to hand casts or unrelated permission casts.
- [x] **exhaust_event**: Dispatch committed exhaust events to battlefield and graveyard observers; include mana exhaust and suspended mana funding without double firing on replay; exclude copies/failed activations
- [x] **exhaust_discount**: Typed generic reduction on exhaust activations of other controlled permanents, applied consistently to real and speculative costs
- [x] **exhaust_refueler**: Per-player actual exhaust-activated-this-turn history plus own-turn as-though spent-exhaust permission, not clearing lifetime usage
- [x] **discard_event_amount**: Expose existing DiscardTriggerView.amount through AmountValue rather than use cumulative discarded-this-turn scalar
- [x] **howler_delayed_hit_draw**: Source-independent until-cleanup observer on exact target combat hit; draw for original resolving controller rather than grant creature-controller draw ability
- [x] **mount_vehicle_entry_history**: Count successful entries with Mount or Vehicle subtype at event time per controlling player, even before Coordinator enters
- [x] **filtered_graveyard_departure**: Add artifact-or-creature filter to one-or-more card departure batch trigger preserving frozen observer
- [x] **speed_scalar_max**: Add current controller speed scalar and max-speed source-grant/trigger condition usable by static/trigger quantity machinery; current StartYourEngines and speed increase are present
- [x] **graveyard_max_speed_activation**: Max-speed granted activation at graveyard source zone; existing SourceMaxSpeedActivation visitor only traverses battlefield
- [x] **no_abilities_predicate**: Abilityless-card predicate for graveyard targets and library searches; keyword-free is insufficient because cycling and other nonbattlefield abilities count
- [x] **library_graveyard_hand_search**: Optional library-and/or-graveyard single-card search to hand with reveal and shuffle only if library searched
- [x] **global_exile_batch**: Any physical card from either player battlefield/graveyard to exile during own turn triggers once per simultaneous event with proper pre/post observers; existing observer restricted to own creatures
- [x] **conditional_search_destination**: Library search one artifact to Battlefield ifMV<=2 else Hand; existing conditional library pick is useful implementation but not a search
- [x] **reanimate_permanent_keywords_own_end**: Attach lasting keyword grant and source-independent next-own-end exile to exact actual reanimation entrant
- [x] **pirate_vehicle_attack_history**: Combat-local historical declarations record Pirate+Vehicle membership, including one attacker satisfying both, and survive subtype/departure changes
- [x] **far_fortune_damage**: Max-speed replacement adds1 to any controlled-source damage to opposing player or permanent; existing NoncombatDamageToOpponentPlus excludes combat
- [x] **event_saddle_animation**: Apply direct saddled and animation effects to exact Subject::Entering/Targets without collecting cost helpers; saddle designation not activated ability
- [x] **controlled_add_creature_subtype**: Add Ooze subtype to live controlled creatures in layer4 while independent live6/6 base remains in7b
- [x] **mimeoplasm_entry_linked_copy**: Pre-entry optional exile0..X own creatures, exact linked actual exile result with3 counters each; target linked exile cards and lasting copy0/0 retaining activation; preserve ability origin/use identities
- [x] **optional_target_hand_discard_draw**: Private look at target player hand followed optional selected discard and draw only when card chosen; no discard choice is legal
- [x] **oviya_hand_entry_result**: Condition two post-entry counters on exact actual artifact entering from optional filtered hand choice; existing PutHandFiltered lacks generic resulting entrant handle
- [x] **attacking_opponent_player_keywords**: Keyword grant filters actual attacked player, excluding attacks on planeswalkers/Battles
- [x] **dynamic_search_maximum**: Use current speed to cap search count at resolution without physical source dependence
- [x] **choose_graveyard_card_to_hand**: Non-targeted optional choice from entire current own graveyard after mill; existing SelectMilled only exact mill batch
- [x] **source_counter_parity**: All-counter sum parity including sparse named counters and source-departure LKI for Sab-Sunen second clause
- [x] **group_base_source_power**: Freeze other controlled creatures and signed source POWER once and set both base axes to it until cleanup
- [x] **library_until_nth_kind**: Reveal until second matching land instead of current LibraryLook::UntilKinds first match; whole remaining library if fewer
- [x] **entry_counter_other_mv**: Other controlled creature/Vehicle pre-entry additional counter amount1 ifMV<=4 else3, entering-world characteristic checks and ordinary replacement composition
- [x] **aura_creature_vehicle**: Aura legal union creature or Vehicle subtype; AuraSpec kind bitmask alone cannot require Vehicle rather than every artifact
- [x] **variable_graveyard_exile_x**: X nonmana activation cost selects any number own artifact graveyard cards and captures chosen count; existing ExileGraveyardCards is fixed count
- [x] **target_third_from_top**: Move target permanent to owner library below top2 or bottom if shorter; existing PutSourceThirdFromTop only source
- [x] **chosen_type_vehicle_group**: Add source-chosen creature type to controlled Vehicle creatures in layer four; compose with chosen-type lord bonus.
- [x] **delayed_next_exhaust_copy**: Register source-independent next nonmana exhaust activation observer until cleanup; copy actual captured packet and retarget.
- [x] **opponent_controls_more_lands**: Intervening condition comparing opponent and controller current land counts in actual and speculative worlds.
- [x] **saddle_target**: Direct saddled effect on source/targets/entrant (shared).
- [x] **speed_scalar_and_gates**: Controller speed scalar, max-speed conditional effects and graveyard gates (shared with color audits).
- [x] **target_player_scaled_chosen_mana**: At resolution controller chooses one color; targeted player receives a scaled count from the actual sacrificed-cost facts.
- [x] **variable_sacrifice_activation_cost**: One-or-more artifact sacrifice activation cost with exact successful frozen count and rollback-safe speculative reservations.

## Exact-API recheck followups

Ordinary card authorship is parked until this complete reentry batch passes one
shared guarded build and independent reviews. The prior accepted checkpoint
remains historical evidence; its reusable outputs are preserved.

- [x] **attached_permanent_untap**: Silken Strength needs current attachment/LKI, including noncreature Vehicles.
- [x] **start_engine_source_index**: Register standalone StartYourEngines providers for actual SBA entry.
- [x] **positive_hand_selection**: Mandatory-if-possible positive type union, complete reveal and discard or unlinked exile for Gastal Raider/Intimidation Tactics.
- [x] **negative_kind_group_growth**: Winter samples one signed amount for every other nonartifact creature.
- [x] **simultaneous_hand_permanents**: Redshift privately selects any number, then commits one simultaneous entry batch.
- [x] **conditional_self_graveyard_cast**: Lightwheel Enhancements uses live speed/owner/zone with ordinary casting.
- [x] **live_subject_subtype**: Alacrian Armory branches independently on current Mount/Vehicle subtypes without a creature requirement.

Reentry accepted at `8657c4e6c`: all seven followups and the activated source-quantity lifecycle repair pass 3,415 combined cases, two independent nonauthor reviews, exact inherited trace comparison, strict baseline promotion, and a fresh-engine-free warm-cache check. See `aetherdrift-engine-freeze.json`. Ordinary card work may resume; full276-card set acceptance remains open.

Permanent spell cast-copy reentry accepted at `9000f120b`. The combined checkpoint, prior-schema proof, two independent nonauthor reviews and fresh-engine-free rewarm are recorded in `aetherdrift-engine-freeze.json`. Ordinary card work may resume; full 276-card set acceptance remains open.

Aggregate target-group reentry accepted at `2c7ffccc6`. The combined checkpoint, prior-schema proof, two independent nonauthor reviews and fresh-engine-free rewarm are recorded in `aetherdrift-engine-freeze.json`. Separate Trade/Junker controller-role card repairs may resume; full 276-card set acceptance remains open.

Late mana-completion reentry accepted at `3b29b5575`. All four audited boundaries, all26 new interaction cases, two independent nonauthor reviews and the strict trace/cache proofs are recorded in `aetherdrift-engine-freeze.json`. All276 typed card definitions have focused outcomes; the complete fixed DFT-only game corpus and full release acceptance remain pending.
