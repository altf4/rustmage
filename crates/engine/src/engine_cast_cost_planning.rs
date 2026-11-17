//! Local validation for declared nonmana clauses and supplied material selections.
use super::*;
#[cfg(feature = "test-support")]
use crate::cards::ZoneReplacementRule;
use ability_world::CharacteristicWorld;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone)]
pub(super) struct CostContext {
    actor: Player,
    source: ObjectRef,
    source_tap_reserved: bool,
    source_counter_reservation: Counters,
    clauses: Vec<CastCostRow>,
    active: Option<u16>,
    casualty: Vec<CasualtyDeclaration>,
    conspire: Vec<ConspireDeclaration>,
    additional: Option<AdditionalCost>,
    additional_life: bool,
    forage: Option<ForageRoute>,
    origin_forage: Option<forage::ForageSelection>,
    paid_evidence: Vec<ObjectRef>,
    selected: Vec<ObjectRef>,
    exact: bool,
    counters: Vec<(ObjectRef, CounterKind)>,
    #[cfg(feature = "test-support")]
    mana: Option<ManaCost>,
    teammates: Vec<ObjectRef>,
    teamwork_exact: bool,
    #[cfg(feature = "test-support")]
    delve_selected: bool,
    #[cfg(feature = "test-support")]
    pub(super) tap_payment_used: usize,
    /// Existing announcement phase, retained only in this private query context.
    mana_closed: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CostContext {
    #[cfg(feature = "test-support")]
    pub(super) fn crafts_permanents(&self) -> bool {
        self.clauses.iter().any(|row| {
            !row.paid
                && (row.clause == CastCostClause::CraftSource
                    || row.clause == CastCostClause::Additional
                        && self.additional.is_some_and(|spec| {
                            matches!(
                                spec.kind.payment_kind(),
                                EffectChoiceKind::ExileCraftNonlands(_)
                                    | EffectChoiceKind::ExileCraftMaterials(_)
                            )
                        }))
        })
    }

    #[cfg(feature = "test-support")]
    pub(super) fn needs_web_return(&self) -> bool {
        self.clauses
            .iter()
            .any(|row| !row.paid && matches!(row.clause, CastCostClause::WebSlingingReturn(_)))
    }

    fn from_announcement(a: &Announcement) -> Option<Box<Self>> {
        let c = a.cast_costs.as_ref()?;
        let mut selected = Vec::new();
        let mut exact = false;
        // Paid selections remain on the announcement as receipts. Only an
        // unpaid printed cost can supply the current choice's prefix.
        if !a.additional_paid {
            selected = a.additional_group.as_deref().cloned().unwrap_or_default();
            if selected.is_empty() {
                selected.extend(a.additional);
            }
            exact = a.additional.is_some() || a.additional_group_complete;
            if let Some(f) = &a.forage {
                selected = f.selected.clone();
                exact = f.complete();
            }
        }
        if let Some(e) = a.evidence.as_ref().filter(|e| !e.paid) {
            selected = e.selected.clone();
            exact = e.complete;
        }
        let mut source_tap_reserved = false;
        let mut source_counter_reservation = Counters::default();
        if let StackKind::ActivatedAbility { ability } = a.item.kind
            && a.activation_cost_route == ActivationCostRoute::Printed
        {
            let cost = a
                .item
                .source
                .expect("activated ability has a source")
                .card
                .activation(ability)
                .cost;
            // Retain the frozen definition and selected route, including a
            // borrowed or granted ability whose source has different rules.
            source_tap_reserved = cost.tap;
            source_counter_reservation = a.counter_payment.clone();
            if let Some(change) = cost.loyalty.filter(|change| *change < 0) {
                source_counter_reservation
                    .add(CounterKind::Loyalty, u64::from(change.unsigned_abs()));
            }
        }
        Some(Box::new(Self {
            #[cfg(feature = "test-support")]
            delve_selected: a.convoke.as_ref().is_some_and(|f| !f.delved.is_empty()),
            #[cfg(feature = "test-support")]
            tap_payment_used: a
                .convoke
                .as_ref()
                .map_or(0, |frame| frame.substituted.len()),
            mana_closed: c.mana_closed,
            source_tap_reserved,
            source_counter_reservation,
            teammates: c.teammates.clone(),
            teamwork_exact: false,
            actor: a.item.controller,
            #[cfg(feature = "test-support")]
            mana: a
                .convoke
                .as_ref()
                .map(|c| c.remaining)
                .or(a.total_cost)
                .or(a.cost)
                .or_else(|| {
                    if !a.ordered_craft() {
                        return None;
                    }
                    let StackKind::ActivatedAbility { ability } = a.item.kind else {
                        return None;
                    };
                    let _ = ability;
                    a.cost_options().next()
                }),
            source: a
                .item
                .source
                .expect("cost announcement has object source")
                .object,
            clauses: c.clauses.clone(),
            active: c.active,
            casualty: a.casualty.as_deref().cloned().unwrap_or_default(),
            conspire: a.conspire.as_deref().cloned().unwrap_or_default(),
            additional: a.printed_additional_cost(),
            additional_life: a.additional_life,
            forage: a.forage.as_ref().and_then(|v| v.route),
            origin_forage: c.origin_forage.as_deref().cloned(),
            paid_evidence: c.paid_evidence.clone(),
            selected,
            exact,
            counters: a
                .creature_counter_cost
                .as_deref()
                .cloned()
                .unwrap_or_default(),
        }))
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FundingState<'_> {
    pub(super) fn beam_return_prohibited(&self, candidate: ObjectRef) -> bool {
        self.attachment_sources().iter().copied().any(|id| {
            let aura = self.object(id);
            aura.zone == Zone::Battlefield
                && aura.attached_to == Some(candidate)
                && CharacteristicWorld::printed_abilities_available(self, id)
                && CharacteristicWorld::definition(self, id)
                    .static_ability_groups()
                    .iter()
                    .flat_map(|group| group.fragments)
                    .any(|fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::AttachedCannotBeBeamedUp
                        )
                    })
        })
    }

