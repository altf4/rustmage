//! Owned mandatory spell and activation costs, chosen after targets and paid before cast completion.
use super::*;

/// A craft material is tested in its current zone, including abilities gained
/// or removed on the battlefield. Activation permission is not relevant.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn has_craft_activated_ability<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> bool {
    let object = world.object(id);
    let printed = object.zone != Zone::Battlefield || world.printed_abilities_available(id);
    let card = world.definition(id);
    let layered_grant = object.zone == Zone::Battlefield
        && ability_evaluation::evaluate_if_needed(world).is_some_and(|evaluation| {
            let current = &evaluation.result.objects[id as usize];
            current.grants.iter().any(|grant| match grant.kind {
                ability_order::GrantedAbilityKind::Template(index) => {
                    grant.granting_card.grants_activation(index)
                }
                _ => false,
            }) || world
                .game()
                .state
                .self_granted_abilities
                .as_ref()
                .is_some_and(|rows| {
                    rows.iter().any(|row| {
                        row.recipient == world.reference(id)
                            && matches!(row.kind, SelfGrantedAbilityKind::Activated { .. })
                            && current.self_grants.contains(&row.timestamp)
                    })
                })
        });
    (printed
        && (card.mana_color().is_some()
            || card.activations().iter().enumerate().any(|(index, _)| {
                !card.self_granted_activation_template(index as u16)
                    && (!card.grants_activation(index as u16)
                        || world.source_activation_template_available(id, index as u16))
                    && !card.legacy_subtype_mana_activation(index as u16)
            })))
        || (object.zone == Zone::Battlefield
            && (layered_grant
                || !world.granted_mana(id).is_empty()
                || !borrowed_activations::abilities(world, id).is_empty()))
}

