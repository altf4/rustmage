//! Owned mandatory spell and activation costs, chosen after targets and paid before cast completion.
use super::*;

#[derive(Clone, Debug, Serialize)]
pub struct AdditionalCostView {
    pub kind: EffectChoiceKind,
    pub candidates: Vec<ObjectRef>,
    pub selected: Option<ObjectRef>,
    pub selected_group: Vec<ObjectRef>,
    pub creature_counter_payments: Vec<(ObjectRef, CounterKind)>,
    pub paid: bool,
}
impl Game {
    pub(super) fn additional_candidate(
        &self,
        spec: AdditionalCost,
        actor: Player,
        source: ObjectRef,
        object: ObjectRef,
    ) -> bool {
        let Some(o) = self.state.objects.get(object.card as usize) else {
            return false;
        };
        let controls_cost_object = match spec.kind {
            EffectChoiceKind::Discard
            | EffectChoiceKind::DiscardNamed(_)
            | EffectChoiceKind::ExileGraveyard
            | EffectChoiceKind::ExileGraveyardCards(_) => o.owner == actor,
            EffectChoiceKind::Sacrifice
            | EffectChoiceKind::SacrificeDifferentNames(_)
            | EffectChoiceKind::ReturnToHand
            | EffectChoiceKind::TapPermanents(_) => o.controller == actor,
            _ => false,
        };
        if !controls_cost_object
            || self.object_ref(object.card) != object
            || object == source
                && (matches!(
                    spec.kind,
                    EffectChoiceKind::Discard | EffectChoiceKind::DiscardNamed(_)
                ) || spec.filter.other)
            || spec.filter.source_only && object != source
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
        let zone = match spec.kind {
            EffectChoiceKind::ExileGraveyard | EffectChoiceKind::ExileGraveyardCards(_) => {
                Zone::Graveyard
            }
            EffectChoiceKind::Discard | EffectChoiceKind::DiscardNamed(_) => Zone::Hand,
            EffectChoiceKind::Sacrifice
            | EffectChoiceKind::SacrificeDifferentNames(_)
            | EffectChoiceKind::ReturnToHand
            | EffectChoiceKind::TapPermanents(_) => Zone::Battlefield,
            _ => return false,
        };
        if let EffectChoiceKind::DiscardNamed(required) = spec.kind
            && !crate::card_names::same_name(name, required)
        {
            return false;
        }
        let f = spec.filter;
        if matches!(
            spec.kind,
            EffectChoiceKind::ExileGraveyard | EffectChoiceKind::ExileGraveyardCards(_)
        ) && (o.is_token() || o.copy_kind == CopyKind::Card)
        {
            return false;
        }
        if matches!(spec.kind, EffectChoiceKind::TapPermanents(_)) && o.tapped {
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
        let definition = self.effective_spell_card(card);
        let skips_additional = definition.mana_alternative_replaces_additional(cost);
        let cost = self.reduced_spell_cost(actor, card, cost);
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
        self.spell_can_fund_total(actor, card, cost, None)
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
        let Some(spec) = definition.additional_cost() else {
            return self.can_fund_for(base, purpose);
        };
        if matches!(
            spec.kind,
            EffectChoiceKind::TapPermanents(_) | EffectChoiceKind::ExileGraveyardCards(_)
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
        if !self.can_fund_for(goal.clone(), purpose) {
            return false;
        }
        let amount = match spec.kind {
            EffectChoiceKind::TapPermanents(n) | EffectChoiceKind::ExileGraveyardCards(n) => n,
            _ => unreachable!("group cost"),
        };
        if chosen.len() == usize::from(amount) {
            return self.can_fund_cost_group(goal, purpose, spec, chosen);
        }
        (0..self.state.objects.len()).any(|id| {
            let r = self.object_ref(id as CardId);
            if chosen.contains(&r)
                || !self.additional_candidate(spec, goal.actor, source, r)
                || (matches!(spec.kind, EffectChoiceKind::TapPermanents(_))
                    && goal
                        .source
                        .as_ref()
                        .is_some_and(|(source, _, _, _, _)| *source == r))
            {
                return false;
            }
            let mut next = chosen.to_vec();
            next.push(r);
            self.group_cost_fundable(goal.clone(), purpose, spec, source, &next)
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
        let source = goal.source.as_ref().expect("activation source").0;
        if matches!(
            spec.kind,
            EffectChoiceKind::TapPermanents(_) | EffectChoiceKind::ExileGraveyardCards(_)
        ) {
            let purpose = self.funding_purpose(&goal);
            return self.group_cost_fundable(goal, purpose, spec, source, &[]);
        }
        if let EffectChoiceKind::SacrificeDifferentNames(amount) = spec.kind {
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
            self.additional_candidate(spec, goal.actor, source, object)
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
        if self.cast_costs_mana_paid() {
            if !self.additional_candidate(spec, a.item.controller, a.item.object, object) {
                return false;
            }
            if matches!(
                spec.kind,
                EffectChoiceKind::TapPermanents(_)
                    | EffectChoiceKind::ExileGraveyardCards(_)
                    | EffectChoiceKind::SacrificeDifferentNames(_)
            ) {
                let mut selected = a.additional_group.as_deref().cloned().unwrap_or_default();
                if selected.contains(&object) {
                    return false;
                }
                if matches!(spec.kind, EffectChoiceKind::SacrificeDifferentNames(_))
                    && selected.iter().any(|r| {
                        crate::card_names::same_name(
                            self.object_name(r.card),
                            self.object_name(object.card),
                        )
                    })
                {
                    return false;
                }
                selected.push(object);
                return self.cast_cost_group_fundable(&selected, false);
            }
            return self.cast_cost_object_fundable(object);
        }
        if let EffectChoiceKind::SacrificeDifferentNames(amount) = spec.kind {
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
                || !self.additional_candidate(spec, a.item.controller, a.item.object, object)
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
            spec.kind,
            EffectChoiceKind::TapPermanents(_) | EffectChoiceKind::ExileGraveyardCards(_)
        ) {
            let mut selected = a
                .additional_group
                .as_ref()
                .map_or_else(Vec::new, |g| g.to_vec());
            if selected.contains(&object)
                || !self.additional_candidate(spec, a.item.controller, a.item.object, object)
                || (matches!(spec.kind, EffectChoiceKind::TapPermanents(_))
                    && a.item.kind != StackKind::Spell
                    && object == a.item.object)
            {
                return false;
            }
            selected.push(object);
            return self.group_cost_fundable(
                self.funding_goal(a),
                self.purpose_for_item(&a.item),
                spec,
                a.item.object,
                &selected,
            );
        }
        if !self.additional_candidate(spec, a.item.controller, a.item.object, object) {
            return false;
        }
        let mut goal = self.funding_goal(a);
        goal.reserve = Some((object, spec));
        self.can_fund(goal)
    }
    pub(super) fn choose_additional_cost(&mut self, object: ObjectRef) {
        let a = self.state.announcing.as_mut().unwrap();
        if let Some(AdditionalCost {
            kind:
                EffectChoiceKind::TapPermanents(amount)
                | EffectChoiceKind::ExileGraveyardCards(amount)
                | EffectChoiceKind::SacrificeDifferentNames(amount),
            ..
        }) = a.additional_cost()
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
    pub(super) fn pay_additional_cost(&mut self, a: &mut Announcement) {
        let Some(spec) = a.additional_cost() else {
            return;
        };
        if let EffectChoiceKind::RemoveCreatureCounters(amount) = spec.kind {
            let selected = a.creature_counter_cost.as_ref().unwrap();
            assert_eq!(selected.len(), usize::from(amount));
            for &(r, kind) in selected.iter() {
                self.pay_counters(r.card, &Counters::of(kind, 1));
            }
            a.additional_paid = true;
            return;
        }
        if let EffectChoiceKind::SacrificeDifferentNames(amount) = spec.kind {
            let group = a
                .additional_group
                .as_ref()
                .expect("selected sacrifice group");
            assert_eq!(group.len(), usize::from(amount));
            assert!(group.iter().all(|&r| self.additional_candidate(
                spec,
                a.item.controller,
                a.item.object,
                r
            )));
            self.sacrifice_batch(
                &group
                    .iter()
                    .map(|r| (a.item.controller, r.card))
                    .collect::<Vec<_>>(),
            );
            a.additional_paid = true;
            return;
        }
        if let EffectChoiceKind::ExileGraveyardCards(amount) = spec.kind {
            let group = a
                .additional_group
                .as_ref()
                .expect("selected graveyard group");
            assert_eq!(group.len(), usize::from(amount));
            assert!(group.iter().all(|&r| self.additional_candidate(
                spec,
                a.item.controller,
                a.item.object,
                r
            )));
            let events = self.move_objects_raw(
                &group.iter().map(|r| r.card).collect::<Vec<_>>(),
                Zone::Exile,
            );
            self.finish_zone_events(&events, &[]);
            a.additional_paid = true;
            return;
        }
        if let EffectChoiceKind::TapPermanents(amount) = spec.kind {
            let group = a.additional_group.as_ref().expect("selected tap group");
            assert_eq!(group.len(), usize::from(amount));
            assert!(group.iter().all(|&r| r != a.item.object
                && self.additional_candidate(spec, a.item.controller, a.item.object, r)));
            self.tap_objects(&group.iter().map(|r| r.card).collect::<Vec<_>>());
            a.additional_paid = true;
            return;
        }
        if evidence::threshold(spec).is_some() {
            self.commit_evidence(
                a.evidence.as_ref().expect("selected evidence"),
                a.item.object,
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
        let object = a.additional.expect("additional cost selected");
        assert!(
            self.additional_candidate(spec, a.item.controller, a.item.object, object),
            "unpaid additional cost"
        );
        a.additional_paid = true;
        if object == a.item.object {
            self.refresh_local_source_counter_quantities(&mut a.item);
        }
        match spec.kind {
            EffectChoiceKind::HandToTop
            | EffectChoiceKind::ExileGraveyardCards(_)
            | EffectChoiceKind::ExileGreatestManaValue
            | EffectChoiceKind::SacrificeDifferentNames(_)
            | EffectChoiceKind::RemoveCreatureCounters(_)
            | EffectChoiceKind::TapPermanents(_)
            | EffectChoiceKind::UntapLands { .. }
            | EffectChoiceKind::HandExileCopy
            | EffectChoiceKind::HandCreature
            | EffectChoiceKind::HandPermanent { .. }
            | EffectChoiceKind::HandLand { .. }
            | EffectChoiceKind::LandCards { .. }
            | EffectChoiceKind::DiscardedLand { .. }
            | EffectChoiceKind::CollectEvidence(_)
            | EffectChoiceKind::Forage
            | EffectChoiceKind::ForageOrBlack
            | EffectChoiceKind::SacrificeOrDiscard { .. } => unreachable!(),
            EffectChoiceKind::ExileGraveyard => {
                self.move_object(object.card, Zone::Exile);
            }
            EffectChoiceKind::Discard | EffectChoiceKind::DiscardNamed(_) => {
                self.discard_batch(&[object.card], None)
            }
            EffectChoiceKind::ReturnToHand => {
                self.move_object(object.card, Zone::Hand);
            }
            EffectChoiceKind::Sacrifice => {
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
                self.sacrifice_batch(&[(a.item.controller, object.card)]);
            }
        }
    }
    pub(super) fn additional_cost_view(&self, viewer: Player) -> Option<AdditionalCostView> {
        let a = self.state.announcing.as_ref()?;
        if viewer != a.item.controller {
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
        })
    }
}