    pub(super) fn mana_window_open(&self, goal: &FundingGoal) -> bool {
        goal.requires_mana() && self.cast_costs.as_ref().is_none_or(|c| !c.mana_closed)
    }
    /// Keep a fixed mana requirement for reveals, graveyard exiles, craft and
    /// witnessed returns. The fixed-world proof separately excludes departures
    /// that could restore mana sources or change an attached recipient.
    #[cfg(feature = "test-support")]
    pub(super) fn fixed_mana_costs(&self, goal: &FundingGoal) -> bool {
        self.cast_costs.as_ref().is_none_or(|c| {
            c.mana
                == Some(ManaCost {
                    colored: goal.required,
                    generic: goal.generic,
                })
                && c.clauses
                    .iter()
                    .any(|row| !row.paid && row.clause == CastCostClause::Mana)
                && c.clauses
                    .iter()
                    .filter(|row| !row.paid)
                    .all(|row| match row.clause {
                        CastCostClause::Mana
                        | CastCostClause::Behold(_)
                        | CastCostClause::CraftSource
                        | CastCostClause::WebSlingingReturn(_)
                        | CastCostClause::BeamMeUpReturn(_, _) => true,
                        CastCostClause::Additional => c.additional.is_some_and(|spec| {
                            matches!(
                                spec.kind.payment_kind(),
                                EffectChoiceKind::Behold
                                    | EffectChoiceKind::BeholdMany(_)
                                    | EffectChoiceKind::CollectEvidence(_)
                                    | EffectChoiceKind::ExileGraveyard
                                    | EffectChoiceKind::ExileGraveyardCards(_)
                                    | EffectChoiceKind::ExileGraveyardRange { .. }
                                    | EffectChoiceKind::ExileGraveyardSymbols { .. }
                                    | EffectChoiceKind::ExileCraftNonlands(_)
                                    | EffectChoiceKind::ExileCraftMaterials(_)
                            )
                        }),
                        _ => false,
                    })
        })
    }
    /// In the fixed tap-only proof, a live exile link is inert unless one of
    /// the remaining ordered costs can remove its source. Unknown cost packets
    /// are rejected separately by `fixed_mana_costs`.
    #[cfg(feature = "test-support")]
    pub(super) fn fixed_mana_link_can_return(&self, goal: &FundingGoal, source: ObjectRef) -> bool {
        if self.reference(source.card) != source
            || self.object(source.card).zone != Zone::Battlefield
        {
            return true;
        }
        if goal
            .source
            .as_ref()
            .is_some_and(|(object, index, _, card, _)| {
                *object == source && {
                    let cost = card.activation(*index).cost;
                    cost.sacrifice || cost.exile_self
                }
            })
        {
            return true;
        }
        self.cast_costs.as_ref().is_some_and(|c| {
            c.clauses
                .iter()
                .filter(|row| !row.paid)
                .any(|row| match row.clause {
                    CastCostClause::CraftSource => c.source == source,
                    CastCostClause::Additional => c.additional.is_some_and(|spec| {
                        !matches!(
                            spec.kind.payment_kind(),
                            EffectChoiceKind::Behold | EffectChoiceKind::BeholdMany(_)
                        ) && self.cost_object(spec, source, c.source)
                    }),
                    CastCostClause::WebSlingingReturn(selected) => {
                        selected.is_none_or(|selected| selected == source)
                            && self.cost_object(web_slinging::RETURN_COST, source, c.source)
                    }
                    CastCostClause::BeamMeUpReturn(selected, provider) => {
                        Some(source) != provider
                            && selected.is_none_or(|selected| selected == source)
                            && self.cost_object(web_slinging::BEAM_RETURN_COST, source, c.source)
                    }
                    _ => false,
                })
        })
    }
    /// Intrinsic land mana cannot repair a counter shortfall. Avoid enumerating
    /// equivalent land subsets while retaining the complete search for costs or
    /// mana abilities that can introduce counters, objects, or new abilities.
    #[cfg(feature = "test-support")]
    pub(super) fn fixed_origin_counter_deficit(&self) -> bool {
        let Some(c) = &self.cast_costs else {
            return false;
        };
        let mut required = 0;
        for row in c.clauses.iter().filter(|row| !row.paid) {
            match row.clause {
                CastCostClause::OriginCounters(amount) => required = required.max(amount),
                CastCostClause::Mana => (),
                _ => return false,
            }
        }
        let state = &self.game.state;
        if required == 0
            || self.continuation.is_some()
            || self.entry_projection.is_some()
            || !state.exile_links.is_empty()
            || state.borrowed_links.is_some()
            || state.graveyard_borrowing.is_some()
            || state.borrowed_slots.is_some()
            || state.activation_observers.is_some()
            || state.temporary_abilities.is_some()
            || state.self_granted_abilities.is_some()
            || !state.attachment_sources.is_empty()
            || !state.tap_sources.is_empty()
        {
            return false;
        }
        let mut upper = 0u128;
        for id in 0..state.objects.len() {
            let id = id as CardId;
            let object = self.object(id);
            // Include inactive and nonbattlefield definitions: a change in
            // availability must not hide a future counter-producing route.
            if object.card.borrows_activations()
                || object
                    .card
                    .activations()
                    .iter()
                    .any(|ability| ability.mana_production().is_some())
                || !CharacteristicWorld::granted_mana(self, id).is_empty()
            {
                return false;
            }
            if object.zone == Zone::Battlefield {
                // Ignore control and creature restrictions for an overestimate.
                upper =
                    upper.saturating_add(CharacteristicWorld::counter_amounts(self, id).total());
            }
        }
        upper < u128::from(required)
    }
    /// Reject an unpaid additional cost only if even an overestimate of the
    /// resources available after funding cannot satisfy any of its routes.
    /// Unknown costs and worlds with linked or nested state changes retain the
    /// complete search. Counts deliberately ignore filters, control, names,
    /// counters and tapping, all of which can change during funding.
    #[cfg(feature = "test-support")]
    pub(super) fn additional_cost_deficit(&self) -> bool {
        let Some(c) = &self.cast_costs else {
            return false;
        };
        let Some(spec) = c.additional else {
            return false;
        };
        if !c
            .clauses
            .iter()
            .any(|r| !r.paid && r.clause == CastCostClause::Additional)
            || !self.game.state.exile_links.is_empty()
            || self.continuation.is_some()
            || self.entry_projection.is_some()
        {
            return false;
        }
        if self.unchanging_behold_witness_deficit(c, spec) {
            return true;
        }
        // These clauses can consume resources, tap, add counters, or move a
        // battlefield object to a graveyard. Without linked returns they cannot
        // increase life or introduce a card to hand or a new battlefield object.
        // Keep this exhaustive so new ordered clauses require a bound audit.
        for row in &c.clauses {
            match row.clause {
                CastCostClause::OriginCounters(_)
                | CastCostClause::OriginForage
                | CastCostClause::OriginEvidence(_)
                | CastCostClause::OriginDiscard(_)
                | CastCostClause::Additional
                | CastCostClause::CraftSource
                | CastCostClause::Mana
                | CastCostClause::Life(_)
                | CastCostClause::AlternativeLife(_)
                | CastCostClause::Teamwork(_)
                | CastCostClause::Blight { .. }
                | CastCostClause::Casualty(_)
                | CastCostClause::Conspire(_)
                | CastCostClause::Harmonize(_) => {}
                CastCostClause::SneakReturn(_)
                | CastCostClause::WebSlingingReturn(_)
                | CastCostClause::BeamMeUpReturn(_, _)
                    if !row.paid =>
                {
                    return false;
                }
                CastCostClause::Behold(_)
                | CastCostClause::SneakReturn(_)
                | CastCostClause::WebSlingingReturn(_)
                | CastCostClause::BeamMeUpReturn(_, _) => {}
            }
        }
        let mut hand = 0usize;
        let mut battlefield = 0usize;
        let mut graveyard = 0usize;
        for r in &self.resources {
            match r.object.zone {
                Zone::Hand if r.object.owner == c.actor && self.reference(r.id) != c.source => {
                    hand += 1;
                }
                Zone::Battlefield => battlefield += 1,
                Zone::Graveyard => graveyard += 1,
                _ => {}
            }
        }
        // Mana payments may sacrifice battlefield objects into graveyards. We
        // count every object there even if it is an opponent's, a token, or not
        // actually sacrificable, and ignore all card-characteristic filters.
        let potential_graveyard = graveyard
            .saturating_add(battlefield)
            .saturating_add(self.active_melds().count());
        let possible = match spec.kind.payment_kind() {
            EffectChoiceKind::Discard
            | EffectChoiceKind::DiscardNamed(_)
            | EffectChoiceKind::DiscardMatchingLegendary
            | EffectChoiceKind::DiscardLegendary
            | EffectChoiceKind::ExileFromHand => hand > 0,
            EffectChoiceKind::DiscardOrLife(life) => hand > 0 || self.life >= i128::from(life),
            EffectChoiceKind::Behold
            | EffectChoiceKind::BeholdMany(_)
            | EffectChoiceKind::BeholdAndExile
            | EffectChoiceKind::OptionalBeholdType(_)
            | EffectChoiceKind::SacrificeOrDiscard { .. } => battlefield > 0 || hand > 0,
            EffectChoiceKind::Sacrifice
            | EffectChoiceKind::SacrificeAttachedEquipment
            | EffectChoiceKind::SacrificeAttachedPermanent
            | EffectChoiceKind::ReturnUnblockedAttacker
            | EffectChoiceKind::ReturnToHand => battlefield > 0,
            EffectChoiceKind::SacrificeRange { minimum: n }
            | EffectChoiceKind::SacrificeDifferentNames(n) => battlefield >= usize::from(n),
            EffectChoiceKind::SacrificePermanents(n) => {
                battlefield.saturating_add(self.game.state.exile_links.len()) >= n as usize
            }
            EffectChoiceKind::TapPermanents(n) => battlefield >= n as usize,
            EffectChoiceKind::ExileGraveyard => potential_graveyard > 0,
            EffectChoiceKind::ExileGraveyardCards(n)
            | EffectChoiceKind::ExileGraveyardRange { minimum: n } => {
                potential_graveyard >= usize::from(n)
            }
            // Any one potential card is treated as sufficient for a positive
            // weight. Its future face, colors and value need not be stable.
            EffectChoiceKind::CollectEvidence(n) => n == 0 || potential_graveyard > 0,
            EffectChoiceKind::ExileGraveyardSymbols { minimum: n, .. } => {
                n == 0 || potential_graveyard > 0
            }
            EffectChoiceKind::RemoveCreatureCounters(n) => n == 0 || battlefield > 0,
            EffectChoiceKind::RemovePermanentCounters { .. } => battlefield > 0,
            EffectChoiceKind::RemoveCreatureCounter(_) => battlefield > 0,
            EffectChoiceKind::Forage | EffectChoiceKind::ForageOrBlack => match c.forage {
                Some(ForageRoute::Food) => battlefield > 0,
                Some(ForageRoute::ExileThree) => potential_graveyard >= 3,
                Some(ForageRoute::Black) | None => true,
            },
            _ => true,
        };
        !possible
    }
    /// Only a fixed witness set permits rejecting an absent chosen subtype
    /// before mana search. This also handles mandatory single behold/exile costs.
    #[cfg(feature = "test-support")]
    fn unchanging_behold_witness_deficit(&self, c: &CostContext, spec: AdditionalCost) -> bool {
        let required = match spec.kind.payment_kind() {
            EffectChoiceKind::BeholdMany(amount) => usize::from(amount),
            EffectChoiceKind::Behold | EffectChoiceKind::BeholdAndExile => 1,
            _ => return false,
        };
        if required == 0
            || spec.filter.keywords != Keywords::NONE
            || c.clauses.iter().filter(|row| !row.paid).any(|row| {
                !matches!(
                    row.clause,
                    CastCostClause::Mana | CastCostClause::Additional | CastCostClause::Behold(_)
                )
            })
        {
            return false;
        }
        let candidates = self.resources.iter().filter(|resource| {
            let object = &resource.object;
            (object.zone == Zone::Battlefield && object.controller == c.actor
                || object.zone == Zone::Hand && object.owner == c.actor)
                && self.cost_object(spec, self.reference(resource.id), c.source)
        });
        if candidates.take(required).count() >= required {
            return false;
        }
        let state = &self.game.state;
        if self.continuation.is_some()
            || self.entry_projection.is_some()
            || self.controls().is_some()
            || state.temporary_controls.is_some()
            || self.copy_recipes.is_some()
            || self.temporary_copy_rows_state().is_some()
            || self.cost_group.is_some()
            || !self.sacrifice_group.is_empty()
            || self.required_face_up.is_some()
            || self.has_contributions()
            || !state.exile_links.is_empty()
            || state.borrowed_links.is_some()
            || state.borrowed_slots.is_some()
            || state.graveyard_borrowing.is_some()
            || state.self_granted_abilities.is_some()
            || state.temporary_abilities.is_some()
            || state.activation_observers.is_some()
            || !state.tap_sources.is_empty()
            || self.roles().is_some_and(|roles| {
                !roles.source_linked_effects.is_empty()
                    || !roles.resolved_ability_modifications.is_empty()
                    || !roles.ability_removal_sources.is_empty()
                    || !roles.creature_types.is_empty()
                    || !roles.basic_land_types.is_empty()
                    || !roles.animations.is_empty()
                    || !roles.live_static.is_empty()
            })
        {
            return false;
        }
        // Explicitly recognize read-only production. In particular, source
        // recoloring and future production families retain exact search.
        let read_only_output = |production: ManaProduction| {
            matches!(
                production.base(),
                ManaProduction::Fixed(_)
                    | ManaProduction::Choice { .. }
                    | ManaProduction::Combination { .. }
                    | ManaProduction::ColorsAmongControlledPermanents
                    | ManaProduction::MonocoloredPermanentColors
                    | ManaProduction::ChosenColor
                    | ManaProduction::ChosenColorAmount(_)
            )
        };
        for resource in &self.resources {
            let object = &resource.object;
            let card = object.card;
            // Command-zone static effects are outside this narrow battlefield
            // proof, including emblems with active characteristic providers.
            if object.zone == Zone::Command {
                return false;
            }
            // The characteristic-source index also contains color-only Auras.
            // Inspect their immutable fragments: AttachedChosenColor is stable
            // here, while AttachedCharacteristics still takes the fallback.
            if object.zone == Zone::Battlefield
                && (card.borrows_activations()
                    || card.graveyard_activation_subtype().is_some()
                    || card
                        .static_ability_groups()
                        .iter()
                        .flat_map(|g| g.fragments)
                        .any(|fragment| {
                            use crate::cards::StaticAbilityFragment::*;
                            match *fragment {
                                IntrinsicKeywords(_)
                                | Changeling
                                | AttachmentStats(_)
                                | AttachmentKeywords(_)
                                | AttachmentTrigger(_)
                                | DynamicStats(..)
                                | BaseStatSetter(..)
                                | CostReduction(..)
                                | TargetCostReduction(..)
                                | OpponentNoncreatureSpellTax(_)
                                | OpponentGraveyardOrExileSpellTax(_)
                                | SpellCastLimit(_)
                                | ChosenCardTypeSpellTax(_)
                                | AdditionalTriggers { .. }
                                | AttachedChosenColor
                                | BasicLandTapManaBonus(_)
                                | ControlledBasicLandManaMultiplier(_)
                                | ControlledLandTapManaBonus(_)
                                | ControlledCreatureTapManaBonus(_)
                                | ControlledLandColorlessTapManaBonus
                                | ControlledArtifactTokenTapManaBonus => false,
                                // This replacement changes only damage amounts;
                                // pure tap/AddMana payments never deal damage.
                                Replacement(
                                    _,
                                    crate::cards::ReplacementSpec::DoubleControlledChosenTypeDamage,
                                ) => false,
                                AttachmentMana(production)
                                | AttachmentManaBonus(production)
                                | LandMana { production, .. }
                                | OtherControlledPermanentTapMana(production) => {
                                    !read_only_output(production)
                                }
                                _ => true,
                            }
                        }))
            {
                return false;
            }
            let owner = if object.zone == Zone::Battlefield {
                object.controller
            } else {
                object.owner
            };
            if owner != c.actor {
                continue;
            }
            // Include temporarily unavailable printed routes. Pure taps cannot
            // change their zone, restore a removed ability, or acquire a new one.
            for (index, ability) in card.activations().iter().enumerate() {
                if let Some(production) = ability.mana_production()
                    && ability.source_zone() == object.zone
                    && (!Self::only_tap_mana(ability)
                        || !read_only_output(production)
                        || card.activation_put_counter(index as u16).is_some())
                {
                    return false;
                }
            }
            if object.zone == Zone::Battlefield
                && CharacteristicWorld::granted_mana(self, resource.id)
                    .iter()
                    .any(|grant| grant.sacrifice || !read_only_output(grant.production))
            {
                return false;
            }
        }
        // Behold does not require an untapped witness. Taps and AddMana cannot
        // introduce another matching hand/battlefield object in this class;
        // no unknown type, control, entry or departure provider was admitted.
        true
    }
    pub(super) fn set_cost_context(&mut self, a: Option<&Announcement>) {
        // Alternative spell costs are intentionally unset while choices are
        // enumerated. Only contributor activations need this separate receipt.
        self.contributors = a
            .filter(|a| {
                matches!(a.item.kind, StackKind::ActivatedAbility { .. })
                    && a.item.program().is_some_and(|(_, program)| {
                        matches!(
                            program,
                            [Instruction::Crew { .. }] | [Instruction::BecomeSaddled { .. }]
                        )
                    })
            })
            .and_then(|a| {
                self.game
                    .contributor_requirement(&self.game.funding_goal(a))
            });
        self.power_sensitive |= self.contributors.is_some();
        self.install_cost_context(a.and_then(CostContext::from_announcement));
        if let Some(source) = a.and_then(|a| a.item.source) {
            self.payment_source = Some(source.object);
            if a.is_some_and(|a| a.item.kind == StackKind::Spell) {
                self.project_cost_spell_source(source.object);
            }
        }
    }