/// Evaluate the complete cost in the actual or projected characteristic world.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn legendary_name_match<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    actor: Player,
    card: CardId,
) -> bool {
    world.legendary(card)
        && (0..world.game().state.objects.len()).any(|i| {
            let id = i as CardId;
            let o = world.object(id);
            o.zone == Zone::Battlefield
                && o.controller == actor
                && world.legendary(id)
                && crate::card_names::same_name(world.name(card), world.name(id))
        })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct AdditionalCostView {
    pub kind: EffectChoiceKind,
    pub candidates: Vec<ObjectRef>,
    pub selected: Option<ObjectRef>,
    pub selected_group: Vec<ObjectRef>,
    pub creature_counter_payments: Vec<(ObjectRef, CounterKind)>,
    pub paid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_finish_group: Option<bool>,
}
/// A tap-symbol cost and a separate tap-permanents cost cannot both tap the
/// same source. Without the tap symbol the source is an ordinary candidate.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn tap_cost_reserves_source(goal: &FundingGoal, object: ObjectRef) -> bool {
    goal.source
        .as_ref()
        .is_some_and(|(source, ability, _, definition, _)| {
            *source == object && definition.activation(*ability).cost.tap
        })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn craft_material_selection_complete(&self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        let Some(AdditionalCost {
            kind: EffectChoiceKind::ExileCraftNonlands(minimum),
            ..
        }) = a.additional_cost()
        else {
            return false;
        };
        let selected = a.additional_group.as_deref().map_or(&[][..], Vec::as_slice);
        selected.len() >= usize::from(minimum) && self.cast_cost_group_fundable(selected, true)
    }
    pub(super) fn additional_candidate(
        &self,
        spec: AdditionalCost,
        actor: Player,
        source: ObjectRef,
        object: ObjectRef,
    ) -> bool {
        self.additional_candidate_inner(spec, actor, source, object, false)
    }
    /// A mana ability may remove a current graveyard-exile prohibition before
    /// this nonmana cost is actually paid. Funding proves that continuation.
    pub(super) fn additional_candidate_for_funding(
        &self,
        spec: AdditionalCost,
        actor: Player,
        source: ObjectRef,
        object: ObjectRef,
    ) -> bool {
        self.additional_candidate_inner(spec, actor, source, object, true)
    }
    fn additional_candidate_inner(
        &self,
        spec: AdditionalCost,
        actor: Player,
        source: ObjectRef,
        object: ObjectRef,
        projected: bool,
    ) -> bool {
        let Some(o) = self.state.objects.get(object.card as usize) else {
            return false;
        };
        if spec.kind.payment_kind() == EffectChoiceKind::SacrificeAttachedEquipment
            && o.attached_to != Some(source)
        {
            return false;
        }
        if !projected
            && matches!(
                spec.kind.payment_kind(),
                EffectChoiceKind::ExileControlledOrGraveyard
                    | EffectChoiceKind::ExileCraftNonlands(_)
                    | EffectChoiceKind::ExileGraveyardRange { .. }
            )
            && ability_world::CharacteristicWorld::graveyard_exile_prohibited(self, object.card)
        {
            return false;
        }
        if matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::ExileCraftNonlands(_)
        ) && !has_craft_activated_ability(self, object.card)
        {
            return false;
        }
        let spec = spec.in_zone(o.zone);
        if matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::Sacrifice
                | EffectChoiceKind::SacrificeRange { .. }
                | EffectChoiceKind::SacrificeDifferentNames(_)
                | EffectChoiceKind::SacrificeAttachedEquipment
        ) && ability_world::CharacteristicWorld::sacrifice_prohibited(self, object.card)
        {
            return false;
        }
        let controls_cost_object = match spec.kind.payment_kind() {
            EffectChoiceKind::Behold => match o.zone {
                Zone::Battlefield => o.controller == actor,
                Zone::Hand => o.owner == actor,
                _ => false,
            },
            EffectChoiceKind::Discard
            | EffectChoiceKind::DiscardNamed(_)
            | EffectChoiceKind::DiscardMatchingLegendary
            | EffectChoiceKind::DiscardLegendary
            | EffectChoiceKind::ExileGraveyard
            | EffectChoiceKind::ExileGraveyardCards(_)
            | EffectChoiceKind::ExileGraveyardRange { .. }
            | EffectChoiceKind::ExileGraveyardSymbols { .. } => o.owner == actor,
            EffectChoiceKind::ExileControlled
            | EffectChoiceKind::Sacrifice
            | EffectChoiceKind::SacrificeRange { .. }
            | EffectChoiceKind::SacrificeDifferentNames(_)
            | EffectChoiceKind::ReturnUnblockedAttacker
            | EffectChoiceKind::ReturnToHand
            | EffectChoiceKind::TapPermanents(_) => o.controller == actor,
            _ => false,
        };
        if !controls_cost_object
            || self.object_ref(object.card) != object
            || object == source
                && (matches!(
                    spec.kind.payment_kind(),
                    EffectChoiceKind::Discard
                        | EffectChoiceKind::DiscardNamed(_)
                        | EffectChoiceKind::DiscardMatchingLegendary
                        | EffectChoiceKind::DiscardLegendary
                ) || spec.filter.other)
            || spec.filter.source_only && object != source
        {
            return false;
        }
        if spec.kind.payment_kind() == EffectChoiceKind::DiscardLegendary
            && !ability_world::CharacteristicWorld::legendary(self, object.card)
        {
            return false;
        }
        if spec.kind.payment_kind() == EffectChoiceKind::DiscardMatchingLegendary
            && !legendary_name_match(self, actor, object.card)
        {
            return false;
        }
        if spec.kind.payment_kind() == EffectChoiceKind::ReturnUnblockedAttacker
            && !self.ninjutsu_attacker(actor, object)
        {
            return false;
        }
        Self::additional_object_matches(
            spec,
            actor,
            o,
            self.current_kinds(object.card),
            self.keywords(object.card),
            self.granted_subtypes(object.card),
            self.object_name(object.card),
        )
    }
    pub(super) fn additional_object_matches(
        spec: AdditionalCost,
        actor: Player,
        o: &Object,
        kinds: u16,
        keywords: Keywords,
        subtypes: GrantedSubtypes,
        name: &str,
    ) -> bool {
        let spec = spec.in_zone(o.zone);
        if !spec.kind.cost_matches_any(o.card, kinds, subtypes) {
            return false;
        }
        if matches!(o.zone, Zone::Hand | Zone::Graveyard) && !o.is_card() {
            return false;
        }
        let zone = match spec.kind.payment_kind() {
            EffectChoiceKind::Behold if matches!(o.zone, Zone::Battlefield | Zone::Hand) => o.zone,
            EffectChoiceKind::ExileGraveyard
            | EffectChoiceKind::ExileGraveyardCards(_)
            | EffectChoiceKind::ExileGraveyardRange { .. }
            | EffectChoiceKind::ExileGraveyardSymbols { .. } => Zone::Graveyard,
            EffectChoiceKind::Discard
            | EffectChoiceKind::DiscardNamed(_)
            | EffectChoiceKind::DiscardMatchingLegendary
            | EffectChoiceKind::DiscardLegendary => Zone::Hand,
            EffectChoiceKind::ExileControlled
            | EffectChoiceKind::Sacrifice
            | EffectChoiceKind::SacrificeRange { .. }
            | EffectChoiceKind::SacrificeDifferentNames(_)
            | EffectChoiceKind::ReturnUnblockedAttacker
            | EffectChoiceKind::ReturnToHand
            | EffectChoiceKind::TapPermanents(_) => Zone::Battlefield,
            _ => return false,
        };
        if let EffectChoiceKind::DiscardNamed(required) = spec.kind.payment_kind()
            && !crate::card_names::same_name(name, required)
        {
            return false;
        }
        if let EffectChoiceKind::ExileGraveyardSymbols { color, .. } = spec.kind.payment_kind()
            && o.card.colors() & (1 << color.index()) == 0
        {
            return false;
        }
        if spec.kind.payment_kind() == EffectChoiceKind::Behold
            && zone == Zone::Hand
            && !o.is_card()
        {
            return false;
        }
        let f = spec.filter;
        if matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::ExileGraveyard
                | EffectChoiceKind::ExileGraveyardCards(_)
                | EffectChoiceKind::ExileGraveyardRange { .. }
                | EffectChoiceKind::ExileGraveyardSymbols { .. }
        ) && !o.is_card()
        {
            return false;
        }
        if matches!(spec.kind.payment_kind(), EffectChoiceKind::TapPermanents(_)) && o.tapped {
            return false;
        }
        (if zone == Zone::Battlefield {
            o.controller
        } else {
            o.owner
        }) == actor
            && o.zone == zone
            && (!f.artifact_enchantment_or_token || kinds & 48 != 0 || o.is_token())
            && (f.kinds == 0 || kinds & u16::from(f.kinds) != 0)
            && (!f.nonland || kinds & 2 == 0)
            && keywords.contains(f.keywords)
            && f.tokens.is_none_or(|token| o.is_token() == token)
            && f.subtype
                .is_none_or(|subtype| subtypes.has(o.card, subtype))
    }

    pub(super) fn spell_can_fund(&self, actor: Player, card: CardId, cost: ManaCost) -> bool {
        self.spell_can_fund_for_definition(actor, card, cost, self.effective_spell_card(card))
    }
    pub(super) fn spell_can_fund_for_definition(
        &self,
        actor: Player,
        card: CardId,
        cost: ManaCost,
        definition: Card,
    ) -> bool {
        let skips_additional = definition.mana_alternative_replaces_additional(cost);
        let Some(cost) = self.spell_total_for_definition(actor, card, cost, definition) else {
            return false;
        };
        if skips_additional {
            return self.can_fund_for(
                FundingGoal {
                    actor,
                    required: cost.colored,
                    generic: cost.generic,
                    source: None,
                    reserve: None,
                },
                self.spell_mana_purpose_for_card(actor, card, definition),
            );
        }
        self.spell_can_fund_total_for_definition(actor, card, cost, None, definition)
    }
    pub(super) fn spell_can_fund_total(
        &self,
        actor: Player,
        card: CardId,
        cost: ManaCost,
        route: Option<ForageRoute>,
    ) -> bool {
        self.spell_can_fund_total_for_definition(
            actor,
            card,
            cost,
            route,
            self.effective_spell_card(card),
        )
    }
    pub(super) fn spell_can_fund_total_for_definition(
        &self,
        actor: Player,
        card: CardId,
        cost: ManaCost,
        route: Option<ForageRoute>,
        definition: Card,
    ) -> bool {
        let purpose = self.spell_mana_purpose_for_card(actor, card, definition);
        let base = FundingGoal {
            actor,
            required: cost.colored,
            generic: cost.generic,
            source: None,
            reserve: None,
        };
        if let Some(result) = self.preview_optional_cast_costs(
            base.clone(),
            purpose,
            self.object_ref(card),
            definition.additional_cost(),
            route,
            &[],
            false,
            &[],
            &[],
        ) {
            return result;
        }
        if definition.has_convoke() {
            return self.can_fund_convoke_for(base, purpose);
        }
        if improvise::potential(self, definition) {
            return self.can_fund_contributions_for(base, purpose, false);
        }
        let Some(spec) = definition.additional_cost() else {
            return self.can_fund_for(base, purpose);
        };
        if spec.kind.variable_cost_minimum().is_some() {
            return self.variable_cost_group_fundable(
                base,
                purpose,
                spec,
                self.object_ref(card),
                &[],
                false,
            );
        }
        if matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::TapPermanents(_)
                | EffectChoiceKind::ExileGraveyardCards(_)
                | EffectChoiceKind::ExileGraveyardRange { .. }
                | EffectChoiceKind::ExileGraveyardSymbols { .. }
        ) {
            return self.group_cost_fundable(base, purpose, spec, self.object_ref(card), &[]);
        }
        if forage::is_forage_cost(spec) {
            return if route == Some(ForageRoute::Black) {
                self.can_fund_for(base, purpose)
            } else {
                self.forage_cost_fundable_for(base, spec, route, purpose)
            };
        }
        self.state.objects.iter().enumerate().any(|(id, _)| {
            let object = self.object_ref(id as CardId);
            self.additional_candidate(spec, actor, self.object_ref(card), object)
                && self.can_fund_for(
                    FundingGoal {
                        reserve: Some((object, spec)),
                        ..base.clone()
                    },
                    purpose,
                )
        })
    }
    pub(super) fn group_cost_fundable(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        spec: AdditionalCost,
        source: ObjectRef,
        chosen: &[ObjectRef],
    ) -> bool {
        if matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::ExileGraveyardSymbols { .. }
        ) {
            return self.can_fund_symbol_cost_group(goal, purpose, spec, source, chosen);
        }
        let amount = match spec.kind.payment_kind() {
            EffectChoiceKind::TapPermanents(n) | EffectChoiceKind::ExileGraveyardCards(n) => n,
            _ => unreachable!("group cost"),
        };
        let Some(needed) = usize::from(amount).checked_sub(chosen.len()) else {
            return false;
        };
        // This fallback enumerates current candidates, not future projected
        // objects. Rule out an undersized candidate set before funding search.
        let eligible = |r: ObjectRef| {
            !chosen.contains(&r)
                && self.additional_candidate(spec, goal.actor, source, r)
                && (!matches!(spec.kind.payment_kind(), EffectChoiceKind::TapPermanents(_))
                    || !tap_cost_reserves_source(&goal, r))
        };
        if needed > 0
            && (0..self.state.objects.len())
                .map(|id| self.object_ref(id as CardId))
                .filter(|&r| eligible(r))
                .take(needed)
                .count()
                < needed
        {
            return false;
        }
        if !self.can_fund_for(goal.clone(), purpose) {
            return false;
        }
        self.group_cost_fundable_selected(goal, purpose, spec, source, chosen, amount)
    }

    fn group_cost_fundable_selected(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        spec: AdditionalCost,
        source: ObjectRef,
        chosen: &[ObjectRef],
        amount: u16,
    ) -> bool {
        if chosen.len() == usize::from(amount) {
            return self.can_fund_cost_group(goal, purpose, spec, chosen);
        }
        (0..self.state.objects.len()).any(|id| {
            let r = self.object_ref(id as CardId);
            if chosen.contains(&r)
                || !self.additional_candidate(spec, goal.actor, source, r)
                || (matches!(spec.kind.payment_kind(), EffectChoiceKind::TapPermanents(_))
                    && tap_cost_reserves_source(&goal, r))
            {
                return false;
            }
            let mut next = chosen.to_vec();
            next.push(r);
            self.group_cost_fundable_selected(goal.clone(), purpose, spec, source, &next, amount)
        })
    }
    pub(super) fn different_name_sacrifice_fundable(
        &self,
        goal: FundingGoal,
        spec: AdditionalCost,
        amount: u16,
        chosen: &[ObjectRef],
    ) -> bool {
        if chosen.len() == usize::from(amount) {
            return self.can_fund_sacrifice_group(goal, spec, chosen);
        }
        let source = goal.source.as_ref().expect("activation source").0;
        (0..self.state.objects.len()).any(|id| {
            let object = self.object_ref(id as CardId);
            if chosen.contains(&object)
                || !self.additional_candidate(spec, goal.actor, source, object)
                || chosen.iter().any(|r| {
                    crate::card_names::same_name(
                        self.object_view(r.card).name(),
                        self.object_view(object.card).name(),
                    )
                })
            {
                return false;
            }
            let mut next = chosen.to_vec();
            next.push(object);
            self.different_name_sacrifice_fundable(goal.clone(), spec, amount, &next)
        })
    }
    pub(super) fn activation_can_fund(
        &self,
        goal: FundingGoal,
        spec: Option<AdditionalCost>,
    ) -> bool {
        let Some(spec) = spec else {
            return self.can_fund(goal);
        };
        if spec.kind.variable_cost_minimum().is_some() {
            let source = goal.source.as_ref().expect("activation source").0;
            let current = self.state.announcing.as_ref().filter(|a| {
                !a.additional_paid
                    && a.item.controller == goal.actor
                    && a.item.source.is_some_and(|s| s.object == source)
                    && goal
                        .source
                        .as_ref()
                        .is_some_and(|(_, index, _, definition, _)| {
                            a.item.kind == StackKind::ActivatedAbility { ability: *index }
                                && a.item.source.is_some_and(|s| s.card == *definition)
                        })
            });
            let selected = current
                .and_then(|a| a.additional_group.as_deref())
                .map_or(&[][..], Vec::as_slice);
            return self.variable_cost_group_fundable(
                goal.clone(),
                self.funding_purpose(&goal),
                spec,
                source,
                selected,
                current.is_some_and(|a| a.additional_group_complete),
            );
        }
        if matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::ExileControlledOrGraveyard | EffectChoiceKind::ExileCraftNonlands(_)
        ) {
            return self.preview_ordered_craft(goal, spec);
        }
        let source = goal.source.as_ref().expect("activation source").0;
        if let EffectChoiceKind::RemoveCreatureCounters(amount) = spec.kind.payment_kind() {
            let purpose = self.funding_purpose(&goal);
            let selected = self
                .state
                .announcing
                .as_ref()
                .filter(|a| {
                    a.item
                        .source
                        .is_some_and(|identity| identity.object == source)
                        && a.item.controller == goal.actor
                })
                .and_then(|a| a.creature_counter_cost.as_deref())
                .map_or(&[][..], Vec::as_slice);
            return self.creature_counter_fundable(goal, purpose, amount, selected);
        }
        if matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::TapPermanents(_)
                | EffectChoiceKind::ExileGraveyardCards(_)
                | EffectChoiceKind::ExileGraveyardRange { .. }
                | EffectChoiceKind::ExileGraveyardSymbols { .. }
        ) {
            let purpose = self.funding_purpose(&goal);
            return self.group_cost_fundable(goal, purpose, spec, source, &[]);
        }
        if let EffectChoiceKind::SacrificeDifferentNames(amount) = spec.kind.payment_kind() {
            return self.different_name_sacrifice_fundable(goal, spec, amount, &[]);
        }
        if let Some(n) = evidence::threshold(spec) {
            return self.can_collect_evidence(goal.actor, n) && self.can_fund(goal);
        }
        if forage::is_forage_cost(spec) {
            return self.forage_cost_fundable(goal, spec, None);
        }
        self.state.objects.iter().enumerate().any(|(id, _)| {
            let object = self.object_ref(id as CardId);
            self.additional_candidate_for_funding(spec, goal.actor, source, object)
                && self.can_fund(FundingGoal {
                    reserve: Some((object, spec)),
                    ..goal.clone()
                })
        })
    }
    pub(super) fn additional_choice_available(&self, object: ObjectRef) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        if !self.mana_parent_fundable(Some(object), None) {
            return false;
        }
        let Some(spec) = a.additional_cost() else {
            return false;
        };
        if spec.kind.variable_cost_minimum().is_some() {
            return self.variable_cost_candidate_available(a, spec, object);
        }
        if self.cast_costs_mana_paid() {
            if !self.additional_candidate(
                spec,
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .object,
                object,
            ) {
                return false;
            }
            if matches!(
                spec.kind.payment_kind(),
                EffectChoiceKind::TapPermanents(_)
                    | EffectChoiceKind::ExileGraveyardCards(_)
                    | EffectChoiceKind::ExileGraveyardRange { .. }
                    | EffectChoiceKind::ExileGraveyardSymbols { .. }
                    | EffectChoiceKind::ExileCraftNonlands(_)
                    | EffectChoiceKind::SacrificeDifferentNames(_)
            ) {
                let mut selected = a.additional_group.as_deref().cloned().unwrap_or_default();
                if selected.contains(&object) {
                    return false;
                }
                if matches!(
                    spec.kind.payment_kind(),
                    EffectChoiceKind::SacrificeDifferentNames(_)
                ) && selected.iter().any(|r| {
                    crate::card_names::same_name(
                        self.object_name(r.card),
                        self.object_name(object.card),
                    )
                }) {
                    return false;
                }
                selected.push(object);
                return self.cast_cost_group_fundable(&selected, false);
            }
            return self.cast_cost_object_fundable(object);
        }
        if let EffectChoiceKind::SacrificeDifferentNames(amount) = spec.kind.payment_kind() {
            let mut selected = a
                .additional_group
                .as_ref()
                .map_or_else(Vec::new, |g| g.to_vec());
            if selected.contains(&object)
                || selected.iter().any(|r| {
                    crate::card_names::same_name(
                        self.object_view(r.card).name(),
                        self.object_view(object.card).name(),
                    )
                })
                || !self.additional_candidate(
                    spec,
                    a.item.controller,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object,
                    object,
                )
            {
                return false;
            }
            selected.push(object);
            return self.different_name_sacrifice_fundable(
                self.funding_goal(a),
                spec,
                amount,
                &selected,
            );
        }
        if matches!(
            spec.kind.payment_kind(),
            EffectChoiceKind::TapPermanents(_)
                | EffectChoiceKind::ExileGraveyardCards(_)
                | EffectChoiceKind::ExileGraveyardRange { .. }
                | EffectChoiceKind::ExileGraveyardSymbols { .. }
        ) {
            let mut selected = a
                .additional_group
                .as_ref()
                .map_or_else(Vec::new, |g| g.to_vec());
            if selected.contains(&object)
                || !self.additional_candidate(
                    spec,
                    a.item.controller,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object,
                    object,
                )
                || (matches!(spec.kind.payment_kind(), EffectChoiceKind::TapPermanents(_))
                    && tap_cost_reserves_source(&self.funding_goal(a), object))
            {
                return false;
            }
            selected.push(object);
            return self.group_cost_fundable(
                self.funding_goal(a),
                self.purpose_for_item(&a.item),
                spec,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .object,
                &selected,
            );
        }
        if !self.additional_candidate_for_funding(
            spec,
            a.item.controller,
            a.item
                .source
                .expect("cost announcement has object source")
                .object,
            object,
        ) {
            return false;
        }
        let mut goal = self.funding_goal(a);
        goal.reserve = Some((object, spec));
        self.can_fund(goal)
    }
    pub(super) fn additional_life_available(&self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        let Some(AdditionalCost {
            kind: EffectChoiceKind::DiscardOrLife(amount),
            ..
        }) = a.additional_cost()
        else {
            return false;
        };
        if self.state.players[a.item.controller.index()].life < i128::from(amount) {
            return false;
        }
        self.discard_or_life_route_fundable(a)
    }
    pub(super) fn choose_additional_life(&mut self) {
        let a = self.state.announcing.as_mut().unwrap();
        a.additional_life = true;
        self.state.pending = Pending::PayCosts {
            player: a.item.controller,
        };
        self.resume_cast_cost_selection();
    }
    pub(super) fn choose_additional_cost(&mut self, object: ObjectRef) {
        let a = self.state.announcing.as_mut().unwrap();
        if a.additional_cost()
            .is_some_and(|spec| spec.kind.variable_cost_minimum().is_some())
        {
            a.additional_group.get_or_insert_default().push(object);
            return;
        }
        if matches!(
            a.additional_cost().unwrap().kind,
            EffectChoiceKind::ExileGraveyardSymbols { .. }
                | EffectChoiceKind::ExileCraftNonlands(_)
        ) {
            a.additional_group.get_or_insert_default().push(object);
            return;
        }
        if let EffectChoiceKind::TapPermanents(amount)
        | EffectChoiceKind::ExileGraveyardCards(amount)
        | EffectChoiceKind::SacrificeDifferentNames(amount) =
            a.additional_cost().unwrap().kind.payment_kind()
        {
            let group = a
                .additional_group
                .get_or_insert_with(|| Box::new(Vec::new()));
            group.push(object);
            if group.len() < usize::from(amount) {
                return;
            }
        }
        a.additional = Some(object);
        self.state.pending = Pending::PayCosts {
            player: a.item.controller,
        };
        self.resume_cast_cost_selection();
    }
    pub(super) fn pay_selected_creature_counters(
        &mut self,
        item: &mut SpellView,
        selected: &[(ObjectRef, CounterKind)],
    ) {
        for &(object, kind) in selected {
            self.pay_counters(object.card, &Counters::of(kind, 1));
            if matches!(item.kind, StackKind::ActivatedAbility { .. })
                && kind == CounterKind::Loyalty
            {
                item.loyalty_counters_paid = item
                    .loyalty_counters_paid
                    .checked_add(1)
                    .capacity("loyalty activation counter payment exhausted");
            }
        }
    }
    pub(super) fn pay_additional_cost(&mut self, a: &mut Announcement) {
        let Some(spec) = a.additional_cost() else {
            return;
        };
        if spec.kind.variable_cost_minimum().is_some() {
            self.pay_variable_additional_cost(a, spec);
            return;
        }
        // The local announcement is outside State while an external cost commits.
        // Preserve its actual pre-departure source, not a borrowed definition.
        if matches!(a.item.kind, StackKind::ActivatedAbility { .. })
            && self.on_battlefield(
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .object
                    .card,
            )
            && self.object_ref(
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .object
                    .card,
            ) == a
                .item
                .source
                .expect("cost announcement has object source")
                .object
        {
            a.item
                .source
                .as_mut()
                .expect("activated ability source exists")
                .facts = DamageFacts::from_view(
                &self.object_view(
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object
                        .card,
                ),
            );
        }
        if let EffectChoiceKind::ExileCraftNonlands(minimum) = spec.kind.payment_kind() {
            let group = a
                .additional_group
                .as_deref()
                .expect("selected craft materials");
            assert!(group.len() >= usize::from(minimum));
            let source = a.item.source.expect("craft source").object;
            assert!(group.iter().all(|&object| self.additional_candidate(
                spec,
                a.item.controller,
                source,
                object
            )));
            let events = self.move_objects_raw(
                &group.iter().map(|r| r.card).collect::<Vec<_>>(),
                Zone::Exile,
            );
            a.item.craft_materials = Some(Box::new(
                events
                    .iter()
                    .filter(|event| event.moved && event.destination == Zone::Exile)
                    .map(|event| event.after)
                    .collect(),
            ));
            self.finish_zone_events(&events, &[]);
            for event in &events {
                self.craft_exile_triggers(event, a.item.controller);
            }
            a.additional_paid = true;
            return;
        }
        if let EffectChoiceKind::ExileGraveyardSymbols { color, minimum } = spec.kind.payment_kind()
        {
            let group = a.additional_group.as_deref().unwrap();
            assert!(self.graveyard_symbol_total(group, color) >= u128::from(minimum));
            assert!(group.iter().all(|&r| {
                self.additional_candidate(
                    spec,
                    a.item.controller,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object,
                    r,
                )
            }));
            let events = self.move_objects_raw(
                &group.iter().map(|r| r.card).collect::<Vec<_>>(),
                Zone::Exile,
            );
            a.item.exiled_symbol_cards = Some(Box::new(
                events
                    .iter()
                    .filter(|e| e.moved && e.destination == Zone::Exile)
                    .map(|e| e.after)
                    .collect(),
            ));
            self.finish_zone_events(&events, &[]);
            a.additional_paid = true;
            return;
        }
        if let EffectChoiceKind::RemoveCreatureCounters(amount) = spec.kind.payment_kind() {
            let selected = a.creature_counter_cost.as_ref().unwrap();
            assert_eq!(selected.len(), usize::from(amount));
            self.pay_selected_creature_counters(&mut a.item, selected);
            a.additional_paid = true;
            return;
        }
        if let EffectChoiceKind::SacrificeDifferentNames(amount) = spec.kind.payment_kind() {
            let group = a
                .additional_group
                .as_ref()
                .expect("selected sacrifice group");
            assert_eq!(group.len(), usize::from(amount));
            assert!(group.iter().all(|&r| {
                self.additional_candidate(
                    spec,
                    a.item.controller,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object,
                    r,
                )
            }));
            self.sacrifice_batch(
                &group
                    .iter()
                    .map(|r| (a.item.controller, r.card))
                    .collect::<Vec<_>>(),
            );
            a.additional_paid = true;
            return;
        }
        if let EffectChoiceKind::ExileGraveyardCards(amount) = spec.kind.payment_kind() {
            let group = a
                .additional_group
                .as_ref()
                .expect("selected graveyard group");
            assert_eq!(group.len(), usize::from(amount));
            assert!(group.iter().all(|&r| {
                self.additional_candidate(
                    spec,
                    a.item.controller,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .object,
                    r,
                )
            }));
            let events = self.move_objects_raw(
                &group.iter().map(|r| r.card).collect::<Vec<_>>(),
                Zone::Exile,
            );
            self.finish_zone_events(&events, &[]);
            a.additional_paid = true;
            return;
        }
        if let EffectChoiceKind::TapPermanents(amount) = spec.kind.payment_kind() {
            let group = a.additional_group.as_ref().expect("selected tap group");
            assert_eq!(group.len(), usize::from(amount));
            assert!(group.iter().all(|&r| {
                !tap_cost_reserves_source(&self.funding_goal(a), r)
                    && self.additional_candidate(
                        spec,
                        a.item.controller,
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .object,
                        r,
                    )
            }));
            if a.item.program().is_some_and(|(_, program)| {
                program.iter().any(|instruction| {
                    matches!(
                        instruction,
                        Instruction::PutCountersScaled {
                            amount: AmountValue::TappedCostCreaturePower,
                            ..
                        }
                    )
                })
            }) {
                let object = group[0];
                a.item.event_object = Some(Box::new(EventObjectView {
                    object,
                    facts: EventObjectFacts::CreatureComparison(
                        CreatureComparisonFacts::from_view(&self.object_view(object.card)),
                    ),
                    conniver_creature: None,
                    subtype_lki: None,
                }));
            }
            self.tap_objects(&group.iter().map(|r| r.card).collect::<Vec<_>>());
            a.additional_paid = true;
            return;
        }
        if evidence::threshold(spec).is_some() {
            self.commit_evidence(
                a.evidence.as_ref().expect("selected evidence"),
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .object,
            );
            a.additional_paid = true;
            return;
        }
        if forage::is_forage_cost(spec) {
            let selection = a.forage.as_ref().expect("forage cost selected");
            if selection.route != Some(ForageRoute::Black) {
                self.commit_forage(a.item.controller, selection);
            }
            a.additional_paid = true;
            return;
        }
        if let EffectChoiceKind::DiscardOrLife(amount) = spec.kind.payment_kind()
            && a.additional_life
        {
            self.pay_life(a.item.controller, i128::from(amount));
            a.additional_paid = true;
            return;
        }
        let object = a.additional.expect("additional cost selected");
        assert!(
            self.additional_candidate(
                spec,
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .object,
                object
            ),
            "unpaid additional cost"
        );
        a.additional_paid = true;
        if object
            == a.item
                .source
                .expect("cost announcement has object source")
                .object
        {
            self.refresh_local_source_counter_quantities(&mut a.item);
        }
        let spec = spec.in_zone(self.state.objects[object.card as usize].zone);
        match spec.kind.payment_kind() {
            EffectChoiceKind::Behold => {
                if self.state.objects[object.card as usize].zone == Zone::Hand {
                    self.remember_public_hand(object);
                }
            }
            EffectChoiceKind::FilteredCost { .. }
            | EffectChoiceKind::TargetAttackingReturn { .. }
            | EffectChoiceKind::AmassGoblins(_)
            | EffectChoiceKind::EmpowerJace(_)
            | EffectChoiceKind::HandToTop
            | EffectChoiceKind::ExileCardFromAnyGraveyard
            | EffectChoiceKind::ExileCreatureAndGraveyard
            | EffectChoiceKind::CoverUpGraveyardCard
            | EffectChoiceKind::CoverUpSearch { .. }
            | EffectChoiceKind::NamedSearch { .. }
            | EffectChoiceKind::CopyExiledTriggerCreature
            | EffectChoiceKind::ExileGraveyardCards(_)
            | EffectChoiceKind::ExileGraveyardRange { .. }
            | EffectChoiceKind::ExileSourceAndNamed { .. }
            | EffectChoiceKind::ExileGraveyardSymbols { .. }
            | EffectChoiceKind::ExileGreatestManaValue
            | EffectChoiceKind::SacrificeGreatestManaValue
            | EffectChoiceKind::SacrificeDifferentNames(_)
            | EffectChoiceKind::SacrificeRange { .. }
            | EffectChoiceKind::RemoveCreatureCounters(_)
            | EffectChoiceKind::TapPermanents(_)
            | EffectChoiceKind::UntapLands { .. }
            | EffectChoiceKind::HandExileCopy
            | EffectChoiceKind::HandPlot { .. }
            | EffectChoiceKind::HandCreature
            | EffectChoiceKind::HandPermanent { .. }
            | EffectChoiceKind::HandLand { .. }
            | EffectChoiceKind::GraveyardCreatures { .. }
            | EffectChoiceKind::LandCards { .. }
            | EffectChoiceKind::DiscardedLand { .. }
            | EffectChoiceKind::CollectEvidence(_)
            | EffectChoiceKind::Forage
            | EffectChoiceKind::ForageOrBlack
            | EffectChoiceKind::SacrificeOrDiscard { .. }
            | EffectChoiceKind::ExileControlledOrGraveyard
            | EffectChoiceKind::ExileCraftNonlands(_)
            | EffectChoiceKind::DiscardOrLife(_)
            | EffectChoiceKind::SacrificeAttachedEquipment => unreachable!(),
            EffectChoiceKind::ExileGraveyard | EffectChoiceKind::ExileControlled => {
                let event = self.move_object(object.card, Zone::Exile);
                if a.ordered_craft() {
                    self.craft_exile_triggers(&event, a.item.controller);
                }
            }
            EffectChoiceKind::Discard
            | EffectChoiceKind::DiscardNamed(_)
            | EffectChoiceKind::DiscardMatchingLegendary
            | EffectChoiceKind::DiscardLegendary => self.discard_batch(&[object.card], None),
            EffectChoiceKind::ReturnUnblockedAttacker => {
                let defender = self
                    .state
                    .attackers
                    .iter()
                    .find(|v| v.card == object.card)
                    .and_then(|v| v.defender);
                if let Some(cast) = &mut a.item.cast {
                    cast.sneak_defender = defender;
                } else {
                    a.item.ninjutsu_defender = defender;
                }
                self.move_object(object.card, Zone::Hand);
            }
            EffectChoiceKind::ReturnToHand => {
                self.move_object(object.card, Zone::Hand);
            }
            EffectChoiceKind::Sacrifice => {
                if a.item.program().is_some_and(|(_, program)| {
                    program
                        .iter()
                        .any(|i| matches!(i, Instruction::SearchSacrificedManaValue { .. }))
                }) {
                    a.item.sacrificed_mana_value =
                        Some(Box::new(self.object_mana_value(object.card)));
                }
                if a.item.program().is_some_and(|(_, program)| {
                    program.iter().any(|i| {
                        matches!(
                            i,
                            Instruction::GainLifeScaled(_, AmountValue::SacrificedToughness)
                        )
                    })
                }) {
                    a.item.sacrificed_toughness = Some(Box::new(self.stats(object.card).toughness));
                }
                if a.item.program().is_some_and(|(_, program)| {
                    program.iter().any(|i| {
                        matches!(
                            i,
                            Instruction::DrawScaled(_, AmountValue::SacrificeCostPower)
                        )
                    })
                }) {
                    a.item.sacrificed_power = Some(Box::new(self.stats(object.card).power));
                }
                if a.item.program().is_some_and(|(_, p)| {
                    p.iter()
                        .any(|i| matches!(i, Instruction::IfSacrificedVehicle { .. }))
                }) {
                    a.item.paid_additional_cost = Some(Box::new(
                        self.sacrifice_cost_batch_outcome(&[(a.item.controller, object.card)])
                            .expect("legal sacrifice cost"),
                    ));
                } else {
                    self.sacrifice_batch(&[(a.item.controller, object.card)]);
                }
            }
        }
    }
    pub(super) fn additional_cost_view(&self, viewer: Player) -> Option<AdditionalCostView> {
        let a = self.state.announcing.as_ref()?;
        if !self.shares_information(viewer, a.item.controller) {
            return None;
        }
        let spec = a.additional_cost()?;
        if forage::is_forage_cost(spec) || evidence::threshold(spec).is_some() {
            return None;
        }
        let candidates = if matches!(self.state.pending, Pending::AdditionalCost { .. }) {
            (0..self.state.objects.len())
                .map(|id| self.object_ref(id as CardId))
                .filter(|&r| self.additional_choice_available(r))
                .collect()
        } else {
            Vec::new()
        };
        Some(AdditionalCostView {
            kind: spec.kind,
            candidates,
            selected: a.additional,
            creature_counter_payments: a
                .creature_counter_cost
                .as_ref()
                .map_or_else(Vec::new, |v| v.as_ref().clone()),
            selected_group: a
                .additional_group
                .as_ref()
                .map_or_else(Vec::new, |v| v.as_ref().clone()),
            paid: a.additional_paid,
            can_finish_group: spec
                .kind
                .variable_cost_minimum()
                .map(|_| self.variable_cost_selection_complete()),
        })
    }
}
