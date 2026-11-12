//! Exact, deferred collect-evidence transactions shared by costs and resolution.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EvidenceSelection {
    pub player: Player,
    pub threshold: u64,
    pub excluded: Option<ObjectRef>,
    pub selected: Vec<ObjectRef>,
    pub total: u128,
    pub complete: bool,
    pub paid: bool,
    pub optional: bool,
    pub skip_to: usize,
    pub ward: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl EvidenceSelection {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.selected.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EvidenceEventView {
    pub sequence: u64,
    pub player: Player,
    pub source: ObjectRef,
    pub threshold: u64,
    pub total_mana: u128,
    pub exiled: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct EvidenceState {
    pub sources: Vec<CardId>,
    pub last: Option<EvidenceEventView>,
    pub sequence: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl EvidenceState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.sources.len() * std::mem::size_of::<CardId>()
            + self
                .last
                .as_ref()
                .map_or(0, |e| e.exiled.len() * std::mem::size_of::<ObjectRef>())
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct EvidenceView {
    pub player: Player,
    pub threshold: u64,
    pub selected: Vec<ObjectRef>,
    pub total_mana: u128,
    pub candidates: Vec<ObjectRef>,
    pub optional: bool,
    pub complete: bool,
    pub paid: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn threshold(spec: AdditionalCost) -> Option<u64> {
    if let EffectChoiceKind::CollectEvidence(n) = spec.kind {
        Some(n)
    } else {
        None
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn cost(n: u64) -> AdditionalCost {
    AdditionalCost {
        kind: EffectChoiceKind::CollectEvidence(n),
        filter: SacrificeFilter {
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 0,
            nonland: false,
            other: false,
            tokens: None,
            artifact_enchantment_or_token: false,
            subtype: None,
        },
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn evidence_data(&mut self) -> &mut EvidenceState {
        self.state
            .evidence_data
            .get_or_insert_with(|| Box::new(EvidenceState::default()))
    }
    pub(super) fn register_evidence_source(&mut self, id: CardId) {
        let d = self.evidence_data();
        assert!(!d.sources.contains(&id));
        d.sources.push(id);
    }
    fn evidence_card(&self, player: Player, object: ObjectRef) -> bool {
        (object.card as usize) < self.state.objects.len()
            && additional_costs::graveyard_exile_card_matches(self, player, object, None)
    }
    pub(super) fn can_collect_evidence(
        &self,
        player: Player,
        threshold: u64,
        excluded: Option<ObjectRef>,
    ) -> bool {
        if threshold == 0 {
            return true;
        }
        // Nonnegative values make existence a linear sum, with no subset search.
        let mut total = 0u128;
        for (index, card) in self.state.objects.iter().enumerate() {
            let object = self.object_ref(index as CardId);
            if Some(object) != excluded && self.evidence_card(player, object) {
                total += u128::from(card.card.fixed_mana_value());
                if total >= u128::from(threshold) {
                    return true;
                }
            }
        }
        false
    }
    pub(super) fn evidence_cast_fundable(
        &self,
        p: Player,
        id: CardId,
        mana: ManaCost,
        collect: bool,
    ) -> bool {
        let card = self.state.objects[id as usize].card;
        let discount = if collect {
            card.evidence_reduction()
        } else {
            0
        };
        let Some(mana) = self.spell_total_with_reductions(
            p,
            id,
            mana,
            card,
            card.fixed_mana_value(),
            0,
            ManaCost {
                colored: [0; 6],
                generic: discount,
            },
            0,
        ) else {
            return false;
        };
        let goal = FundingGoal {
            actor: p,
            required: mana.colored,
            generic: mana.generic,
            source: None,
            reserve: None,
        };
        if collect {
            self.can_fund_evidence_for(
                goal,
                self.object_ref(id),
                card.collect_evidence_cost().unwrap().into(),
                self.spell_mana_purpose_for_card(p, id, card),
            )
        } else {
            self.can_fund_for(goal, self.spell_mana_purpose_for_card(p, id, card))
        }
    }
    pub(super) fn evidence_intent_fundable(&self, collect: bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        if a.has_ordered_origin_payment()
            || a.has_ordered_life_cost()
            || self.cast_requires_target_payment(a.item.controller, a.item.source.unwrap().card)
        {
            let mut proposal = a.clone();
            proposal.item.cast.as_mut().unwrap().evidence = Some(collect);
            return proposal
                .cost_options()
                .any(|cost| self.announcement_cost_fundable(&proposal, cost));
        }
        a.cost_options().any(|c| {
            self.evidence_cast_fundable(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
                c,
                collect,
            )
        })
    }
    fn evidence_selection(&self) -> &EvidenceSelection {
        self.state
            .announcing
            .as_ref()
            .and_then(|a| a.evidence.as_deref())
            .or_else(|| {
                self.state
                    .resolving
                    .as_ref()
                    .and_then(|r| r.evidence.as_deref())
            })
            .expect("evidence continuation")
    }
    fn evidence_selection_mut(&mut self) -> &mut EvidenceSelection {
        if let Some(a) = &mut self.state.announcing {
            a.evidence.as_deref_mut().unwrap()
        } else {
            self.state
                .resolving
                .as_mut()
                .unwrap()
                .evidence
                .as_deref_mut()
                .unwrap()
        }
    }
    pub(super) fn start_announcement_evidence(&mut self) {
        let a = self.state.announcing.as_mut().unwrap();
        let n = threshold(a.additional_cost().unwrap()).unwrap();
        let p = a.item.controller;
        let excluded = a.item.source.map(|source| source.object);
        a.evidence = Some(Box::new(EvidenceSelection {
            player: p,
            threshold: n,
            excluded,
            selected: Vec::new(),
            total: 0,
            complete: false,
            paid: false,
            optional: false,
            skip_to: 0,
            ward: false,
        }));
        self.state.pending = Pending::Evidence { player: p };
    }
    pub(super) fn start_resolution_evidence(&mut self, n: u16, skip: usize, ward: bool) -> bool {
        let r = self.state.resolving.as_ref().unwrap();
        let p = if ward {
            r.item.related_stack.unwrap().controller
        } else {
            r.item.controller
        };
        let skip_to =
            r.pc.checked_add(skip)
                .expect("evidence continuation offset");
        assert!(skip_to <= r.program.len());
        let n = u64::from(n);
        if !self.can_collect_evidence(p, n, None) {
            if ward {
                let link = r.item.related_stack.unwrap();
                self.counter_stack_item(link.id, false);
            } else {
                self.state.resolving.as_mut().unwrap().pc = skip_to;
            }
            return false;
        }
        self.state.resolving.as_mut().unwrap().evidence = Some(Box::new(EvidenceSelection {
            player: p,
            threshold: n,
            excluded: None,
            selected: Vec::new(),
            total: 0,
            complete: false,
            paid: false,
            optional: true,
            skip_to,
            ward,
        }));
        self.state.pending = Pending::Evidence { player: p };
        true
    }
    fn evidence_selection_fundable(&self, extra: Option<ObjectRef>, exact: bool) -> bool {
        let Some(a) = &self.state.announcing else {
            return true;
        };
        if a.production.is_some() {
            let mut proposal = a.clone();
            let selection = proposal.evidence.as_mut().unwrap();
            if let Some(object) = extra {
                selection.selected.push(object);
                selection.total += u128::from(
                    self.state.objects[object.card as usize]
                        .card
                        .fixed_mana_value(),
                );
            }
            selection.complete = exact;
            return self.activation_total_fundable(&proposal);
        }
        if !self.cast_costs_mana_paid() {
            return true;
        }
        let mut selected = self.evidence_selection().selected.clone();
        selected.extend(extra);
        self.cast_cost_group_fundable(&selected, exact)
    }

    fn evidence_candidate(&self, r: ObjectRef) -> bool {
        let e = self.evidence_selection();
        Some(r) != e.excluded
            && self.evidence_card(e.player, r)
            && !e.selected.contains(&r)
            && self.state.announcing.as_ref().is_none_or(|a| {
                a.cast_costs
                    .as_ref()
                    .is_none_or(|costs| !costs.paid_evidence.contains(&r))
            })
            && self.evidence_selection_fundable(Some(r), false)
    }
    pub(super) fn visit_evidence_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let e = self.evidence_selection();
        if e.optional && emit(Action::DeclineEvidence) {
            return true;
        }
        if e.total >= u128::from(e.threshold)
            && self.evidence_selection_fundable(None, true)
            && emit(Action::FinishEvidence)
        {
            return true;
        }
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            if self.evidence_candidate(object) && emit(Action::ChooseEvidence { object }) {
                return true;
            }
        }
        if self.state.announcing.is_some() && !self.cast_costs_mana_paid() {
            return self.visit_cost_payment(emit);
        }
        false
    }
    pub(super) fn reopen_evidence_after_mana_sacrifice(&mut self, source: CardId) {
        let o = &self.state.objects[source as usize];
        if o.zone != Zone::Graveyard || !o.is_card() {
            return;
        }
        if let Some(a) = &mut self.state.announcing
            && a.item.controller == o.owner
            && !a.cast_costs.as_ref().is_some_and(|costs| costs.mana_closed)
            && let Some(e) = &mut a.evidence
            && !e.paid
        {
            e.complete = false;
            self.state.pending = Pending::Evidence {
                player: a.item.controller,
            };
        }
    }
    pub(super) fn choose_evidence(&mut self, object: ObjectRef) {
        let value = self.state.objects[object.card as usize]
            .card
            .fixed_mana_value();
        let e = self.evidence_selection_mut();
        e.selected.push(object);
        e.total = e
            .total
            .checked_add(u128::from(value))
            .capacity("selected evidence mana value exhausted");
    }
    pub(super) fn finish_evidence(&mut self) {
        self.evidence_selection_mut().complete = true;
        if let Some(a) = &self.state.announcing {
            self.state.pending = Pending::PayCosts {
                player: a.item.controller,
            };
            self.resume_cast_cost_selection();
        } else {
            let r = self.state.resolving.as_mut().unwrap();
            let e = r.evidence.take().unwrap();
            let source = r.item.source.expect("evidence effect has source").object;
            self.commit_evidence(&e, source);
            self.run_program();
        }
    }
    pub(super) fn decline_evidence(&mut self) {
        let r = self.state.resolving.as_mut().unwrap();
        let e = r.evidence.take().unwrap();
        if e.ward {
            let link = r.item.related_stack.unwrap();
            self.counter_stack_item(link.id, false);
        } else {
            r.pc = e.skip_to;
        }
        self.run_program();
    }
    pub(super) fn pay_cast_evidence(&mut self, a: &mut Announcement) {
        let selection = a.evidence.as_mut().expect("selected evidence");
        selection.paid = true;
        if let Some(costs) = &mut a.cast_costs {
            assert!(
                selection
                    .selected
                    .iter()
                    .all(|r| !costs.paid_evidence.contains(r))
            );
            costs.paid_evidence.extend_from_slice(&selection.selected);
        }
        self.commit_evidence(selection, a.item.source.unwrap().object);
    }

    pub(super) fn evidence_selection_total(
        &self,
        player: Player,
        excluded: Option<ObjectRef>,
        selected: &[ObjectRef],
    ) -> Option<u128> {
        let mut total = 0;
        for (index, &object) in selected.iter().enumerate() {
            if Some(object) == excluded
                || !self.evidence_card(player, object)
                || selected[..index].contains(&object)
            {
                return None;
            }
            total += u128::from(
                self.state.objects[object.card as usize]
                    .card
                    .fixed_mana_value(),
            );
        }
        Some(total)
    }

    pub(super) fn commit_evidence(&mut self, e: &EvidenceSelection, source: ObjectRef) {
        assert!(e.complete && e.total >= u128::from(e.threshold));
        assert_eq!(
            self.evidence_selection_total(e.player, e.excluded, &e.selected),
            Some(e.total)
        );
        let ids = e.selected.iter().map(|r| r.card).collect::<Vec<_>>();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        self.finish_zone_events(&events, &[]);
        let d = self.evidence_data();
        d.sequence = d
            .sequence
            .checked_add(1)
            .capacity("collected evidence event sequence exhausted");
        d.last = Some(EvidenceEventView {
            sequence: d.sequence,
            player: e.player,
            source,
            threshold: e.threshold,
            total_mana: e.total,
            // A legal payment can be modified by a replacement (CR 118.11).
            // Linked consumers must use only the objects that actually reached exile.
            exiled: events
                .iter()
                .filter(|event| event.moved && event.destination == Zone::Exile)
                .flat_map(|event| event.objects_after())
                .collect(),
        });
        let sources = d.sources.clone();
        for id in sources {
            let source = self.object_view(id);
            if source.controller == e.player {
                for (i, a) in source.card.triggers().iter().enumerate() {
                    if a.condition.event() == TriggerCondition::EvidenceCollected {
                        self.queue_trigger(&source, i);
                    }
                }
            }
        }
    }
    pub(super) fn evidence_view(&self, viewer: Player) -> Option<EvidenceView> {
        let viewer = self.choice_information_player(viewer);
        let e = self
            .state
            .announcing
            .as_ref()
            .and_then(|a| a.evidence.as_deref())
            .or_else(|| {
                self.state
                    .resolving
                    .as_ref()
                    .and_then(|r| r.evidence.as_deref())
            })?;
        if viewer != e.player || self.semantic_actor() != Some(viewer) {
            return None;
        }
        let choosing = matches!(self.state.pending, Pending::Evidence { .. });
        Some(EvidenceView {
            player: e.player,
            threshold: e.threshold,
            selected: e.selected.clone(),
            total_mana: e.total,
            candidates: if choosing {
                (0..self.state.objects.len())
                    .map(|id| self.object_ref(id as CardId))
                    .filter(|&r| self.evidence_candidate(r))
                    .collect()
            } else {
                Vec::new()
            },
            optional: e.optional,
            complete: e.complete,
            paid: e.paid,
        })
    }
}