    pub(super) fn set_payment_context(&mut self, a: &Announcement) {
        self.set_tap_payment(a.tap_payment());
        self.payment_source = a.item.source.map(|s| s.object);
        self.exhaust_source_reuse_authorized = a.exhaust_reuse_authorized;
        self.source_use_independent = a.item.self_granted_activation.is_some();
        if a.cast_costs.is_none() && a.additional_cost().is_some() {
            let mut shadow = a.clone();
            shadow.cast_costs = Some(Box::default());
            shadow.initialize_cast_costs(matches!(self.purpose, mana_lots::Purpose::Spell(spell) if convoke::cost_potential(self, spell.card)));
            self.set_cost_context(Some(&shadow));
        } else {
            self.set_cost_context(Some(a));
        }
    }
    fn install_cost_context(&mut self, context: Option<Box<CostContext>>) {
        if let Some(context) = &context {
            self.payment_source = Some(context.source);
        }
        self.cast_costs = context;
        if self.cast_costs.is_some() {
            self.power_sensitive = true;
            // All live candidates and witnesses must follow virtual departures.
            // Hidden hands remain internal to the acting player's cost search.
            for (id, o) in self.game.state.objects.iter().enumerate() {
                if !self.resources.iter().any(|r| r.id == id as CardId) {
                    self.resources.push(Resource {
                        id: id as CardId,
                        object: o.clone(),
                        attacking: self
                            .game
                            .state
                            .attackers
                            .iter()
                            .any(|r| r.card == id as CardId),
                    });
                }
            }
        }
        if let Some(source) = self.cast_costs.as_ref().map(|context| context.source) {
            self.project_cost_spell_source(source);
        }
    }
    fn project_cost_spell_source(&mut self, source: ObjectRef) {
        // CR601.2a precedes mana abilities and every cost payment. A borrowed
        // proposal has already left its origin, even before the real card has
        // moved. Excluding it only from Delve leaves it available to a mana
        // child's external graveyard cost. Keep the bound proposal reference;
        // this shadow move does not allocate a new live incarnation.
        if !matches!(self.purpose, mana_lots::Purpose::Spell(_))
            || self.game.state.objects.get(source.card as usize).is_none()
            || self.reference(source.card) != source
        {
            return;
        }
        let index = self
            .resources
            .iter()
            .position(|resource| resource.id == source.card)
            .unwrap_or_else(|| {
                let object = self.object(source.card).clone();
                self.resources.push(Resource {
                    id: source.card,
                    object,
                    attacking: false,
                });
                self.resources.len() - 1
            });
        let object = &mut self.resources[index].object;
        if matches!(
            object.zone,
            Zone::Hand | Zone::Graveyard | Zone::Exile | Zone::Library
        ) {
            if object.zone == Zone::Graveyard && object.is_card() {
                let count = &mut self.graveyard_left[object.owner.index()];
                *count = count
                    .checked_add(1)
                    .capacity("projected graveyard departure history exhausted");
            }
            object.zone = Zone::Stack;
            object.controller = self.actor;
        }
    }
    #[cfg(feature = "test-support")]
    pub(super) fn cost_object(
        &self,
        spec: AdditionalCost,
        r: ObjectRef,
        source: ObjectRef,
    ) -> bool {
        if spec.kind.chooses_cost_creature() {
            return additional_costs::chosen_cost_creature_matches(self, self.actor, r, spec.kind);
        }
        if evidence::threshold(spec).is_some() {
            return r != source
                && additional_costs::graveyard_exile_card_matches(self, self.actor, r, None);
        }
        if let EffectChoiceKind::ExileSourceAndNamed { name, .. } = spec.kind {
            return additional_costs::named_graveyard_exile_candidate(
                self, self.actor, source, r, name,
            );
        }
        let o = self.object(r.card);
        if spec.kind.payment_kind() == EffectChoiceKind::ExileFromHand && o.owner != self.actor {
            return false;
        }
        if spec.kind.is_attached_sacrifice() && o.attached_to != Some(source) {
            return false;
        }
        let craft = matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::ExileControlledOrGraveyard
                | EffectChoiceKind::ExileCraftNonlands(_)
                | EffectChoiceKind::ExileCraftMaterials(_)
                | EffectChoiceKind::ExileGraveyardRange { .. }
        );
        if matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::ExileCraftNonlands(_)
        ) && !additional_costs::has_craft_activated_ability(self, r.card)
        {
            return false;
        }
        if spec.kind.craft_materials().is_some()
            && (r == source || !craft::material_matches(self, spec.kind, r))
        {
            return false;
        }
        let spec = spec.in_zone(o.zone);
        if matches!(spec.kind.payment_kind(), EffectChoiceKind::BlightCost(amount) if amount > 0)
            && replacements::counters_prohibited(self, r)
        {
            return false;
        }
        (!craft || !CharacteristicWorld::graveyard_exile_prohibited(self, r.card))
            && (!matches!(
                spec.kind.payment_kind(),
                EffectChoiceKind::Sacrifice
                    | EffectChoiceKind::SacrificeRange { .. }
                    | EffectChoiceKind::SacrificeDifferentNames(_)
                    | EffectChoiceKind::SacrificePermanents(_)
                    | EffectChoiceKind::SacrificeAttachedEquipment
                    | EffectChoiceKind::SacrificeAttachedPermanent
            ) || !self.sacrifice_prohibited(r.card))
            && (spec.kind.payment_kind() != EffectChoiceKind::ReturnUnblockedAttacker
                || (self.game.ninjutsu_attacker(self.actor, r) && self.attacking(r.card)))
            && self.reference(r.card) == r
            && (r != source
                || !matches!(
                    spec.kind.payment_kind(),
                    EffectChoiceKind::Discard
                        | EffectChoiceKind::DiscardNamed(_)
                        | EffectChoiceKind::DiscardMatchingLegendary
                        | EffectChoiceKind::DiscardLegendary
                ) && !spec.filter.other)
            && (!spec.filter.source_only || r == source)
            && (spec.kind.payment_kind() != EffectChoiceKind::DiscardLegendary
                || self.legendary(r.card))
            && (spec.kind.payment_kind() != EffectChoiceKind::DiscardMatchingLegendary
                || additional_costs::legendary_name_match(self, self.actor, r.card))
            && Game::additional_object_matches(
                spec,
                self.actor,
                o,
                CharacteristicWorld::kinds(self, r.card),
                CharacteristicWorld::mana_value(self, r.card),
                self.keywords_for_cost(r.card),
                self.subtypes(r.card),
                self.name(r.card),
                || self.legendary(r.card),
                || self.suspected(r.card),
            )
    }
    #[cfg(feature = "test-support")]
    fn keywords_for_cost(&self, id: CardId) -> Keywords {
        self.resources
            .iter()
            .find(|r| r.id == id)
            .map_or(Keywords::NONE, |r| self.keywords(r))
    }
    #[cfg(feature = "test-support")]
    pub(super) fn cost_departures(
        &self,
        objects: &[ObjectRef],
        requested: Zone,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        record_payment_projection(1);
        if objects.iter().any(|&object| {
            self.reference(object.card) != object || self.object(object.card).zone == Zone::Outside
        }) {
            return false;
        }
        // Simultaneous departures all use characteristics from the same
        // pre-event world, even when one leaving object is a type provider.
        let before_facts: Vec<_> = objects
            .iter()
            .map(|r| {
                let object = self.object(r.card);
                let kinds = CharacteristicWorld::kinds(self, r.card);
                let graveyard_move =
                    object.zone == Zone::Battlefield && requested == Zone::Graveyard;
                let subtypes = graveyard_move.then(|| self.subtypes(r.card));
                let has_subtype = |subtype| {
                    subtypes
                        .as_ref()
                        .is_some_and(|types| types.has(self.definition(r.card), subtype))
                };
                (
                    object.zone,
                    object.controller,
                    object.owner,
                    kinds,
                    self.active_melds()
                        .find(|row| row.object == *r)
                        .map(|row| row.entry),
                    object.is_card(),
                    has_subtype("Food"),
                    graveyard_move && kinds & 1 != 0 && !has_subtype("Zombie"),
                )
            })
            .collect();
        let destinations = objects
            .iter()
            .zip(&before_facts)
            .map(|(r, &(_, _, _, kinds, _, _, _, _))| {
                let o = self.object(r.card);
                let prevented = self.zone_move_prevented(*r, requested);
                if requested != Zone::Graveyard {
                    let cannot_exile =
                        requested == Zone::Exile && self.graveyard_exile_prohibited(r.card);
                    return vec![((!prevented && !cannot_exile).then_some(requested), None)];
                }
                let own = o.card.zone_replacement();
                let shuffle = own == Some(ZoneReplacementRule::RevealShuffleSelf)
                    && (o.zone != Zone::Battlefield || self.printed_abilities_available(r.card));
                let exile = self.static_sources().iter().any(|&id| {
                    let p = self.object(id);
                    p.zone == Zone::Battlefield
                        && self.printed_abilities_available(id)
                        && match p.card.zone_replacement() {
                            Some(ZoneReplacementRule::ExileAllCardsAndTokens) => {
                                o.is_card() || o.is_token()
                            }
                            Some(ZoneReplacementRule::ExileControllerGraveyard) => {
                                (o.is_card() || o.is_token()) && p.controller == o.owner
                            }
                            Some(
                                rule @ (ZoneReplacementRule::ExileOpponentGraveyardCards
                                | ZoneReplacementRule::ExileUncontrolledOpponentGraveyardCardsLinked),
                            ) => zone_replacements::opponent_graveyard_card_matches(
                                rule, p.controller, o.owner, o.controller, o.zone, o.is_card(),
                            ),
                            Some(
                                ZoneReplacementRule::ExileOpposingCreatureDeath { .. }
                                | ZoneReplacementRule::ExileOpposingCreatureDeathPlain,
                            ) => {
                                o.zone == Zone::Battlefield
                                    && kinds & 1 != 0
                                    && p.controller != o.controller
                            }
                            Some(ZoneReplacementRule::ExileInstantSorceryCards) => {
                                kinds & 12 != 0 && o.is_card()
                            }
                            _ => false,
                        }
                }) || (o.zone == Zone::Battlefield
                    && (self.counter_count(r.card, CounterKind::Finality) > 0
                        || self.game.opponent_death_exile_applies(o.controller, kinds)
                        || self
                            .game
                            .state
                            .death_exile_replacements
                            .as_ref()
                            .is_some_and(|rules| {
                                rules.iter().any(|rule| {
                                    rule.object == *r && rule.expires_turn == self.game.state.turn
                                })
                            })));
                let mut choices = Vec::new();
                if shuffle {
                    choices.push((Some(Zone::Library), None));
                }
                if exile || self.game.turn_exiles_graveyard_card(o) {
                    choices.push((Some(Zone::Exile), None));
                }
                if o.zone == Zone::Battlefield && kinds & 1 != 0 && !o.is_token() {
                    for &id in self.static_sources() {
                        let provider = self.object(id);
                        let Some(ZoneReplacementRule::ExileOpposingNontokenCreatureDeathLinked {
                            life,
                        }) = provider.card.zone_replacement()
                        else {
                            continue;
                        };
                        if provider.zone == Zone::Battlefield
                            && provider.controller != o.controller
                            && self.printed_abilities_available(id)
                        {
                            let choice = (Some(Zone::Exile), Some((provider.controller, life)));
                            if !choices.contains(&choice) {
                                choices.push(choice);
                            }
                        }
                    }
                }
                if prevented {
                    choices.push((None, None));
                }
                if choices.is_empty() {
                    choices.push((Some(Zone::Graveyard), None));
                }
                choices
            })
            .collect::<Vec<_>>();
        let mut initial = self.clone();
        // These costs move one simultaneous group. Freeze all source facts
        // before any member or a layer provider leaves that world.
        for object in objects {
            initial.remember_payment_departure(object.card);
        }
        let mut worlds = vec![(initial, Vec::new())];
        for (
            (r, choices),
            &(before_zone, controller, owner, kinds, meld, was_card, food, non_zombie),
        ) in objects.iter().zip(destinations).zip(&before_facts)
        {
            let mut next = Vec::new();
            for (w, gains) in worlds {
                for &(zone, gain) in &choices {
                    let mut n = w.clone();
                    let Some(zone) = zone else {
                        // No departure happened; do not retain a newly captured
                        // departure witness for this prevented event.
                        if !self
                            .payment_departures
                            .iter()
                            .any(|(reference, _, _)| reference == r)
                        {
                            n.payment_departures
                                .retain(|(reference, _, _)| reference != r);
                        }
                        next.push((n, gains.clone()));
                        continue;
                    };
                    if before_zone == Zone::Battlefield && zone != Zone::Battlefield {
                        n.permanent_left_battlefield[controller.index()] = true;
                        n.void_active_this_turn |= kinds & 2 == 0;
                    }
                    if before_zone == Zone::Battlefield && zone == Zone::Graveyard && kinds & 1 != 0
                    {
                        n.controlled_creatures_died[controller.index()] = n
                            .controlled_creatures_died[controller.index()]
                        .checked_add(1)
                        .capacity("virtual controlled death history exhausted");
                        n.creatures_died = n
                            .creatures_died
                            .checked_add(1)
                            .capacity("virtual death history exhausted");
                        n.non_zombie_creature_died |= non_zombie;
                    }
                    if before_zone == Zone::Graveyard && zone != Zone::Graveyard && was_card {
                        let count = &mut n.graveyard_left[owner.index()];
                        *count = count
                            .checked_add(1)
                            .capacity("projected graveyard departure history exhausted");
                    }
                    if food && zone != Zone::Battlefield {
                        n.food_sacrificed[controller.index()] = true;
                    }
                    if before_zone == Zone::Battlefield && kinds & 16 != 0 {
                        n.zone_change_turn_history
                            .record_artifact_cost_departure(controller, requested, zone);
                    }
                    n.project_nonbattlefield_move(*r, zone, meld.map(|entry| entry.front_cards[0]));
                    if let Some(entry) = meld {
                        // The companion follows the chosen event. It does not
                        // receive another replacement, death, or life gain.
                        debug_assert_eq!(n.object(entry.companion.card).zone, Zone::Outside);
                        n.project_nonbattlefield_move(
                            entry.companion,
                            zone,
                            Some(entry.front_cards[1]),
                        );
                    }
                    let mut gains = gains.clone();
                    gains.extend(gain);
                    next.push((n, gains));
                }
            }
            worlds = next;
        }
        worlds.iter_mut().any(|(world, gains)| {
            // Gains use the post-departure world but retain the frozen recipients.
            world.refresh_control_projection();
            world.prune_source_linked_projection();
            world.refresh_enduring_projection();
            world.visit_death_exile_gains(gains, &mut |next| next.visit_linked_returns(emit))
        })
    }
    pub(super) fn project_nonbattlefield_move(
        &mut self,
        before: ObjectRef,
        zone: Zone,
        meld_front: Option<Card>,
    ) -> ObjectRef {
        debug_assert_eq!(self.reference(before.card), before);
        let index = self
            .resources
            .iter()
            .position(|row| row.id == before.card)
            .unwrap_or_else(|| {
                // An Outside meld component normally has no funding resource.
                let index = self.resources.len();
                self.resources.push(Resource {
                    id: before.card,
                    object: self.game.state.objects[before.card as usize].clone(),
                    attacking: false,
                });
                index
            });
        self.project_resource_nonbattlefield_move(index, before, zone, meld_front)
    }

    pub(super) fn project_resource_nonbattlefield_move(
        &mut self,
        index: usize,
        before: ObjectRef,
        zone: Zone,
        meld_front: Option<Card>,
    ) -> ObjectRef {
        let face_down_physical = self.projected_face_down_physical(before);
        let physical_copy = self
            .temporary_copy_rows_state()
            .as_ref()
            .and_then(|rows| rows.iter().find(|row| row.object == before))
            .map(|row| (row.physical_card, row.physical_kind));
        let resource = &mut self.resources[index];
        let object = &mut resource.object;
        let graveyard_arrival = object.zone != Zone::Graveyard && zone == Zone::Graveyard;
        object.zone = zone;
        object.generation = object
            .generation
            .checked_add(1)
            .capacity("virtual generation exhausted");
        // Departure facts retain the old face. The new object has its
        // physical identity even when the melded permanent was a copy.
        if let Some((card, kind)) = physical_copy {
            object.card = card;
            object.copy_kind = kind;
        } else {
            self.game.restore_projected_copy_identity(before, object);
        }
        if let Some(front) = meld_front {
            object.card = front;
        } else if let Some(physical) = face_down_physical {
            object.card = physical.front_face();
        } else if !object.is_token() && !object.is_spell_copy() {
            object.card = object.card.front_face();
        }
        object.counters = InitialCounters::default();
        object.growth = Stats::default();
        object.attached_to = None;
        object.attached_to_player = None;
        object.controller = object.owner;
        object.summoning_sick = false;
        object.tapped = false;
        object.damage = 0;
        object.damaged_this_turn = false;
        object.dealt_damage_this_turn = false;
        object.targeted_this_turn_by = 0;
        object.dealt_damage_this_incarnation = false;
        object.dealt_combat_damage_this_incarnation = false;
        object.deathtouch_since_sba = false;
        object.gained_keywords = Keywords::NONE;
        object.used_abilities = 0;
        object.loyalty_activated_this_turn = false;
        object.flashback = false;
        object.cast_for_warp = false;
        object.cast_for_impending = false;
        object.class_level = 1;
        resource.attacking = false;
        let after = ObjectRef {
            card: before.card,
            generation: object.generation,
        };
        if let Some(projection) = &mut self.entry_projection {
            projection
                .manifested
                .retain(|(source, _)| *source != before);
        }
        live_counters::set(&mut self.named_counters, before, None);
        self.project_copy_departure(before, after);
        if graveyard_arrival {
            let card = &self.resources[index].object;
            if card.is_card() && CharacteristicWorld::kinds(self, after.card) & 1 != 0 {
                let owner = card.owner;
                self.zone_change_turn_history
                    .record_creature_card_arrival(owner);
            }
        }
        after
    }
    #[cfg(feature = "test-support")]
    fn visit_death_exile_gains(
        &self,
        gains: &[(Player, u8)],
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let Some((&(player, base), later)) = gains.split_first() else {
            return emit(self);
        };
        life_gain::replacement_amounts(self, player, i128::from(base))
            .into_iter()
            .any(|amount| {
                let mut next = self.clone();
                if amount > 0 {
                    let life = if player == next.actor {
                        &mut next.life
                    } else {
                        &mut next.other_life
                    };
                    *life = life
                        .checked_add(amount)
                        .capacity("projected life total exhausted");
                    next.life_gained[player.index()] = true;
                    next.power_sensitive = true;
                }
                next.visit_death_exile_gains(later, emit)
            })
    }
    #[cfg(feature = "test-support")]
    pub(super) fn set_cost_counter_prefix(&mut self, selected: &[(ObjectRef, CounterKind)]) {
        if let Some(c) = &mut self.cast_costs {
            c.counters = selected.to_vec();
        }
    }
    /// Validate the supplied active clause or partial selection only. No
    /// resource is projected and no alternate future payment is explored.
    pub(in crate::engine) fn cost_selection_admissible(&self) -> bool {
        let Some(c) = &self.cast_costs else {
            return true;
        };
        let Some(active) = c.active else {
            return true;
        };
        let Some(row) = c.clauses.get(usize::from(active)).filter(|row| !row.paid) else {
            return false;
        };
        let selected_object = |spec: AdditionalCost, selected: Option<ObjectRef>| {
            selected.is_none_or(|object| {
                self.game
                    .additional_candidate_in_world(self, spec, c.actor, c.source, object, false)
            })
        };
        match row.clause {
            CastCostClause::Mana => true,
            CastCostClause::Life(amount) | CastCostClause::AlternativeLife(amount) => {
                amount == 0 || self.life >= i128::from(amount)
            }
            CastCostClause::CraftSource => {
                self.reference(c.source.card) == c.source
                    && self.object(c.source.card).zone == Zone::Battlefield
            }
            CastCostClause::Harmonize(object) => {
                harmonize::contribution(self, c.actor, object).is_some()
            }
            CastCostClause::Behold(_) => true,
            CastCostClause::OriginDiscard(selected) => {
                selected_object(equip_costs::discard_cost(), selected)
            }
            CastCostClause::SneakReturn(selected) => selected_object(sneak::COST, selected),
            CastCostClause::WebSlingingReturn(selected) => {
                selected_object(web_slinging::RETURN_COST, selected)
            }
            CastCostClause::BeamMeUpReturn(selected, provider) => selected.is_none_or(|object| {
                Some(object) != provider
                    && !self.beam_return_prohibited(object)
                    && selected_object(web_slinging::BEAM_RETURN_COST, Some(object))
            }),
            CastCostClause::Blight { selected, amount } => selected.is_none_or(|object| {
                self.reference(object.card) == object
                    && self.object(object.card).zone == Zone::Battlefield
                    && self.object(object.card).controller == c.actor
                    && CharacteristicWorld::kinds(self, object.card) & 1 != 0
                    && (amount == 0 || !replacements::counters_prohibited(self, object))
            }),
            CastCostClause::Casualty(instance) => {
                let declaration = &c.casualty[usize::from(instance)];
                declaration.sacrifice.is_none_or(|object| {
                    self.reference(object.card) == object
                        && self.object(object.card).zone == Zone::Battlefield
                        && self.object(object.card).controller == c.actor
                        && CharacteristicWorld::kinds(self, object.card) & 1 != 0
                        && self.power(object.card) >= i128::from(declaration.grant.minimum)
                        && !CharacteristicWorld::sacrifice_prohibited(self, object.card)
                })
            }
            CastCostClause::Conspire(instance) => c.conspire[usize::from(instance)]
                .objects
                .is_none_or(|objects| {
                    objects[0].card < objects[1].card
                        && objects.iter().all(|object| {
                            self.resources
                                .iter()
                                .position(|resource| {
                                    resource.id == object.card
                                        && resource.object.generation == object.generation
                                })
                                .is_some_and(|index| {
                                    self.conspire_candidate(
                                        c.actor,
                                        CharacteristicWorld::colors(self, c.source.card),
                                        index,
                                    )
                                })
                        })
                }),
            CastCostClause::Teamwork(minimum) => {
                let mut sum = saddle::PowerSum::default();
                c.teammates.iter().enumerate().all(|(index, &object)| {
                    if c.teammates[..index].contains(&object)
                        || self.reference(object.card) != object
                        || self.object(object.card).zone != Zone::Battlefield
                        || self.object(object.card).controller != c.actor
                        || self.object(object.card).tapped
                        || CharacteristicWorld::kinds(self, object.card) & 1 == 0
                    {
                        return false;
                    }
                    sum.add(self.power(object.card));
                    true
                }) && (!c.teamwork_exact || sum.enough(minimum))
            }
            CastCostClause::OriginCounters(amount) => self.local_counter_selection_valid(
                c,
                EffectChoiceKind::RemoveCreatureCounters(amount)
                    .counter_requirement()
                    .unwrap(),
            ),
            CastCostClause::OriginEvidence(threshold) => {
                self.local_evidence_selection_valid(c, threshold)
            }
            CastCostClause::OriginForage => {
                let Some(choice) = c.origin_forage.as_ref() else {
                    return true;
                };
                self.local_forage_selection_valid(
                    c,
                    choice.route,
                    &choice.selected,
                    choice.complete(),
                )
            }
            CastCostClause::Additional => {
                let Some(spec) = c.additional else {
                    return true;
                };
                if spec.kind == EffectChoiceKind::DiscardHand {
                    return true;
                }
                if let Some(amount) = spec.kind.additional_life_payment()
                    && (c.additional_life || matches!(spec.kind, EffectChoiceKind::PayLife(_)))
                {
                    return self.life >= i128::from(amount);
                }
                if let Some(requirement) = spec.kind.counter_requirement() {
                    return self.local_counter_selection_valid(c, requirement);
                }
                if let Some(threshold) = evidence::threshold(spec) {
                    return self.local_evidence_selection_valid(c, threshold);
                }
                if forage::is_forage_cost(spec) {
                    return self.local_forage_selection_valid(c, c.forage, &c.selected, c.exact);
                }
                let mut spec = spec;
                if spec.kind.additional_life_payment().is_some() {
                    spec.kind = EffectChoiceKind::Discard;
                }
                c.selected.iter().all(|object| {
                    !(c.source_tap_reserved
                        && *object == c.source
                        && matches!(spec.kind.payment_kind(), EffectChoiceKind::TapPermanents(_)))
                }) && self.game.additional_selection_valid_in_world(
                    self,
                    spec,
                    c.actor,
                    c.source,
                    &c.selected,
                    c.exact,
                )
            }
        }
    }
    pub(in crate::engine) fn cost_completion(&self) -> bool {
        self.cost_selection_admissible()
    }
    fn local_counter_selection_valid(
        &self,
        c: &CostContext,
        requirement: CounterCostRequirement,
    ) -> bool {
        c.counters.len() <= usize::from(requirement.amount)
            && c.counters
                .iter()
                .enumerate()
                .all(|(index, &(object, kind))| {
                    !requirement.kind.is_some_and(|expected| expected != kind)
                        && additional_costs::counter_cost_bearer(
                            self,
                            c.actor,
                            object,
                            requirement.bearer_kinds,
                        )
                        && (c.counters[..=index]
                            .iter()
                            .filter(|&&(r, k)| r == object && k == kind)
                            .count() as u64)
                            .checked_add(if object == c.source {
                                c.source_counter_reservation.get(kind)
                            } else {
                                0
                            })
                            .is_some_and(|needed| {
                                CharacteristicWorld::counter_count(self, object.card, kind)
                                    >= needed
                            })
                })
    }
    fn local_evidence_selection_valid(&self, c: &CostContext, threshold: u64) -> bool {
        c.selected.iter().enumerate().all(|(index, &object)| {
            !c.selected[..index].contains(&object)
                && !c.paid_evidence.contains(&object)
                && object != c.source
                && additional_costs::graveyard_exile_card_matches(self, c.actor, object, None)
        }) && (!c.exact
            || c.selected
                .iter()
                .map(|object| u128::from(self.object(object.card).card.fixed_mana_value()))
                .sum::<u128>()
                >= u128::from(threshold))
    }
    fn local_forage_selection_valid(
        &self,
        c: &CostContext,
        route: Option<ForageRoute>,
        selected: &[ObjectRef],
        exact: bool,
    ) -> bool {
        match route {
            None => !exact && selected.is_empty(),
            Some(ForageRoute::Black) => selected.is_empty(),
            Some(ForageRoute::Food) => self.game.additional_selection_valid_in_world(
                self,
                forage::food_cost(),
                c.actor,
                c.source,
                selected,
                exact,
            ),
            Some(ForageRoute::ExileThree) => {
                selected.len() <= 3
                    && (!exact || selected.len() == 3)
                    && selected.iter().enumerate().all(|(index, &object)| {
                        !selected[..index].contains(&object)
                            && object != c.source
                            && additional_costs::graveyard_exile_card_matches(
                                self, c.actor, object, None,
                            )
                    })
            }
        }
    }
    fn conspire_candidate(&self, actor: Player, colors: u8, index: usize) -> bool {
        let resource = &self.resources[index];
        resource.object.zone == Zone::Battlefield
            && resource.object.controller == actor
            && !resource.object.tapped
            && CharacteristicWorld::kinds(self, resource.id) & 1 != 0
            && CharacteristicWorld::colors(self, resource.id) & colors != 0
    }

    pub(in crate::engine) fn set_teamwork_prefix(&mut self, selected: &[ObjectRef], exact: bool) {
        let c = self.cast_costs.as_mut().expect("teamwork cost context");
        c.teammates = selected.to_vec();
        c.teamwork_exact = exact;
    }

    #[cfg(feature = "test-support")]
    pub(super) fn has_ordered_convoke(&self) -> bool {
        self.cast_costs.as_ref().is_some_and(|c| {
            c.clauses
                .iter()
                .any(|r| !r.paid && r.clause == CastCostClause::Mana)
        })
    }
    /// A deliberately narrow negative bound across future mana-source moves.
    /// Unknown providers and non-tap mutations retain the complete search.
    #[cfg(feature = "test-support")]
    pub(super) fn pure_tap_teamwork_deficit(&self, goal: &FundingGoal) -> bool {
        let Some(c) = &self.cast_costs else {
            return false;
        };
        let mut minimum = None;
        for row in c.clauses.iter().filter(|row| !row.paid) {
            match row.clause {
                CastCostClause::Teamwork(n) => minimum = Some(minimum.unwrap_or(0).max(n)),
                CastCostClause::Mana => (),
                _ => return false,
            }
        }
        let Some(minimum) = minimum else {
            return false;
        };
        let mut upper = saddle::PowerSum::default();
        for resource in &self.resources {
            let o = &resource.object;
            if o.zone == Zone::Battlefield
                && o.controller == c.actor
                && !o.tapped
                && CharacteristicWorld::kinds(self, resource.id) & 1 != 0
            {
                let power = self.power(resource.id);
                if power > 0 {
                    upper.add(power);
                }
            }
        }
        if upper.enough(minimum) {
            return false;
        }
        if goal.source.is_some()
            || goal.reserve.is_some()
            || self.continuation.is_some()
            || self.entry_projection.is_some()
            || self.controls().is_some()
            || self.payment_source.is_some()
            || self.cost_group.is_some()
            || !self.sacrifice_group.is_empty()
            || self.game.state.borrowed_links.is_some()
            || !self.game.state.exile_links.is_empty()
        {
            return false;
        }
        if self
            .roles()
            .is_some_and(|r| !r.base_stats.is_empty() || !r.live_static.is_empty())
        {
            return false;
        }
        // Inspect every resource and every immutable mana activation, including
        // hidden/nonbattlefield/unavailable routes. Pure taps cannot introduce
        // a new route, recipient, incarnation or counter configuration.
        for resource in &self.resources {
            let card = resource.object.card;
            if self
                .mana_activations(resource.id)
                .any(|(_, a, borrowed, acquired, granted)| {
                    borrowed.is_some()
                        || acquired.is_some()
                        || granted.is_some()
                        || !Self::only_tap_mana(a)
                })
                || CharacteristicWorld::granted_mana(self, resource.id)
                    .iter()
                    .any(|g| g.sacrifice)
            {
                return false;
            }
            if resource.object.zone != Zone::Battlefield {
                continue;
            }
            if card.dynamic_stats().is_some()
                || !card.scaling_modifiers().is_empty()
                || !card.static_modifiers().is_empty()
                || !card.enchanted_modifiers().is_empty()
                || card.base_stat_setter().is_some()
                || card
                    .static_ability_groups()
                    .iter()
                    .flat_map(|g| g.fragments)
                    .any(|f| {
                        !matches!(
                            f,
                            crate::cards::StaticAbilityFragment::IntrinsicKeywords(_)
                                | crate::cards::StaticAbilityFragment::Changeling
                                | crate::cards::StaticAbilityFragment::AttachmentStats(_)
                                | crate::cards::StaticAbilityFragment::AttachmentKeywords(_)
                                | crate::cards::StaticAbilityFragment::AttachmentTrigger(_)
                                | crate::cards::StaticAbilityFragment::SpellImprovise(_)
                                | crate::cards::StaticAbilityFragment::LandMana { .. }
                                | crate::cards::StaticAbilityFragment::ControlledPermanentTapMana { .. }
                                | crate::cards::StaticAbilityFragment::ControlledTokenTapMana { .. }
                                | crate::cards::StaticAbilityFragment::ControlledSubtypeTapMana { .. }
                                | StaticAbilityFragment::ControlledKindOrSubtypeTapMana { .. }
                        | crate::cards::StaticAbilityFragment::OtherControlledPermanentTapMana(_)
                                | crate::cards::StaticAbilityFragment::NoMaximumHandSize
                                | crate::cards::StaticAbilityFragment::MaximumHandSize(_)
                                | crate::cards::StaticAbilityFragment::SpendAnyManaForCreatures
                                | crate::cards::StaticAbilityFragment::SpendAnyColorForSpellSubtype(_)
                        )
                    })
            {
                return false;
            }
        }
        // Ordinary Teamwork keeps its mana in FundingGoal rather than an ordered
        // Mana row. Fixed attachment bonuses/keywords/trigger grants do not
        // change from pure tap/AddMana moves; no attachment can move here.
        // In this flat class base exceptions, fixed animation/base values,
        // counters, growth, choices and copy layers remain immutable. No mana
        // step changes turn/combat/attachment state. Further taps can only
        // remove a candidate from this exact positive-power upper sum.
        true
    }
    pub(super) fn only_tap_mana(a: &ActivatedAbility) -> bool {
        let c = a.cost;
        a.origin == ActivationOrigin::Battlefield
            && c.tap
            && !c.sacrifice
            && !c.exile_self
            && !c.discard
            && c.life == 0
            && c.remove.is_none()
            && c.loyalty.is_none()
            && c.additional.is_none()
            && !a.exhaust
            && !a.once_per_turn
            && matches!(a.program, [Instruction::AddMana(_)])
    }

    pub(super) fn mark_mana_clause_paid(&mut self) {
        if let Some(c) = &mut self.cast_costs {
            for row in &mut c.clauses {
                if row.clause == CastCostClause::Mana {
                    row.paid = true;
                }
            }
            c.active = None;
        }
    }
    #[cfg(feature = "test-support")]
    pub(in crate::engine) fn project_convoker(
        &mut self,
        object: ObjectRef,
        contribution: ConvokeContribution,
    ) -> bool {
        let Some(index) = self
            .resources
            .iter()
            .position(|r| r.id == object.card && r.object.generation == object.generation)
        else {
            return false;
        };
        let r = &self.resources[index];
        if r.object.zone != Zone::Battlefield
            || r.object.controller != self.actor
            || r.object.tapped
            || CharacteristicWorld::kinds(self, object.card) & 1 == 0
        {
            return false;
        }
        let colors = CharacteristicWorld::colors(self, object.card);
        let Some(c) = &mut self.cast_costs else {
            return false;
        };
        let Some(cost) = &mut c.mana else {
            return false;
        };
        if !contribution.apply(colors, cost) {
            return false;
        }
        self.resources[index].object.tapped = true;
        true
    }
    #[cfg(feature = "test-support")]
    pub(in crate::engine) fn project_improviser(&mut self, object: ObjectRef) -> bool {
        if !self.improvise_available() || !improvise::eligible(self, self.actor, object) {
            return false;
        }
        self.project_generic_tapper(object)
    }
    #[cfg(feature = "test-support")]
    pub(in crate::engine) fn project_substituter(&mut self, object: ObjectRef) -> bool {
        let Some(payment) = self.activation_tap_payment else {
            return false;
        };
        if !self.cast_costs.as_ref().is_some_and(|context| {
            (context.tap_payment_used as u64) < payment.cap()
                && payment.eligible(self, self.actor, object)
        }) || !self.project_generic_tapper(object)
        {
            return false;
        }
        self.cast_costs.as_mut().unwrap().tap_payment_used += 1;
        true
    }
    #[cfg(feature = "test-support")]
    fn project_generic_tapper(&mut self, object: ObjectRef) -> bool {
        let Some(index) = self
            .resources
            .iter()
            .position(|r| r.id == object.card && r.object.generation == object.generation)
        else {
            return false;
        };
        let Some(cost) = self.cast_costs.as_mut().and_then(|c| c.mana.as_mut()) else {
            return false;
        };
        let Some(generic) = cost.generic.checked_sub(1) else {
            return false;
        };
        cost.generic = generic;
        self.resources[index].object.tapped = true;
        true
    }
    #[cfg(feature = "test-support")]
    pub(in crate::engine) fn project_delver(&mut self, object: ObjectRef) -> bool {
        if !self.delve_available()
            || !delve::eligible(self, self.actor, object, self.payment_source)
        {
            return false;
        }
        let Some(context) = self.cast_costs.as_mut() else {
            return false;
        };
        let Some(cost) = context.mana.as_mut() else {
            return false;
        };
        let Some(remaining) = cost.generic.checked_sub(1) else {
            return false;
        };
        cost.generic = remaining;
        context.delve_selected = true;
        let mut projected = None;
        self.cost_departures(&[object], Zone::Exile, &mut |next| {
            projected = Some(next.clone());
            true
        });
        if let Some(next) = projected {
            *self = next;
            true
        } else {
            false
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(in crate::engine) fn teamwork_prefix_fundable(
        &self,
        selected: &[ObjectRef],
        exact: bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let mut world = self.cast_cost_world(a);
        world.set_teamwork_prefix(selected, exact);
        world.cost_completion()
    }
    pub(in crate::engine) fn blight_subject_fundable(&self, object: ObjectRef) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let mut world = self.cast_cost_world(a);
        let costs = world.cast_costs.as_mut().unwrap();
        let active = costs.active.unwrap();
        let row = &mut costs.clauses[usize::from(active)];
        let CastCostClause::Blight { amount, .. } = row.clause else {
            return false;
        };
        row.clause = CastCostClause::Blight {
            amount,
            selected: Some(object),
        };
        world.cost_completion()
    }
    pub(in crate::engine) fn cast_cost_world(&self, a: &Announcement) -> FundingState<'_> {
        let goal = FundingGoal {
            actor: a.item.controller,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut world = FundingState::from_game_with_convoke(
            self,
            &goal,
            convoke::potential(
                self,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
            ),
        );
        world.set_purpose(self.purpose_for_item(&a.item));
        world.set_tap_payment(a.tap_payment());
        world.set_cost_context(Some(a));
        world
    }
    pub(in crate::engine) fn visit_cast_cost_clauses(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let mut world = self.cast_cost_world(a);
        for (index, row) in a.cast_costs.as_ref().unwrap().clauses.iter().enumerate() {
            if row.paid {
                continue;
            }
            world.cast_costs.as_mut().unwrap().active = Some(index as u16);
            if world.cost_selection_admissible()
                && emit(Action::ChooseCastCost {
                    clause: index as u16,
                })
            {
                return true;
            }
        }
        false
    }
    pub(in crate::engine) fn visit_origin_discard_cost(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        self.state.objects.iter().enumerate().any(|(id, _)| {
            let object = self.object_ref(id as CardId);
            self.additional_candidate(
                equip_costs::discard_cost(),
                a.item.controller,
                a.item.source.unwrap().object,
                object,
            ) && emit(Action::ChooseAdditionalCost { object })
        })
    }

    pub(in crate::engine) fn visit_sneak_return_cost(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        self.state.attackers.iter().any(|attack| {
            let object = self.object_ref(attack.card);
            self.additional_candidate(
                sneak::COST,
                a.item.controller,
                a.item.source.unwrap().object,
                object,
            ) && emit(Action::PaySneakReturn { object })
        })
    }
    pub(in crate::engine) fn visit_web_slinging_return_cost(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        self.state.objects.iter().enumerate().any(|(id, _)| {
            let object = self.object_ref(id as CardId);
            self.additional_candidate(
                web_slinging::RETURN_COST,
                a.item.controller,
                a.item.source.unwrap().object,
                object,
            ) && emit(Action::PayWebSlingingReturn { object })
        })
    }
    pub(in crate::engine) fn visit_beam_me_up_return_cost(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let Some(CastCostClause::BeamMeUpReturn(_, provider)) =
            a.cast_costs.as_ref().and_then(|c| c.active_clause())
        else {
            return false;
        };
        let world = self.cast_cost_world(a);
        self.state.objects.iter().enumerate().any(|(id, _)| {
            let object = self.object_ref(id as CardId);
            Some(object) != provider
                && !world.beam_return_prohibited(object)
                && self.additional_candidate(
                    web_slinging::BEAM_RETURN_COST,
                    a.item.controller,
                    a.item.source.unwrap().object,
                    object,
                )
                && emit(Action::PayBeamMeUpReturn { object })
        })
    }
    pub(in crate::engine) fn visit_conspire_payments(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let Pending::ConspirePayment { instance, .. } = self.state.pending else {
            return false;
        };
        let a = self.state.announcing.as_ref().unwrap();
        let first = a.conspire.as_ref().unwrap()[usize::from(instance)].first_object;
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            if !self.conspire_object_available(object) || first == Some(object) {
                continue;
            }
            let action = if let Some(first) = first {
                let mut objects = [first, object];
                objects.sort_unstable_by_key(|object| object.card);
                if !self.conspire_payment_valid(objects) {
                    continue;
                }
                Action::PayConspire { objects }
            } else {
                Action::ChooseConspireObject { object }
            };
            if emit(action) {
                return true;
            }
        }
        false
    }
    pub(in crate::engine) fn visit_casualty_payments(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let Pending::CasualtyPayment { instance, .. } = self.state.pending else {
            return false;
        };
        let a = self.state.announcing.as_ref().unwrap();
        let minimum = a.casualty.as_ref().unwrap()[usize::from(instance)]
            .grant
            .minimum;
        self.state
            .objects
            .iter()
            .enumerate()
            .any(|(id, permanent)| {
                let id = id as CardId;
                permanent.zone == Zone::Battlefield
                    && permanent.controller == a.item.controller
                    && self.is_creature(id)
                    && self.stats(id).power >= i128::from(minimum)
                    && !CharacteristicWorld::sacrifice_prohibited(self, id)
                    && emit(Action::PayCasualty {
                        object: self.object_ref(id),
                    })
            })
    }

    pub(in crate::engine) fn cast_cost_group_fundable(
        &self,
        selected: &[ObjectRef],
        exact: bool,
    ) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        if a.cast_costs.as_ref().is_none_or(|c| c.active.is_none()) {
            return a.additional_cost().is_some_and(|spec| {
                self.additional_selection_valid(
                    spec,
                    a.item.controller,
                    a.item.source.unwrap().object,
                    selected,
                    exact,
                )
            });
        }
        let mut world = self.cast_cost_world(a);
        let c = world.cast_costs.as_mut().unwrap();
        if let Some(origin) = &mut c.origin_forage {
            origin.selected = selected.to_vec();
        } else {
            c.selected = selected.to_vec();
            c.exact = exact;
        }
        world.cost_selection_admissible()
    }

    pub(in crate::engine) fn conspire_intent_fundable(&self, _pay: bool) -> bool {
        self.state
            .announcing
            .as_ref()
            .and_then(|a| a.conspire.as_ref())
            .is_some_and(|rows| rows.iter().any(|row| !row.declared))
    }
    pub(in crate::engine) fn casualty_intent_fundable(&self, _pay: bool) -> bool {
        self.state
            .announcing
            .as_ref()
            .and_then(|a| a.casualty.as_ref())
            .is_some_and(|rows| rows.iter().any(|row| !row.declared))
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// Check supplied declarations while leaving unselected optional costs open.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::engine) fn preview_optional_cast_costs(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        source: ObjectRef,
        additional: Option<AdditionalCost>,
        route: Option<ForageRoute>,
        selected: &[ObjectRef],
        exact: bool,
        counters: &[(ObjectRef, CounterKind)],
        _extra: &[CastCostClause],
    ) -> Option<bool> {
        let mana_lots::Purpose::Spell(spell) = purpose else {
            return None;
        };
        let current = self.state.announcing.as_ref().filter(|a| {
            a.item.kind == StackKind::Spell
                && a.item.controller == goal.actor
                && a.item
                    .source
                    .is_some_and(|source| source.card == spell.card)
        });
        let additional = additional.and_then(|spec| {
            if matches!(spec.kind, EffectChoiceKind::OptionalBeholdType(_)) {
                current.and_then(Announcement::additional_cost)
            } else {
                Some(spec)
            }
        });
        // Unselected optional costs are declarations still owned by the caller.
        // Validate a supplied material prefix without choosing other declarations.
        if selected.is_empty() && counters.is_empty() && !exact {
            return Some(true);
        }
        let spec = additional?;
        let mut world = FundingState::from_game(self, &goal);
        world.set_purpose(purpose);
        world.install_cost_context(Some(Box::new(CostContext {
            actor: goal.actor,
            source,
            source_tap_reserved: false,
            source_counter_reservation: Counters::default(),
            clauses: vec![CastCostRow {
                clause: CastCostClause::Additional,
                paid: false,
            }],
            active: Some(0),
            casualty: Vec::new(),
            conspire: Vec::new(),
            additional: Some(spec),
            additional_life: current.is_some_and(|a| a.additional_life),
            forage: route,
            origin_forage: None,
            paid_evidence: Vec::new(),
            selected: selected.to_vec(),
            exact,
            counters: counters.to_vec(),
            #[cfg(feature = "test-support")]
            mana: None,
            teammates: Vec::new(),
            teamwork_exact: false,
            #[cfg(feature = "test-support")]
            delve_selected: false,
            #[cfg(feature = "test-support")]
            tap_payment_used: 0,
            mana_closed: current
                .and_then(|a| a.cast_costs.as_ref())
                .is_some_and(|c| c.mana_closed),
        })));
        Some(world.cost_selection_admissible())
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(in crate::engine) fn can_fund_symbol_cost_group(
        &self,
        goal: FundingGoal,
        _purpose: mana_lots::Purpose,
        spec: AdditionalCost,
        source: ObjectRef,
        selected: &[ObjectRef],
    ) -> bool {
        self.additional_selection_valid(spec, goal.actor, source, selected, false)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FundingState<'_> {
    /// Enumerate the exact pre-SBA counter worlds produced by paying a blight cost.
    #[cfg(feature = "test-support")]
    fn visit_blight_cost_projection(
        &self,
        subject: ObjectRef,
        amount: u32,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let Some(row) = self.resources.iter().find(|r| r.id == subject.card) else {
            return false;
        };
        if row.object.zone != Zone::Battlefield
            || row.object.generation != subject.generation
            || row.object.controller != self.actor
            || CharacteristicWorld::kinds(self, subject.card) & 1 == 0
        {
            return false;
        }
        if amount == 0 {
            return emit(self);
        }
        if replacements::counters_prohibited(self, subject) {
            return false;
        }
        let placement = replacements::CounterPlacement {
            placed_by: self.actor,
            object: subject,
            card: row.object.card,
            owner: row.object.owner,
            controller: row.object.controller,
            protector: CharacteristicWorld::battle_protector(self, subject),
            subtypes: self.subtypes(subject.card),
            kind: CharacteristicWorld::kinds(self, subject.card),
            power: characteristic_query::CharacteristicQuery::in_world(self)
                .stats(subject.card)
                .power,
            amount: Counters::of(CounterKind::MinusOne, u64::from(amount)),
            effect: false,
            entry_index: None,
            private: false,
        };
        replacements::counter_outcomes(
            self,
            &placement,
            &mut Vec::new(),
            &|placement| placement.clone(),
            &mut |placed| {
                let mut next = self.clone();
                let mut counters =
                    CharacteristicWorld::counter_amounts(&next, subject.card).to_owned();
                for kind in CounterKind::ALL {
                    counters.add(kind, placed.get(kind));
                }
                let resource = next
                    .resources
                    .iter_mut()
                    .find(|r| r.id == subject.card)
                    .unwrap();
                resource.object.counters = counters.hot();
                live_counters::set(&mut next.named_counters, subject, counters.named);
                emit(&next)
            },
        )
    }
}
