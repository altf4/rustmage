//! Completion search for declared nonmana clauses in the borrowed funding world.
use super::*;
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
    selected: Vec<ObjectRef>,
    exact: bool,
    counters: Vec<(ObjectRef, CounterKind)>,
    mana: Option<ManaCost>,
    teammates: Vec<ObjectRef>,
    teamwork_exact: bool,
    delve_selected: bool,
    /// Existing announcement phase, retained only in this private query context.
    mana_closed: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CostContext {
    fn from_announcement(a: &Announcement) -> Option<Box<Self>> {
        let c = a.cast_costs.as_ref()?;
        let mut selected = a.additional_group.as_deref().cloned().unwrap_or_default();
        if selected.is_empty() {
            selected.extend(a.additional);
        }
        let mut exact = a.additional.is_some() || a.additional_group_complete;
        if let Some(f) = &a.forage {
            selected = f.selected.clone();
            exact = f.complete();
        }
        if let Some(e) = &a.evidence {
            selected = e.selected.clone();
            exact = e.complete;
        }
        let mut source_tap_reserved = false;
        let mut source_counter_reservation = Counters::default();
        if let StackKind::ActivatedAbility { ability } = a.item.kind
            && a.activation_cost_route != ActivationCostRoute::FirstEquipZero
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
            delve_selected: a.convoke.as_ref().is_some_and(|f| !f.delved.is_empty()),
            mana_closed: c.mana_closed,
            source_tap_reserved,
            source_counter_reservation,
            teammates: c.teammates.clone(),
            teamwork_exact: false,
            actor: a.item.controller,
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
    pub(super) fn mana_window_open(&self, goal: &FundingGoal) -> bool {
        goal.requires_mana() && self.cast_costs.as_ref().is_none_or(|c| !c.mana_closed)
    }
    /// An ordered mana row has the same fixed requirement as the funding
    /// goal when the only other unpaid costs reveal existing witnesses.
    pub(super) fn only_mana_and_reveals(&self, goal: &FundingGoal) -> bool {
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
                        CastCostClause::Mana | CastCostClause::Behold(_) => true,
                        CastCostClause::Additional => c.additional.is_some_and(|spec| {
                            matches!(
                                spec.kind.payment_kind(),
                                EffectChoiceKind::Behold | EffectChoiceKind::BeholdMany(_)
                            )
                        }),
                        _ => false,
                    })
        })
    }
    /// Intrinsic land mana cannot repair a counter shortfall. Avoid enumerating
    /// equivalent land subsets while retaining the complete search for costs or
    /// mana abilities that can introduce counters, objects, or new abilities.
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
                CastCostClause::SneakReturn(_) | CastCostClause::WebSlingingReturn(_)
                    if !row.paid =>
                {
                    return false;
                }
                CastCostClause::Behold(_)
                | CastCostClause::SneakReturn(_)
                | CastCostClause::WebSlingingReturn(_) => {}
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
            | EffectChoiceKind::DiscardLegendary => hand > 0,
            EffectChoiceKind::DiscardOrLife(life) => hand > 0 || self.life >= i128::from(life),
            EffectChoiceKind::Behold
            | EffectChoiceKind::BeholdMany(_)
            | EffectChoiceKind::BeholdAndExile
            | EffectChoiceKind::OptionalBeholdType(_)
            | EffectChoiceKind::SacrificeOrDiscard { .. } => battlefield > 0 || hand > 0,
            EffectChoiceKind::Sacrifice
            | EffectChoiceKind::SacrificeAttachedEquipment
            | EffectChoiceKind::ReturnUnblockedAttacker
            | EffectChoiceKind::ReturnToHand => battlefield > 0,
            EffectChoiceKind::SacrificeRange { minimum: n }
            | EffectChoiceKind::SacrificeDifferentNames(n)
            | EffectChoiceKind::SacrificePermanents(n) => battlefield >= usize::from(n),
            EffectChoiceKind::TapPermanents(n) => battlefield >= n as usize,
            EffectChoiceKind::ExileGraveyard => potential_graveyard > 0,
            EffectChoiceKind::ExileGraveyardCards(n)
            | EffectChoiceKind::ExileGraveyardRange { minimum: n } => {
                potential_graveyard >= usize::from(n)
            }
            // Any one potential card is treated as sufficient for a positive
            // weight. Its future face, colors and value need not be stable.
            EffectChoiceKind::CollectEvidence(n)
            | EffectChoiceKind::ExileGraveyardSymbols { minimum: n, .. } => {
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
                                | ChosenCardTypeSpellTax(_)
                                | AdditionalTriggers { .. }
                                | AttachedChosenColor
                                | BasicLandTapManaBonus(_)
                                | ControlledLandTapManaBonus(_)
                                | ControlledLandColorlessTapManaBonus => false,
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
        if let Some(a) = a.filter(|a| a.cast_costs.is_some())
            && let Some(frame) = &a.convoke
        {
            // A contributor is reserved during selection and becomes tapped in
            // the speculative world. The live group commits only with payment.
            for selected in frame.objects() {
                if let Some(r) = self
                    .resources
                    .iter_mut()
                    .find(|r| r.id == selected.card && r.object.generation == selected.generation)
                {
                    r.object.tapped = true;
                }
            }
            if !frame.delved.is_empty() {
                let mut projected = None;
                self.cost_departures(&frame.delved, Zone::Exile, &mut |next| {
                    projected = Some(next.clone());
                    true
                });
                if let Some(next) = projected {
                    *self = next;
                }
            }
        }
    }
    pub(super) fn set_payment_context(&mut self, a: &Announcement) {
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
            if object.zone == Zone::Graveyard {
                self.graveyard_left[object.owner.index()] = true;
            }
            object.zone = Zone::Stack;
            object.controller = self.actor;
        }
    }
    pub(super) fn cost_object(
        &self,
        spec: AdditionalCost,
        r: ObjectRef,
        source: ObjectRef,
    ) -> bool {
        if spec.kind == EffectChoiceKind::ChooseControlledOrWarpedCreature {
            return additional_costs::chosen_cost_creature_matches(self, self.actor, r);
        }
        let o = self.object(r.card);
        if spec.kind.payment_kind() == EffectChoiceKind::SacrificeAttachedEquipment
            && o.attached_to != Some(source)
        {
            return false;
        }
        let craft = matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::ExileControlledOrGraveyard
                | EffectChoiceKind::ExileCraftNonlands(_)
                | EffectChoiceKind::ExileGraveyardRange { .. }
        );
        if matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::ExileCraftNonlands(_)
        ) && !additional_costs::has_craft_activated_ability(self, r.card)
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
            )
    }
    fn keywords_for_cost(&self, id: CardId) -> Keywords {
        self.resources
            .iter()
            .find(|r| r.id == id)
            .map_or(Keywords::NONE, |r| self.keywords(r))
    }
    pub(super) fn cost_departures(
        &self,
        objects: &[ObjectRef],
        requested: Zone,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
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
                (
                    object.zone,
                    object.controller,
                    object.owner,
                    CharacteristicWorld::kinds(self, r.card),
                    self.active_melds()
                        .find(|row| row.object == *r)
                        .map(|row| row.entry),
                )
            })
            .collect();
        let destinations = objects
            .iter()
            .zip(&before_facts)
            .map(|(r, &(_, _, _, kinds, _))| {
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
                if exile {
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
        for ((r, choices), &(before_zone, controller, owner, kinds, meld)) in
            objects.iter().zip(destinations).zip(&before_facts)
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
                    }
                    if before_zone == Zone::Graveyard {
                        n.graveyard_left[owner.index()] = true;
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
        let face_down_physical = self.projected_face_down_physical(before);
        let physical_copy = self
            .temporary_copy_rows_state()
            .as_ref()
            .and_then(|rows| rows.iter().find(|row| row.object == before))
            .map(|row| (row.physical_card, row.physical_kind));
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
        let resource = &mut self.resources[index];
        let object = &mut resource.object;
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
        object.controller = object.owner;
        object.summoning_sick = false;
        object.tapped = false;
        object.damage = 0;
        object.damaged_this_turn = false;
        object.dealt_damage_this_turn = false;
        object.dealt_damage_this_incarnation = false;
        object.dealt_combat_damage_this_incarnation = false;
        object.deathtouch_since_sba = false;
        object.gained_keywords = Keywords::NONE;
        object.used_abilities = 0;
        object.loyalty_activated_this_turn = false;
        object.flashback = false;
        object.cast_for_warp = false;
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
        after
    }
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
    pub(super) fn set_cost_counter_prefix(&mut self, selected: &[(ObjectRef, CounterKind)]) {
        if let Some(c) = &mut self.cast_costs {
            c.counters = selected.to_vec();
        }
    }
    pub(in crate::engine) fn cost_completion(&self) -> bool {
        let Some(context) = &self.cast_costs else {
            return true;
        };
        let unpaid = context
            .clauses
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.paid)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        self.complete_cost_rows(context, &unpaid)
    }
    fn conspire_candidate(&self, actor: Player, colors: u8, index: usize) -> bool {
        let resource = &self.resources[index];
        resource.object.zone == Zone::Battlefield
            && resource.object.controller == actor
            && !resource.object.tapped
            && CharacteristicWorld::kinds(self, resource.id) & 1 != 0
            && CharacteristicWorld::colors(self, resource.id) & colors != 0
    }
    fn complete_conspire_clause(
        &self,
        current: &CostContext,
        later: &CostContext,
        remaining: &[usize],
        instance: u16,
    ) -> bool {
        let colors = CharacteristicWorld::colors(self, current.source.card);
        let complete = |[first, second]: [usize; 2]| {
            let mut next = self.clone();
            next.resources[first].object.tapped = true;
            next.resources[second].object.tapped = true;
            next.complete_cost_rows(later, remaining)
        };
        if let Some(objects) = current.conspire[usize::from(instance)].objects {
            // A selected pair is one exact payment, not a prefix to enumerate.
            // Resource insertion order differs from public object/action order.
            if objects[0].card >= objects[1].card {
                return false;
            }
            let mut pair = [0; 2];
            for (index, object) in objects.iter().enumerate() {
                let Some(resource) = self
                    .resources
                    .iter()
                    .position(|r| r.id == object.card && r.object.generation == object.generation)
                else {
                    return false;
                };
                if !self.conspire_candidate(current.actor, colors, resource) {
                    return false;
                }
                pair[index] = resource;
            }
            return complete(pair);
        }
        // Future unpaid instances still explore distinct pairs in the current
        // projected world, after earlier mana and nonmana costs have changed it.
        let eligible: Vec<_> = (0..self.resources.len())
            .filter(|&index| self.conspire_candidate(current.actor, colors, index))
            .collect();
        for (at, &first) in eligible.iter().enumerate() {
            for &second in &eligible[at + 1..] {
                if complete([first, second]) {
                    return true;
                }
            }
        }
        false
    }
    fn complete_cost_rows(&self, c: &CostContext, remaining: &[usize]) -> bool {
        if remaining.is_empty() {
            return true;
        }
        // With only Teamwork and Mana left, these inner costs merely tap
        // contributors. They cannot add mana or restore/change a type, color,
        // provider or mana-type permission. The current Hall test is therefore
        // an upper bound before any teammates are spent. A successful bound
        // retains the complete ordered search and every live power evaluation.
        // Other costs may remove suppression or return providers, so they must
        // keep the general path even when mana is not currently payable.
        if remaining
            .iter()
            .any(|&i| matches!(c.clauses[i].clause, CastCostClause::Teamwork(_)))
            && remaining
                .iter()
                .any(|&i| c.clauses[i].clause == CastCostClause::Mana)
            && remaining.iter().all(|&i| {
                matches!(
                    c.clauses[i].clause,
                    CastCostClause::Teamwork(_) | CastCostClause::Mana
                )
            })
            && !self.mana_clause_payable(c)
        {
            return false;
        }
        for (position, &index) in remaining.iter().enumerate() {
            if c.active.is_some_and(|i| usize::from(i) != index) {
                continue;
            }
            let mut rest = remaining.to_vec();
            rest.remove(position);
            let mut later = c.clone();
            later.active = None;
            if matches!(
                c.clauses[index].clause,
                CastCostClause::Additional | CastCostClause::OriginCounters(_)
            ) {
                later.selected.clear();
                later.exact = false;
                later.counters.clear();
            }
            match c.clauses[index].clause {
                CastCostClause::OriginCounters(amount) => {
                    let requirement = EffectChoiceKind::RemoveCreatureCounters(amount)
                        .counter_requirement()
                        .unwrap();
                    if self.complete_counter_cost(c, &later, &rest, requirement) {
                        return true;
                    }
                }
                CastCostClause::Teamwork(minimum) => {
                    if self.complete_teamwork_clause(c, &later, &rest, minimum) {
                        return true;
                    }
                }
                CastCostClause::Blight { amount, selected } => {
                    for resource in &self.resources {
                        let reference = self.reference(resource.id);
                        if selected.is_some_and(|wanted| wanted != reference) {
                            continue;
                        }
                        if self.visit_blight_cost_projection(
                            reference,
                            u32::from(amount),
                            &mut |next| next.complete_cost_rows(&later, &rest),
                        ) {
                            return true;
                        }
                    }
                }
                CastCostClause::Mana => {
                    if self.complete_convoke_clause(c, &later, &rest) {
                        return true;
                    }
                }
                CastCostClause::Life(amount) | CastCostClause::AlternativeLife(amount) => {
                    if amount == 0 || self.life >= i128::from(amount) {
                        let mut next = self.clone();
                        next.pay_projected_life(next.actor, amount, &[]);
                        if next.complete_cost_rows(&later, &rest) {
                            return true;
                        }
                    }
                }
                CastCostClause::Behold(subtype) => {
                    if hand_choices::visit_behold_candidates(self, c.actor, subtype, &[], |_| true)
                        && self.complete_cost_rows(&later, &rest)
                    {
                        return true;
                    }
                }
                CastCostClause::Conspire(instance) => {
                    if self.complete_conspire_clause(c, &later, &rest, instance) {
                        return true;
                    }
                }
                CastCostClause::Casualty(instance) => {
                    let row = &c.casualty[usize::from(instance)];
                    for resource in &self.resources {
                        let r = self.reference(resource.id);
                        let o = &resource.object;
                        if row.sacrifice.is_some_and(|wanted| wanted != r)
                            || o.zone != Zone::Battlefield
                            || o.controller != c.actor
                            || CharacteristicWorld::kinds(self, resource.id) & 1 == 0
                            || self.power(resource.id) < i128::from(row.grant.minimum)
                        {
                            continue;
                        }
                        if self.cost_departures(&[r], Zone::Graveyard, &mut |w| {
                            w.complete_cost_rows(&later, &rest)
                        }) {
                            return true;
                        }
                    }
                }
                CastCostClause::Harmonize(object) => {
                    if harmonize::contribution(self, c.actor, object).is_some()
                        && let Some(index) = self.resources.iter().position(|r| {
                            r.id == object.card && r.object.generation == object.generation
                        })
                    {
                        let mut next = self.clone();
                        next.resources[index].object.tapped = true;
                        if next.complete_cost_rows(&later, &rest) {
                            return true;
                        }
                    }
                }
                CastCostClause::CraftSource => {
                    if self.reference(c.source.card) == c.source
                        && self.object(c.source.card).zone == Zone::Battlefield
                        && self.cost_departures(&[c.source], Zone::Exile, &mut |world| {
                            world.complete_cost_rows(&later, &rest)
                        })
                    {
                        return true;
                    }
                }
                CastCostClause::SneakReturn(selected) => {
                    for attack in &self.game.state.attackers {
                        let object = self.game.object_ref(attack.card);
                        if selected.is_none_or(|wanted| wanted == object)
                            && self.cost_object(sneak::COST, object, c.source)
                            && self.cost_departures(&[object], Zone::Hand, &mut |world| {
                                world.complete_cost_rows(&later, &rest)
                            })
                        {
                            return true;
                        }
                    }
                }
                CastCostClause::WebSlingingReturn(selected) => {
                    for resource in &self.resources {
                        let object = self.reference(resource.id);
                        if selected.is_none_or(|wanted| wanted == object)
                            && self.cost_object(web_slinging::RETURN_COST, object, c.source)
                            && (rest.is_empty()
                                || self.cost_departures(&[object], Zone::Hand, &mut |world| {
                                    world.complete_cost_rows(&later, &rest)
                                }))
                        {
                            return true;
                        }
                    }
                }
                CastCostClause::Additional => {
                    if self.complete_additional(c, &later, &rest) {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub(in crate::engine) fn set_teamwork_prefix(&mut self, selected: &[ObjectRef], exact: bool) {
        let c = self.cast_costs.as_mut().expect("teamwork cost context");
        c.teammates = selected.to_vec();
        c.teamwork_exact = exact;
    }
    fn complete_teamwork_clause(
        &self,
        c: &CostContext,
        later: &CostContext,
        rest: &[usize],
        minimum: u16,
    ) -> bool {
        let eligible = |r: ObjectRef| {
            let o = self.object(r.card);
            self.reference(r.card) == r
                && o.zone == Zone::Battlefield
                && o.controller == c.actor
                && !o.tapped
                && CharacteristicWorld::kinds(self, r.card) & 1 != 0
        };
        if !c.teammates.iter().all(|&r| eligible(r)) {
            return false;
        }
        let candidates: Vec<_> = self
            .resources
            .iter()
            .map(|r| self.reference(r.id))
            .filter(|&r| eligible(r))
            .collect();
        let mut selected = c.teammates.clone();
        self.complete_teamwork_choices(
            later,
            rest,
            minimum,
            &candidates,
            0,
            &mut selected,
            c.teamwork_exact,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn complete_teamwork_choices(
        &self,
        later: &CostContext,
        rest: &[usize],
        minimum: u16,
        candidates: &[ObjectRef],
        at: usize,
        selected: &mut Vec<ObjectRef>,
        exact: bool,
    ) -> bool {
        let mut sum = saddle::PowerSum::default();
        for r in selected.iter() {
            sum.add(self.power(r.card));
        }
        if sum.enough(minimum) {
            let mut next = self.clone();
            for r in selected.iter() {
                let i = next.resources.iter().position(|v| v.id == r.card).unwrap();
                next.resources[i].object.tapped = true;
            }
            if next.complete_cost_rows(later, rest) {
                return true;
            }
        }
        if exact {
            return false;
        }
        let mut maximum = sum;
        for r in &candidates[at..] {
            if !selected.contains(r) {
                maximum.add(self.power(r.card).max(0));
            }
        }
        if !maximum.enough(minimum) {
            return false;
        }
        for i in at..candidates.len() {
            let r = candidates[i];
            if selected.contains(&r) {
                continue;
            }
            selected.push(r);
            if self.complete_teamwork_choices(
                later,
                rest,
                minimum,
                candidates,
                i + 1,
                selected,
                false,
            ) {
                return true;
            }
            selected.pop();
        }
        false
    }
    pub(super) fn has_ordered_convoke(&self) -> bool {
        self.cast_costs.as_ref().is_some_and(|c| {
            c.clauses
                .iter()
                .any(|r| !r.paid && r.clause == CastCostClause::Mana)
        })
    }
    /// A deliberately narrow negative bound across future mana-source moves.
    /// Unknown providers and non-tap mutations retain the complete search.
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
                .any(|(_, a, borrowed, acquired)| {
                    borrowed.is_some() || acquired.is_some() || !Self::only_tap_mana(a)
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
                        | crate::cards::StaticAbilityFragment::OtherControlledPermanentTapMana(_)
                                | crate::cards::StaticAbilityFragment::NoMaximumHandSize
                                | crate::cards::StaticAbilityFragment::MaximumHandSize(_)
                                | crate::cards::StaticAbilityFragment::SpendAnyManaForCreatures
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
    fn mana_clause_payable(&self, c: &CostContext) -> bool {
        if c.delve_selected && !self.delve_available() {
            return false;
        }
        let Some(cost) = c.mana else {
            return false;
        };
        convoke::payable_in(
            self,
            c.actor,
            self.purpose,
            mana_lots::available(self.mana, &self.lots, self.purpose),
            cost.colored,
            cost.generic,
            &self.convoke_masks(),
        )
    }
    fn complete_convoke_clause(
        &self,
        c: &CostContext,
        later: &CostContext,
        rest: &[usize],
    ) -> bool {
        let Some(cost) = c.mana else {
            return false;
        };
        let pool = mana_lots::available(self.mana, &self.lots, self.purpose);
        // Fail before assigning individual helpers when the current mana clause
        // cannot be paid at all. If it is last, exact Hall matching is the whole
        // obligation; later nonmana costs still use the original assignment walk.
        if !self.mana_clause_payable(c) {
            return false;
        }
        if rest.is_empty() {
            return true;
        }
        let (required, generic) = cast_origins::payment_requirements(
            self,
            c.actor,
            self.purpose,
            cost.colored,
            cost.generic,
        );
        if payable(pool, required, generic) && self.complete_cost_rows(later, rest) {
            return true;
        }
        for resource in &self.resources {
            let object = self.reference(resource.id);
            if self.delve_available()
                && delve::eligible(self, c.actor, object, self.payment_source)
                && cost.generic > 0
            {
                let mut context = c.clone();
                context.mana.as_mut().unwrap().generic -= 1;
                context.delve_selected = true;
                if self.cost_departures(&[object], Zone::Exile, &mut |next| {
                    next.complete_convoke_clause(&context, later, rest)
                }) {
                    return true;
                }
            }
            if resource.object.zone != Zone::Battlefield
                || resource.object.controller != c.actor
                || resource.object.tapped
            {
                continue;
            }
            let colors = CharacteristicWorld::colors(self, resource.id);
            let kinds = CharacteristicWorld::kinds(self, resource.id);
            let choices: &[ConvokeContribution] = if self.convoke_available() && kinds & 1 != 0 {
                &convoke::contributions()
            } else if self.improvise_available() && kinds & 16 != 0 {
                &[ConvokeContribution::Generic]
            } else {
                continue;
            };
            for &contribution in choices {
                let mut remaining = cost;
                if !contribution.apply(colors, &mut remaining) {
                    continue;
                }
                let mut next = self.clone();
                let i = next
                    .resources
                    .iter()
                    .position(|r| r.id == resource.id)
                    .unwrap();
                next.resources[i].object.tapped = true;
                let mut context = c.clone();
                context.mana = Some(remaining);
                if next.complete_convoke_clause(&context, later, rest) {
                    return true;
                }
            }
        }
        false
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
    pub(in crate::engine) fn project_improviser(&mut self, object: ObjectRef) -> bool {
        if !self.improvise_available() || !improvise::eligible(self, self.actor, object) {
            return false;
        }
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
    fn complete_additional(
        &self,
        c: &CostContext,
        later: &CostContext,
        remaining: &[usize],
    ) -> bool {
        let Some(mut spec) = c.additional else {
            return true;
        };
        if spec.kind == EffectChoiceKind::DiscardHand {
            // Whole-hand costs select no individual resource and remain payable
            // after any legal mana children. Project their actual current group
            // before proving later ordered costs or an outer announcement.
            let hand = self
                .resources
                .iter()
                .filter_map(|resource| {
                    (resource.object.zone == Zone::Hand
                        && resource.object.owner == c.actor
                        && resource.object.is_card())
                    .then_some(self.reference(resource.id))
                })
                .collect::<Vec<_>>();
            return self.cost_departures(&hand, Zone::Graveyard, &mut |world| {
                world.complete_cost_rows(later, remaining)
            });
        }
        if let EffectChoiceKind::DiscardOrLife(life) = spec.kind.payment_kind() {
            if !c.exact && self.life >= i128::from(life) {
                let mut next = self.clone();
                next.pay_projected_life(next.actor, life, &[]);
                if next.complete_cost_rows(later, remaining) {
                    return true;
                }
            }
            if c.additional_life {
                return false;
            }
            spec.kind = EffectChoiceKind::Discard;
        }
        if let EffectChoiceKind::BlightCost(amount) = spec.kind.payment_kind() {
            return self.resources.iter().any(|resource| {
                let object = self.reference(resource.id);
                self.cost_object(spec, object, c.source)
                    && (!c.exact || c.selected == [object])
                    && (c.selected.is_empty() || c.selected == [object])
                    && self.visit_blight_cost_projection(object, amount, &mut |next| {
                        next.complete_cost_rows(later, remaining)
                    })
            });
        }
        let mut amount = 1usize;
        let mut destination = Zone::Graveyard;
        let mut evidence_min = None;
        let mut counters = None;
        let mut any_graveyard = false;
        match spec.kind.payment_kind() {
            EffectChoiceKind::SacrificeRange { minimum } => {
                amount = c.selected.len().max(usize::from(minimum));
            }
            EffectChoiceKind::ExileGraveyardRange { minimum } => {
                destination = Zone::Exile;
                amount = c.selected.len().max(usize::from(minimum));
            }
            EffectChoiceKind::Forage | EffectChoiceKind::ForageOrBlack => match c.forage {
                Some(ForageRoute::Black) => return self.complete_cost_rows(later, remaining),
                Some(ForageRoute::Food) => spec = forage::food_cost(),
                Some(ForageRoute::ExileThree) => {
                    amount = 3;
                    destination = Zone::Exile;
                    spec.kind = EffectChoiceKind::ExileGraveyard;
                    any_graveyard = true;
                }
                None => return false,
            },
            EffectChoiceKind::CollectEvidence(n) => {
                evidence_min = Some((u128::from(n), None));
                destination = Zone::Exile;
                spec.kind = EffectChoiceKind::ExileGraveyard;
                any_graveyard = true;
            }
            EffectChoiceKind::ExileGraveyardSymbols { color, minimum } => {
                evidence_min = Some((u128::from(minimum), Some(color)));
                destination = Zone::Exile;
            }
            EffectChoiceKind::RemoveCreatureCounters(_)
            | EffectChoiceKind::RemoveCreatureCounter(_)
            | EffectChoiceKind::RemovePermanentCounters { .. } => {
                counters = spec.kind.counter_requirement();
            }
            EffectChoiceKind::BeholdMany(n)
            | EffectChoiceKind::SacrificeDifferentNames(n)
            | EffectChoiceKind::SacrificePermanents(n) => amount = usize::from(n),
            EffectChoiceKind::TapPermanents(n) => amount = n as usize,
            EffectChoiceKind::ExileGraveyard
            | EffectChoiceKind::BeholdAndExile
            | EffectChoiceKind::ExileControlled
            | EffectChoiceKind::ExileControlledOrGraveyard => destination = Zone::Exile,
            EffectChoiceKind::ExileCraftNonlands(n) => {
                destination = Zone::Exile;
                amount = usize::from(n);
            }
            EffectChoiceKind::ExileGraveyardCards(n) => {
                destination = Zone::Exile;
                amount = usize::from(n);
                spec.kind = EffectChoiceKind::ExileGraveyard;
            }
            EffectChoiceKind::ReturnUnblockedAttacker | EffectChoiceKind::ReturnToHand => {
                destination = Zone::Hand
            }
            EffectChoiceKind::Behold
            | EffectChoiceKind::ChooseControlledOrWarpedCreature
            | EffectChoiceKind::Sacrifice
            | EffectChoiceKind::SacrificeAttachedEquipment
            | EffectChoiceKind::SacrificeOrDiscard { .. }
            | EffectChoiceKind::Discard
            | EffectChoiceKind::DiscardNamed(_)
            | EffectChoiceKind::DiscardMatchingLegendary
            | EffectChoiceKind::DiscardLegendary => {}
            _ => return false,
        }
        if let Some(requirement) = counters {
            return self.complete_counter_cost(c, later, remaining, requirement);
        }
        if any_graveyard {
            spec.filter = SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            };
        }
        let candidates = self
            .resources
            .iter()
            .filter_map(|v| {
                let r = self.reference(v.id);
                (self.cost_object(spec, r, c.source)
                    && !(c.source_tap_reserved
                        && r == c.source
                        && matches!(spec.kind.payment_kind(), EffectChoiceKind::TapPermanents(_))))
                .then_some(r)
            })
            .collect::<Vec<_>>();
        if evidence_min.is_some_and(|(n, color)| {
            candidates
                .iter()
                .map(|r| cost_weight(self.object(r.card).card, color))
                .sum::<u128>()
                < n
        }) {
            return false;
        }
        if !c.selected.iter().all(|r| candidates.contains(r)) {
            return false;
        }
        if remaining.is_empty()
            && matches!(spec.kind.payment_kind(), EffectChoiceKind::TapPermanents(_))
        {
            return c.selected.len() <= amount
                && (!c.exact || c.selected.len() == amount)
                && candidates.len() >= amount;
        }
        let mut selected = c.selected.clone();
        self.complete_object_choices(
            c,
            later,
            remaining,
            spec,
            destination,
            amount,
            evidence_min,
            &candidates,
            0,
            &mut selected,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn complete_object_choices(
        &self,
        c: &CostContext,
        later: &CostContext,
        remaining: &[usize],
        spec: AdditionalCost,
        destination: Zone,
        amount: usize,
        evidence_min: Option<(u128, Option<crate::cards::ChoiceColor>)>,
        candidates: &[ObjectRef],
        at: usize,
        selected: &mut Vec<ObjectRef>,
    ) -> bool {
        let enough = evidence_min.map_or(
            if matches!(
                spec.kind.payment_kind(),
                EffectChoiceKind::ExileCraftNonlands(_)
            ) {
                selected.len() >= amount
            } else {
                selected.len() == amount
            },
            |(minimum, color)| {
                selected
                    .iter()
                    .map(|r| cost_weight(self.object(r.card).card, color))
                    .sum::<u128>()
                    >= minimum
            },
        );
        if enough {
            if remaining.is_empty() {
                return true;
            }
            let success = if matches!(
                spec.kind.payment_kind(),
                EffectChoiceKind::Behold
                    | EffectChoiceKind::BeholdMany(_)
                    | EffectChoiceKind::ChooseControlledOrWarpedCreature
            ) {
                self.complete_cost_rows(later, remaining)
            } else if matches!(spec.kind.payment_kind(), EffectChoiceKind::TapPermanents(_)) {
                let mut next = self.clone();
                for r in selected.iter() {
                    let i = next.resources.iter().position(|v| v.id == r.card).unwrap();
                    next.resources[i].object.tapped = true;
                }
                next.complete_cost_rows(later, remaining)
            } else {
                self.cost_departures(selected, destination, &mut |w| {
                    w.complete_cost_rows(later, remaining)
                })
            };
            if success {
                return true;
            }
        }
        if c.exact
            || evidence_min.is_none()
                && selected.len() >= amount
                && !matches!(
                    spec.kind.payment_kind(),
                    EffectChoiceKind::ExileCraftNonlands(_)
                )
        {
            return false;
        }
        for i in at..candidates.len() {
            let r = candidates[i];
            if selected.contains(&r) {
                continue;
            }
            if matches!(
                spec.kind.payment_kind(),
                EffectChoiceKind::SacrificeDifferentNames(_)
            ) && selected
                .iter()
                .any(|s| crate::card_names::same_name(self.name(s.card), self.name(r.card)))
            {
                continue;
            }
            selected.push(r);
            if self.complete_object_choices(
                c,
                later,
                remaining,
                spec,
                destination,
                amount,
                evidence_min,
                candidates,
                i + 1,
                selected,
            ) {
                return true;
            }
            selected.pop();
        }
        false
    }
    fn complete_counter_cost(
        &self,
        c: &CostContext,
        later: &CostContext,
        remaining: &[usize],
        requirement: CounterCostRequirement,
    ) -> bool {
        let CounterCostRequirement {
            amount,
            kind: required_kind,
            bearer_kinds,
        } = requirement;
        let mut next = self.clone();
        for &(r, kind) in &c.counters {
            if required_kind.is_some_and(|expected| expected != kind)
                || !next.remove_cost_counter(r, kind, bearer_kinds)
            {
                return false;
            }
        }
        if c.counters.len() > usize::from(amount)
            || (!c.source_counter_reservation.is_empty()
                && !CharacteristicWorld::counter_amounts(&next, c.source.card)
                    .contains(&c.source_counter_reservation))
        {
            return false;
        }
        fn fill(
            w: &FundingState<'_>,
            left: usize,
            later: &CostContext,
            remaining: &[usize],
            first: usize,
            source_reservation: (ObjectRef, &Counters),
            requirement: CounterCostRequirement,
        ) -> bool {
            let (source, reservation) = source_reservation;
            if left == 0 {
                return w.complete_cost_rows(later, remaining);
            }
            for (index, resource) in w.resources.iter().enumerate().skip(first) {
                let object = w.reference(resource.id);
                if !additional_costs::counter_cost_bearer(
                    w,
                    w.actor,
                    object,
                    requirement.bearer_kinds,
                ) {
                    continue;
                }
                for kind in CounterKind::ALL {
                    if requirement.kind.is_some_and(|expected| expected != kind) {
                        continue;
                    }
                    let mut next = w.clone();
                    if !next.remove_cost_counter(object, kind, requirement.bearer_kinds) {
                        continue;
                    }
                    if !reservation.is_empty()
                        && !CharacteristicWorld::counter_amounts(&next, source.card)
                            .contains(reservation)
                    {
                        continue;
                    }
                    if fill(
                        &next,
                        left - 1,
                        later,
                        remaining,
                        index,
                        source_reservation,
                        requirement,
                    ) {
                        return true;
                    }
                }
            }
            false
        }
        fill(
            &next,
            usize::from(amount) - c.counters.len(),
            later,
            remaining,
            0,
            (c.source, &c.source_counter_reservation),
            requirement,
        )
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// Complete mana abilities before testing the orders of the remaining costs.
    pub(in crate::engine) fn preview_ordered_activation_cost(
        &self,
        goal: FundingGoal,
        spec: AdditionalCost,
        craft: bool,
    ) -> bool {
        let Some((source, _, _, _, _)) = goal.source.as_ref() else {
            return false;
        };
        let purpose = self.funding_purpose(&goal);
        let mut world = FundingState::from_game(self, &goal);
        world.set_purpose(purpose);
        let mana_closed = world.cast_costs.as_ref().is_some_and(|c| c.mana_closed);
        world.install_cost_context(Some(Box::new(CostContext {
            delve_selected: false,
            mana_closed,
            actor: goal.actor,
            source: *source,
            source_tap_reserved: goal.reserves_source_tap(*source),
            source_counter_reservation: Counters::default(),
            clauses: std::iter::once(CastCostClause::Mana)
                .chain(craft.then_some(CastCostClause::CraftSource))
                .chain(std::iter::once(CastCostClause::Additional))
                .map(|clause| CastCostRow {
                    clause,
                    paid: false,
                })
                .collect(),
            active: None,
            casualty: Vec::new(),
            conspire: Vec::new(),
            additional: Some(spec),
            additional_life: false,
            forage: None,
            selected: Vec::new(),
            exact: false,
            counters: Vec::new(),
            mana: Some(ManaCost {
                colored: goal.required,
                generic: goal.generic,
            }),
            teammates: Vec::new(),
            teamwork_exact: false,
        })));
        world.reachable(&goal, &mut HashSet::new())
    }
    pub(in crate::engine) fn teamwork_proposal_fundable(&self, a: &Announcement) -> bool {
        a.cost_options().any(|cost| {
            let mut choice = a.clone();
            choice.cost = Some(cost);
            let goal = self.funding_goal(&choice);
            let mut world = FundingState::from_game_with_convoke(
                self,
                &goal,
                convoke::potential(
                    self,
                    choice
                        .item
                        .source
                        .expect("cost announcement has object source")
                        .card,
                ),
            );
            world.set_purpose(self.purpose_for_item(&choice.item));
            world.set_cost_context(Some(&choice));
            world.reachable(&goal, &mut HashSet::new())
        })
    }
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
        world.set_cost_context(Some(a));
        world
    }
    pub(in crate::engine) fn visit_cast_cost_clauses(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let world = self.cast_cost_world(a);
        for (index, row) in a.cast_costs.as_ref().unwrap().clauses.iter().enumerate() {
            if row.paid {
                continue;
            }
            let mut choice = world.clone();
            choice.cast_costs.as_mut().unwrap().active = Some(index as u16);
            if choice.cost_completion()
                && emit(Action::ChooseCastCost {
                    clause: index as u16,
                })
            {
                return true;
            }
        }
        false
    }
    pub(in crate::engine) fn visit_sneak_return_cost(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let world = self.cast_cost_world(a);
        for attack in &self.state.attackers {
            let object = self.object_ref(attack.card);
            let mut choice = world.clone();
            let costs = choice.cast_costs.as_mut().unwrap();
            let active = usize::from(costs.active.unwrap());
            costs.clauses[active].clause = CastCostClause::SneakReturn(Some(object));
            if choice.cost_completion() && emit(Action::PaySneakReturn { object }) {
                return true;
            }
        }
        false
    }
    pub(in crate::engine) fn visit_web_slinging_return_cost(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let announcement = self.state.announcing.as_ref().unwrap();
        let world = self.cast_cost_world(announcement);
        for (id, permanent) in self.state.objects.iter().enumerate() {
            if permanent.zone != Zone::Battlefield || !permanent.tapped {
                continue;
            }
            let object = self.object_ref(id as CardId);
            let mut choice = world.clone();
            let costs = choice.cast_costs.as_mut().unwrap();
            let active = usize::from(costs.active.unwrap());
            costs.clauses[active].clause = CastCostClause::WebSlingingReturn(Some(object));
            if choice.cost_completion() && emit(Action::PayWebSlingingReturn { object }) {
                return true;
            }
        }
        false
    }
    pub(in crate::engine) fn visit_conspire_payments(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let Pending::ConspirePayment { instance, .. } = self.state.pending else {
            return false;
        };
        let a = self.state.announcing.as_ref().unwrap();
        let world = self.cast_cost_world(a);
        let colors = CharacteristicWorld::colors(&world, a.item.source.unwrap().id);
        let mut candidates: Vec<_> = (0..world.resources.len())
            .filter(|&index| world.conspire_candidate(a.item.controller, colors, index))
            .map(|index| world.reference(world.resources[index].id))
            .collect();
        candidates.sort_unstable_by_key(|object| object.card);
        for (at, &first) in candidates.iter().enumerate() {
            for &second in &candidates[at + 1..] {
                let objects = [first, second];
                let mut choice = world.clone();
                choice.cast_costs.as_mut().unwrap().conspire[usize::from(instance)].objects =
                    Some(objects);
                if choice.cost_completion() && emit(Action::PayConspire { objects }) {
                    return true;
                }
            }
        }
        false
    }
    pub(in crate::engine) fn visit_casualty_payments(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let instance = match self.state.pending {
            Pending::CasualtyPayment { instance, .. } => instance,
            _ => return false,
        };
        let a = self.state.announcing.as_ref().unwrap();
        let world = self.cast_cost_world(a);
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            let mut choice = world.clone();
            choice.cast_costs.as_mut().unwrap().casualty[usize::from(instance)].sacrifice =
                Some(object);
            if choice.cost_completion() && emit(Action::PayCasualty { object }) {
                return true;
            }
        }
        false
    }
    pub(in crate::engine) fn cast_cost_object_fundable(&self, object: ObjectRef) -> bool {
        let Some(a) = self
            .state
            .announcing
            .as_ref()
            .filter(|a| a.cast_costs.as_ref().is_some_and(|c| c.mana_closed))
        else {
            return true;
        };
        let mut world = self.cast_cost_world(a);
        let c = world.cast_costs.as_mut().unwrap();
        c.selected = vec![object];
        c.exact = true;
        world.cost_completion()
    }
    pub(in crate::engine) fn cast_cost_group_fundable(
        &self,
        selected: &[ObjectRef],
        exact: bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let mut world = self.cast_cost_world(a);
        let c = world.cast_costs.as_mut().unwrap();
        c.selected = selected.to_vec();
        c.exact = exact;
        world.cost_completion()
    }
    pub(in crate::engine) fn cast_cost_counters_fundable(
        &self,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let mut world = self.cast_cost_world(a);
        world.cast_costs.as_mut().unwrap().counters = selected.to_vec();
        world.cost_completion()
    }
    pub(in crate::engine) fn conspire_intent_fundable(&self, pay: bool) -> bool {
        let mut a = self.state.announcing.as_ref().unwrap().clone();
        let row = a
            .conspire
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|r| !r.declared)
            .unwrap();
        row.pay = pay;
        row.declared = true;
        a.initialize_cast_costs(convoke::cost_potential(
            self,
            a.item
                .source
                .expect("cost announcement has object source")
                .card,
        ));
        if a.cast_costs.is_none() {
            a.cast_costs = Some(Box::default());
            a.initialize_cast_costs(convoke::cost_potential(
                self,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
            ));
        }
        if a.item.cast.is_some_and(|c| c.method.is_harmonize()) {
            return self.harmonize_choice_fundable(&a, None)
                || (0..self.state.objects.len()).any(|id| {
                    self.harmonize_choice_fundable(&a, Some(self.object_ref(id as CardId)))
                });
        }
        a.cost
            .into_iter()
            .chain(a.cost_options().filter(|_| a.cost.is_none()))
            .any(|cost| {
                let mut choice = a.clone();
                choice.cost = Some(cost);
                let goal = self.funding_goal(&choice);
                let mut world = FundingState::from_game_with_convoke(
                    self,
                    &goal,
                    convoke::potential(
                        self,
                        choice
                            .item
                            .source
                            .expect("cost announcement has object source")
                            .card,
                    ),
                );
                world.set_purpose(self.purpose_for_item(&choice.item));
                world.set_cost_context(Some(&choice));
                world.reachable(&goal, &mut HashSet::new())
            })
    }
    pub(in crate::engine) fn casualty_intent_fundable(&self, pay: bool) -> bool {
        let mut a = self.state.announcing.as_ref().unwrap().clone();
        let row = a
            .casualty
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|r| !r.declared)
            .unwrap();
        row.pay = pay;
        row.declared = true;
        a.initialize_cast_costs(convoke::cost_potential(
            self,
            a.item
                .source
                .expect("cost announcement has object source")
                .card,
        ));
        if a.cast_costs.is_none() {
            a.cast_costs = Some(Box::default());
            a.initialize_cast_costs(convoke::cost_potential(
                self,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .card,
            ));
        }
        if a.item.cast.is_some_and(|c| c.method.is_harmonize()) {
            return self.harmonize_choice_fundable(&a, None)
                || (0..self.state.objects.len()).any(|id| {
                    self.harmonize_choice_fundable(&a, Some(self.object_ref(id as CardId)))
                });
        }
        a.cost_options().any(|cost| {
            let mut choice = a.clone();
            choice.cost = Some(cost);
            let goal = self.funding_goal(&choice);
            let mut world = FundingState::from_game_with_convoke(
                self,
                &goal,
                convoke::potential(
                    self,
                    choice
                        .item
                        .source
                        .expect("cost announcement has object source")
                        .card,
                ),
            );
            world.set_purpose(self.purpose_for_item(&choice.item));
            world.set_cost_context(Some(&choice));
            world.reachable(&goal, &mut HashSet::new())
        })
    }
    /// One resource solver handles mana and ordered additional life costs, including previews before announcement.
    pub(in crate::engine) fn can_fund_life_cost(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        life: u32,
    ) -> bool {
        let mut world = FundingState::from_game(self, &goal);
        world.set_purpose(purpose);
        if world.cast_costs.is_none() {
            world.install_cost_context(Some(Box::new(CostContext {
                delve_selected: false,
                mana_closed: false,
                teammates: Vec::new(),
                teamwork_exact: false,
                actor: goal.actor,
                mana: None,
                source_tap_reserved: false,
                source_counter_reservation: Counters::default(),
                source: ObjectRef {
                    card: 0,
                    generation: 0,
                },
                clauses: Vec::new(),
                active: None,
                casualty: Vec::new(),
                conspire: Vec::new(),
                additional: None,
                additional_life: false,
                forage: None,
                selected: Vec::new(),
                exact: false,
                counters: Vec::new(),
            })));
        }
        let c = world.cast_costs.as_mut().unwrap();
        if !c
            .clauses
            .iter()
            .any(|r| matches!(r.clause, CastCostClause::Life(_)))
        {
            c.clauses.push(CastCostRow {
                clause: CastCostClause::Life(life),
                paid: false,
            });
        }
        world.reachable(&goal, &mut HashSet::new())
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// Preview declarations that are still optional; fixed declarations stay fixed.
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
        extra: &[CastCostClause],
    ) -> Option<bool> {
        let mana_lots::Purpose::Spell(spell) = purpose else {
            return None;
        };
        let current = self.state.announcing.as_ref().filter(|a| {
            a.item.kind == StackKind::Spell
                && a.item.controller == goal.actor
                && a.item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    == spell.card
        });
        let additional = additional.and_then(|spec| {
            if matches!(spec.kind, EffectChoiceKind::OptionalBeholdType(_)) {
                current.and_then(Announcement::additional_cost)
            } else {
                Some(spec)
            }
        });
        let ordered_additional = additional.is_some_and(|s| {
            matches!(
                s.kind,
                EffectChoiceKind::BeholdMany(_)
                    | EffectChoiceKind::BeholdAndExile
                    | EffectChoiceKind::ChooseControlledOrWarpedCreature
            )
        }) || delve::potential(self)
            || additional.is_some_and(|a| {
                matches!(
                    a.kind.payment_kind(),
                    EffectChoiceKind::DiscardOrLife(_) | EffectChoiceKind::BlightCost(_)
                )
            })
            || additional.is_some()
                && (convoke::potential(self, spell.card)
                    || improvise::potential(self, spell.card)
                    || delve::potential(self));
        if !ordered_additional
            && extra.is_empty()
            && current.is_none_or(|a| {
                a.casualty.is_none() && a.conspire.is_none() && a.cast_costs.is_none()
            })
            && !self.has_casualty(goal.actor, spell.card)
        {
            return None;
        }
        let grants = current
            .and_then(|a| a.casualty.as_deref())
            .cloned()
            .unwrap_or_else(|| self.casualty_grants(goal.actor, spell.card));
        if !ordered_additional
            && grants.is_empty()
            && extra.is_empty()
            && current.is_none_or(|a| a.cast_costs.is_none())
        {
            return None;
        }
        fn declarations(
            rows: &mut [CasualtyDeclaration],
            index: usize,
            visit: &mut impl FnMut(&[CasualtyDeclaration]) -> bool,
        ) -> bool {
            if index == rows.len() {
                return visit(rows);
            }
            if rows[index].declared {
                return declarations(rows, index + 1, visit);
            }
            rows[index].declared = true;
            for pay in [false, true] {
                rows[index].pay = pay;
                if declarations(rows, index + 1, visit) {
                    return true;
                }
            }
            rows[index].declared = false;
            false
        }
        let mut grants = grants;
        let routes = if additional.is_some_and(forage::is_forage_cost) && route.is_none() {
            let mut v = vec![Some(ForageRoute::ExileThree), Some(ForageRoute::Food)];
            if additional.is_some_and(|s| s.kind == EffectChoiceKind::ForageOrBlack) {
                v.push(Some(ForageRoute::Black));
            }
            v
        } else {
            vec![route]
        };
        Some(declarations(&mut grants, 0, &mut |casualty| {
            routes.iter().any(|&chosen_route| {
                let mut attempt = goal.clone();
                if chosen_route == Some(ForageRoute::Black) && route.is_none() {
                    attempt.required[2] = attempt.required[2]
                        .checked_add(1)
                        .capacity("forage mana exhausted");
                }
                let mut clauses = casualty
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| r.pay && !r.paid)
                    .map(|(i, _)| CastCostRow {
                        clause: CastCostClause::Casualty(i as u16),
                        paid: false,
                    })
                    .collect::<Vec<_>>();
                if let Some(rows) = current.and_then(|a| a.conspire.as_deref()) {
                    clauses.extend(
                        rows.iter()
                            .enumerate()
                            .filter(|(_, r)| r.pay && !r.paid)
                            .map(|(i, _)| CastCostRow {
                                clause: CastCostClause::Conspire(i as u16),
                                paid: false,
                            }),
                    );
                    if rows.iter().any(|r| r.pay) {
                        clauses.push(CastCostRow {
                            clause: CastCostClause::Mana,
                            paid: false,
                        });
                    }
                }
                if let Some(a) = current
                    && a.item.cast.is_some_and(|c| c.teamwork == Some(true))
                {
                    clauses.push(CastCostRow {
                        clause: CastCostClause::Teamwork(spell.card.teamwork().unwrap().minimum),
                        paid: false,
                    });
                }
                if additional.is_some() {
                    clauses.push(CastCostRow {
                        clause: CastCostClause::Additional,
                        paid: false,
                    });
                }
                if additional.is_some_and(|s| {
                    matches!(
                        s.kind,
                        EffectChoiceKind::BeholdMany(_)
                            | EffectChoiceKind::BeholdAndExile
                            | EffectChoiceKind::ChooseControlledOrWarpedCreature
                    )
                }) || convoke::potential(self, spell.card)
                    || improvise::potential(self, spell.card)
                    || delve::potential(self)
                {
                    clauses.push(CastCostRow {
                        clause: CastCostClause::Mana,
                        paid: false,
                    });
                }
                for &clause in extra {
                    if !clauses.iter().any(|r| r.clause == clause) {
                        clauses.push(CastCostRow {
                            clause,
                            paid: false,
                        });
                    }
                }
                let context = CostContext {
                    delve_selected: false,
                    mana_closed: current
                        .and_then(|a| a.cast_costs.as_ref())
                        .is_some_and(|c| c.mana_closed),
                    teammates: current
                        .and_then(|a| a.cast_costs.as_ref())
                        .map_or_else(Vec::new, |c| c.teammates.clone()),
                    teamwork_exact: false,
                    actor: goal.actor,
                    mana: (additional.is_some_and(|s| {
                        matches!(
                            s.kind,
                            EffectChoiceKind::BeholdMany(_)
                                | EffectChoiceKind::BeholdAndExile
                                | EffectChoiceKind::ChooseControlledOrWarpedCreature
                        )
                    }) || current.is_some_and(|a| {
                        a.conspire
                            .as_deref()
                            .is_some_and(|rows| rows.iter().any(|r| r.pay))
                    }) || convoke::potential(self, spell.card)
                        || improvise::potential(self, spell.card)
                        || delve::potential(self))
                    .then_some(ManaCost {
                        colored: attempt.required,
                        generic: attempt.generic,
                    }),
                    source,
                    source_tap_reserved: false,
                    source_counter_reservation: Counters::default(),
                    clauses,
                    active: None,
                    casualty: casualty.to_vec(),
                    conspire: current
                        .and_then(|a| a.conspire.as_deref())
                        .cloned()
                        .unwrap_or_default(),
                    additional,
                    additional_life: current.is_some_and(|a| a.additional_life),
                    forage: chosen_route,
                    selected: selected.to_vec(),
                    exact,
                    counters: counters.to_vec(),
                };
                let mut world = FundingState::from_game_with_convoke(
                    self,
                    &attempt,
                    convoke::potential(self, spell.card),
                );
                world.set_purpose(purpose);
                world.install_cost_context(Some(Box::new(context)));
                world.reachable(&attempt, &mut HashSet::new())
            })
        }))
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn cost_weight(card: Card, color: Option<crate::cards::ChoiceColor>) -> u128 {
    u128::from(color.map_or_else(
        || card.fixed_mana_value(),
        |color| card.mana_symbols_of_color(color),
    ))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(in crate::engine) fn can_fund_symbol_cost_group(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        spec: AdditionalCost,
        source: ObjectRef,
        selected: &[ObjectRef],
    ) -> bool {
        let mut world = FundingState::from_game(self, &goal);
        world.set_purpose(purpose);
        let mana_closed = world.cast_costs.as_ref().is_some_and(|c| c.mana_closed);
        world.install_cost_context(Some(Box::new(CostContext {
            delve_selected: false,
            mana_closed,
            actor: goal.actor,
            source,
            source_tap_reserved: goal.reserves_source_tap(source),
            source_counter_reservation: Counters::default(),
            clauses: vec![
                CastCostRow {
                    clause: CastCostClause::Additional,
                    paid: false,
                },
                CastCostRow {
                    clause: CastCostClause::Mana,
                    paid: false,
                },
            ],
            active: None,
            casualty: Vec::new(),
            conspire: Vec::new(),
            additional: Some(spec),
            additional_life: false,
            forage: None,
            selected: selected.to_vec(),
            exact: false,
            counters: Vec::new(),
            mana: Some(ManaCost {
                colored: goal.required,
                generic: goal.generic,
            }),
            teammates: Vec::new(),
            teamwork_exact: false,
        })));
        world.reachable(&goal, &mut HashSet::new())
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FundingState<'_> {
    /// Enumerate the exact pre-SBA counter worlds produced by paying a blight cost.
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
