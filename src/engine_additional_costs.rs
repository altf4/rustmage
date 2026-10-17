//! Owned mandatory spell and activation costs, chosen after targets and paid before cast completion.
use super::*;

#[derive(Clone, Debug, Serialize)]
pub struct AdditionalCostView {
    pub kind: EffectChoiceKind,
    pub candidates: Vec<ObjectRef>,
    pub selected: Option<ObjectRef>,
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
            EffectChoiceKind::Discard => o.owner == actor,
            EffectChoiceKind::Sacrifice => o.controller == actor,
            _ => false,
        };
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
            EffectChoiceKind::Discard => Zone::Hand,
            EffectChoiceKind::Sacrifice => Zone::Battlefield,
            _ => return false,
        };
        let f = spec.filter;
        (if zone == Zone::Battlefield {
            o.controller
        } else {
            o.owner
        }) == actor
            && o.zone == zone
            && (!f.artifact_enchantment_or_token || kinds & 48 != 0 || o.is_token())
            && (f.kinds == 0 || kinds & u16::from(f.kinds) != 0)
            && (!f.nonland || !o.card.is_land())
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
            return self.can_fund(FundingGoal {
                actor,
                required: cost.colored,
                generic: cost.generic,
                source: None,
                reserve: None,
            });
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
        let base = FundingGoal {
            actor,
            required: cost.colored,
            generic: cost.generic,
            source: None,
            reserve: None,
        };
        if definition.has_convoke() {
            return self.can_fund_convoke(base);
        }
        let Some(spec) = definition.additional_cost() else {
            return self.can_fund(base);
        };
        if forage::is_forage_cost(spec) {
            return if route == Some(ForageRoute::Black) {
                self.can_fund(base)
            } else {
                self.forage_cost_fundable(base, spec, route)
            };
        }
        self.state.objects.iter().enumerate().any(|(id, _)| {
            let object = self.object_ref(id as CardId);
            self.additional_candidate(spec, actor, self.object_ref(card), object)
                && self.can_fund(FundingGoal {
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
        if !self.additional_candidate(spec, a.item.controller, a.item.object, object) {
            return false;
        }
        let mut goal = self.funding_goal(a);
        goal.reserve = Some((object, spec));
        self.can_fund(goal)
    }
    pub(super) fn choose_additional_cost(&mut self, object: ObjectRef) {
        let a = self.state.announcing.as_mut().unwrap();
        a.additional = Some(object);
        self.state.pending = Pending::PayCosts {
            player: a.item.controller,
        };
    }
    pub(super) fn pay_additional_cost(&mut self, a: &mut Announcement) {
        let Some(spec) = a.additional_cost() else {
            return;
        };
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
            EffectChoiceKind::UntapLands { .. }
            | EffectChoiceKind::HandCreature
            | EffectChoiceKind::HandLand { .. }
            | EffectChoiceKind::LandCards { .. }
            | EffectChoiceKind::CollectEvidence(_)
            | EffectChoiceKind::Forage
            | EffectChoiceKind::ForageOrBlack => unreachable!(),
            EffectChoiceKind::Discard => self.discard_batch(&[object.card], None),
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
            paid: a.additional_paid,
        })
    }
}
