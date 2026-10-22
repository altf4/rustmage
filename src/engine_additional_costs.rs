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
    pub supplemental_counter_amount: Option<u16>,
    pub counters_after_printed: Option<bool>,
    pub counter_order_options: Vec<bool>,
}
impl Announcement {
    fn tap_group_excludes_source(&self, spec: AdditionalCost) -> bool {
        spec.filter.other
            || match self.item.kind {
                StackKind::ActivatedAbility { ability } => {
                    self.item.source_card.activations()[usize::from(ability)]
                        .cost
                        .tap
                }
                _ => false,
            }
    }
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
            EffectChoiceKind::Behold
            | EffectChoiceKind::BeholdAndExile
            | EffectChoiceKind::BeholdMany(_) => match o.zone {
                Zone::Battlefield => o.controller == actor,
                Zone::Hand => o.owner == actor,
                _ => false,
            },
            EffectChoiceKind::Discard => o.owner == actor,
            EffectChoiceKind::BlightCost(_)
            | EffectChoiceKind::Sacrifice
            | EffectChoiceKind::ReturnToHand
            | EffectChoiceKind::TapPermanents(_) => o.controller == actor,
            _ => false,
        };
        if matches!(spec.kind, EffectChoiceKind::BlightCost(amount) if amount > 0)
            && !self.can_receive_counters(object.card)
        {
            return false;
        }
        if !controls_cost_object
            || self.object_ref(object.card) != object
            || object == source && (spec.kind == EffectChoiceKind::Discard || spec.filter.other)
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
        )
    }
    pub(super) fn additional_object_matches(
        spec: AdditionalCost,
        actor: Player,
        o: &Object,
        kinds: u16,
        keywords: Keywords,
        subtypes: GrantedSubtypes,
    ) -> bool {
        let zone = match spec.kind {
            EffectChoiceKind::Behold
            | EffectChoiceKind::BeholdAndExile
            | EffectChoiceKind::BeholdMany(_)
                if matches!(o.zone, Zone::Battlefield | Zone::Hand) =>
            {
                o.zone
            }
            EffectChoiceKind::Discard => Zone::Hand,
            EffectChoiceKind::BlightCost(_)
            | EffectChoiceKind::Sacrifice
            | EffectChoiceKind::ReturnToHand
            | EffectChoiceKind::TapPermanents(_) => Zone::Battlefield,
            _ => return false,
        };
        let f = spec.filter;
        if matches!(
            spec.kind,
            EffectChoiceKind::Behold
                | EffectChoiceKind::BeholdAndExile
                | EffectChoiceKind::BeholdMany(_)
        ) && zone == Zone::Hand
            && o.is_token()
        {
            return false;
        }
        if matches!(spec.kind, EffectChoiceKind::BlightCost(_)) && kinds & 1 == 0 {
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

    pub(super) fn optional_additional_choice_fundable(
        &self,
        a: &Announcement,
        object: Option<ObjectRef>,
    ) -> bool {
        a.cost_options()
            .any(|cost| self.optional_additional_cost_fundable(a, cost, object))
    }
    pub(super) fn optional_additional_cost_fundable(
        &self,
        a: &Announcement,
        cost: ManaCost,
        object: Option<ObjectRef>,
    ) -> bool {
        let Some(spec) = a.item.source_card.additional_cost() else {
            return false;
        };
        let EffectChoiceKind::OptionalBlightCost(amount) = spec.kind else {
            return false;
        };
        let cost_spec = AdditionalCost {
            kind: EffectChoiceKind::BlightCost(amount),
            ..spec
        };
        if object.is_some_and(|object| {
            !self.additional_candidate(cost_spec, a.item.controller, a.item.object, object)
        }) {
            return false;
        }
        let mut proposal = a.clone();
        proposal.cost = Some(cost);
        proposal.additional = object;
        proposal.item.cast.as_mut().unwrap().optional_additional = Some(object.is_some());
        if let Some(mode) = proposal.item.mode {
            if !self.modal_optional_intent_matches(&proposal.item, usize::from(mode)) {
                return false;
            }
        } else if !proposal
            .item
            .source_card
            .mode_optional_additional_paid()
            .is_empty()
        {
            return proposal
                .item
                .modes()
                .unwrap()
                .iter()
                .enumerate()
                .any(|(index, mode)| {
                    if !self.modal_choice_unused(&proposal.item, index)
                        || !self.item_targets_available(&proposal.item, mode.targets)
                    {
                        return false;
                    }
                    let mut selected = proposal.clone();
                    selected.item.mode = Some(index as u16);
                    selected.plan = mode.targets;
                    self.optional_additional_cost_fundable(&selected, cost, object)
                        && self.announcement_target_fundable(
                            &selected,
                            Some(mode.targets),
                            None,
                            None,
                            None,
                            false,
                        )
                });
        }
        let goal = self.funding_goal(&proposal);
        self.can_fund_for(goal, self.purpose_for_item(&proposal.item))
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
        self.spell_can_fund_total_with_purpose(actor, card, cost, route, definition, purpose)
    }
    pub(super) fn spell_can_fund_total_at_mana_value(
        &self,
        actor: Player,
        card: CardId,
        cost: ManaCost,
        definition: Card,
        mana_value: u64,
    ) -> bool {
        let mana_lots::Purpose::Spell(mut facts) =
            self.spell_mana_purpose_for_card(actor, card, definition)
        else {
            unreachable!()
        };
        facts.mana_value = mana_value;
        self.spell_can_fund_total_with_purpose(
            actor,
            card,
            cost,
            None,
            definition,
            mana_lots::Purpose::Spell(facts),
        )
    }
    fn spell_can_fund_total_with_purpose(
        &self,
        actor: Player,
        card: CardId,
        cost: ManaCost,
        route: Option<ForageRoute>,
        definition: Card,
        purpose: mana_lots::Purpose,
    ) -> bool {
        let base = FundingGoal {
            actor,
            required: cost.colored,
            generic: cost.generic,
            source: None,
            reserve: None,
        };
        let fund = |goal| {
            if self.may_have_convoke(actor, definition) {
                self.can_fund_convoke_for(goal, purpose)
            } else {
                self.can_fund_for(goal, purpose)
            }
        };
        let Some(mut spec) = definition.additional_cost() else {
            return fund(base);
        };
        // Initial availability may choose zero. It still needs a controlled
        // creature witness, even though that witness receives no counters.
        if spec.kind == EffectChoiceKind::BlightXCost {
            spec.kind = EffectChoiceKind::BlightCost(0);
        }
        if matches!(
            spec.kind,
            EffectChoiceKind::OptionalBlightCost(_) | EffectChoiceKind::OptionalBeholdType(_)
        ) {
            return fund(base);
        }
        if forage::is_forage_cost(spec) {
            return if route == Some(ForageRoute::Black) {
                fund(base)
            } else {
                self.forage_cost_fundable_for(base, spec, route, purpose)
            };
        }
        self.state.objects.iter().enumerate().any(|(id, _)| {
            let object = self.object_ref(id as CardId);
            self.additional_candidate(spec, actor, self.object_ref(card), object)
                && fund(FundingGoal {
                    reserve: Some((object, spec)),
                    ..base.clone()
                })
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
        if let EffectChoiceKind::TapPermanents(amount) = spec.kind {
            let excludes_source = spec.filter.other
                || goal
                    .source
                    .as_ref()
                    .is_some_and(|(_, index, _, definition)| {
                        definition.activations()[usize::from(*index)].cost.tap
                    });
            let count = (0..self.state.objects.len())
                .filter(|&id| {
                    let r = self.object_ref(id as CardId);
                    (!excludes_source || r != source)
                        && self.additional_candidate(spec, goal.actor, source, r)
                })
                .count();
            return count >= usize::from(amount) && self.can_fund(goal);
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
        let Some(spec) = a.additional_cost() else {
            return false;
        };
        if matches!(spec.kind, EffectChoiceKind::BeholdMany(_)) {
            let mut selected = a
                .additional_group
                .as_deref()
                .map_or_else(Vec::new, |group| group.clone());
            if selected.contains(&object) {
                return false;
            }
            selected.push(object);
            return self.behold_group_fundable(a, None, &selected);
        }
        if let EffectChoiceKind::TapPermanents(_) = spec.kind {
            return (!a.tap_group_excludes_source(spec) || object != a.item.object)
                && !a
                    .additional_group
                    .as_ref()
                    .is_some_and(|v| v.contains(&object))
                && self.additional_candidate(spec, a.item.controller, a.item.object, object);
        }
        if !self.additional_candidate(spec, a.item.controller, a.item.object, object) {
            return false;
        }
        let mut goal = self.funding_goal(a);
        goal.reserve = Some((object, spec));
        if a.item.kind == StackKind::Spell
            && self.may_have_convoke(a.item.controller, a.item.source_card)
        {
            self.can_fund_convoke_for(goal, self.purpose_for_item(&a.item))
        } else {
            self.can_fund(goal)
        }
    }
    pub(super) fn choose_additional_cost(&mut self, object: ObjectRef) {
        let a = self.state.announcing.as_mut().unwrap();
        if let Some(AdditionalCost {
            kind: EffectChoiceKind::TapPermanents(amount) | EffectChoiceKind::BeholdMany(amount),
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
        if let EffectChoiceKind::BeholdMany(amount) = spec.kind {
            let group = a.additional_group.as_ref().expect("selected behold group");
            assert_eq!(group.len(), usize::from(amount));
            for &reference in group.iter() {
                assert!(self.additional_candidate(
                    spec,
                    a.item.controller,
                    a.item.object,
                    reference
                ));
                if self.state.objects[reference.card as usize].zone == Zone::Hand
                    && !self.state.known_hand.contains(&reference)
                {
                    self.state.known_hand.push(reference);
                }
            }
            a.additional_paid = true;
            return;
        }
        if let EffectChoiceKind::TapPermanents(amount) = spec.kind {
            let group = a.additional_group.as_ref().expect("selected tap group");
            assert_eq!(group.len(), usize::from(amount));
            assert!(group.iter().all(|&r| (!a.tap_group_excludes_source(spec)
                || r != a.item.object)
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
            EffectChoiceKind::Behold
            | EffectChoiceKind::BeholdAndExile
            | EffectChoiceKind::BeholdMany(_) => {
                if self.state.objects[object.card as usize].zone == Zone::Hand
                    && !self.state.known_hand.contains(&object)
                {
                    self.state.known_hand.push(object);
                }
                if spec.kind == EffectChoiceKind::BeholdAndExile {
                    let event = self.move_object(object.card, Zone::Exile);
                    if event.moved && event.destination == Zone::Exile {
                        self.remember_linked_exile_for(a.item.object, &[object.card]);
                    }
                }
            }
            EffectChoiceKind::BlightCost(amount) => {
                self.activation_cost_counters(
                    object,
                    CounterKind::MinusOne,
                    u64::from(amount),
                    a.item.controller,
                );
            }
            EffectChoiceKind::ExileFromHand
            | EffectChoiceKind::ExileGraveyardCard
            | EffectChoiceKind::Blight(_)
            | EffectChoiceKind::BlightXCost
            | EffectChoiceKind::OptionalBlightCost(_)
            | EffectChoiceKind::OptionalBeholdType(_)
            | EffectChoiceKind::RemoveCreatureCounters(_)
            | EffectChoiceKind::TapPermanents(_)
            | EffectChoiceKind::UntapLands { .. }
            | EffectChoiceKind::PowerDifferenceBoost
            | EffectChoiceKind::KeepSharedCreatureTypes
            | EffectChoiceKind::RemoveCounters
            | EffectChoiceKind::Proliferate
            | EffectChoiceKind::HandCreature
            | EffectChoiceKind::HandLand { .. }
            | EffectChoiceKind::LandCards { .. }
            | EffectChoiceKind::CollectEvidence(_)
            | EffectChoiceKind::Forage
            | EffectChoiceKind::ForageOrBlack => unreachable!(),
            EffectChoiceKind::Discard => self.discard_batch(&[object.card], None),
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
        let supplemental = a
            .creature_counter_cost
            .as_ref()
            .filter(|c| c.amount.is_some());
        let choosing_counters = matches!(
            self.state.pending,
            Pending::CreatureCounterCost { .. } | Pending::CounterCostOrder { .. }
        );
        let spec = if choosing_counters {
            Self::creature_counter_spec(supplemental?.amount.unwrap())
        } else {
            a.additional_cost()
                .or_else(|| supplemental.map(|c| Self::creature_counter_spec(c.amount.unwrap())))?
        };
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
        let candidates = if matches!(self.state.pending, Pending::CreatureCounterCost { .. }) {
            let mut refs = Vec::new();
            self.visit_creature_counter_cost(&mut |action| {
                if let Action::ChooseCreatureCounterCost { object, .. } = action
                    && !refs.contains(&object)
                {
                    refs.push(object);
                }
                false
            });
            refs
        } else {
            candidates
        };
        Some(AdditionalCostView {
            kind: spec.kind,
            candidates,
            selected: a.additional,
            creature_counter_payments: a
                .creature_counter_cost
                .as_ref()
                .map_or_else(Vec::new, |v| v.selected.clone()),
            selected_group: a
                .additional_group
                .as_ref()
                .map_or_else(Vec::new, |v| v.as_ref().clone()),
            paid: if choosing_counters {
                supplemental.is_some_and(|c| c.paid)
            } else {
                a.additional_paid
            },
            supplemental_counter_amount: supplemental.and_then(|c| c.amount),
            counters_after_printed: supplemental.and_then(|c| c.after_printed),
            counter_order_options: if matches!(self.state.pending, Pending::CounterCostOrder { .. })
            {
                [false, true]
                    .into_iter()
                    .filter(|&after| self.supplemental_counter_order_available(after))
                    .collect()
            } else {
                Vec::new()
            },
        })
    }
}
