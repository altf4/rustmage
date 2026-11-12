//! Exact, deferred collect-evidence transactions shared by costs and resolution.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EvidenceSelection {
    pub player: Player,
    pub threshold: u64,
    pub excluded: Option<ObjectRef>,
    /// A separate cost in the same payment, with its own return continuation.
    pub source_exiles: Box<[ObjectRef]>,
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
        std::mem::size_of::<Self>()
            + (self.selected.len() + self.source_exiles.len()) * std::mem::size_of::<ObjectRef>()
    }
}
/// The activation index separates independent evidence payments on the
/// same permanent. Definition and acquisition identify copied abilities.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LinkedEvidenceExile {
    pub view: LinkedExileView,
    activation: u16,
    definition: Card,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl LinkedEvidenceExile {
    fn matches(&self, item: &SpellView, activation: u16) -> bool {
        self.activation == activation
            && item
                .source
                .is_some_and(|source| source.object == self.view.source)
            && self.definition == item.trigger_definition()
            && item.source_ability_origin == self.view.ability_origin
            && item.linked_activation_cohort() == self.view.acquired_link_cohort
    }

    fn referenced_by(&self, item: &SpellView) -> bool {
        matches!(item.kind, StackKind::TriggeredAbility { .. })
            && self.matches(item, self.activation)
            && item.program().is_some_and(|(_, program)| program.iter().any(|instruction| {
                matches!(instruction, Instruction::CastLinkedEvidenceInstantOrSorcery { activation } if *activation == self.activation)
            }))
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
    fn remember_linked_evidence(&mut self, item: &SpellView, exiled: &[ObjectRef]) {
        let (StackKind::ActivatedAbility { ability }, Some(source)) = (item.kind, item.source)
        else {
            return;
        };
        if exiled.is_empty()
            || !source.trigger_templates().any(|(_, trigger)| {
                trigger.program.iter().any(|instruction| {
                    matches!(instruction, Instruction::CastLinkedEvidenceInstantOrSorcery { activation } if *activation == ability)
                })
            })
        {
            return;
        }
        let turn = self.state.turn;
        let links = &mut self
            .state
            .linked_choices
            .get_or_insert_default()
            .evidence_links;
        links.extend(exiled.iter().map(|&exiled| LinkedEvidenceExile {
            view: LinkedExileView {
                exiled_turn: turn,
                source: source.object,
                exiled,
                ability_origin: item.source_ability_origin,
                acquired_link_cohort: item.linked_activation_cohort(),
            },
            activation: ability,
            definition: source.card,
        }));
    }

    pub(super) fn prune_linked_evidence(&mut self) {
        let state = &mut self.state;
        let Some(links) = state
            .linked_choices
            .as_mut()
            .map(|choices| &mut choices.evidence_links)
        else {
            return;
        };
        let in_zone = |object: ObjectRef, zone| {
            let row = &state.objects[object.card as usize];
            row.generation == object.generation && row.zone == zone
        };
        links.retain(|link| {
            in_zone(link.view.exiled, Zone::Exile)
                && (in_zone(link.view.source, Zone::Battlefield)
                    || state
                        .stack
                        .iter()
                        .chain(&state.pending_triggers)
                        .chain(&state.ordering_triggers)
                        .chain(state.resolving.iter().map(|r| &r.item))
                        .chain(state.announcing.iter().map(|a| &a.item))
                        .any(|item| link.referenced_by(item)))
        });
    }

    pub(super) fn offer_linked_evidence_cast(&mut self, item: &SpellView, activation: u16) -> bool {
        let Some(choices) = &self.state.linked_choices else {
            return false;
        };
        let remaining = choices
            .evidence_links
            .iter()
            .filter_map(|link| {
                let exiled = link.view.exiled;
                let object = &self.state.objects[exiled.card as usize];
                (link.matches(item, activation)
                    && object.zone == Zone::Exile
                    && object.generation == exiled.generation
                    && object.is_card()
                    && CastFace::choices(object.card).any(|face| {
                        face.definition(object.card)
                            .is_some_and(|card| card.kind_mask() & 12 != 0)
                    }))
                .then_some(exiled)
            })
            .collect::<Vec<_>>();
        if remaining.is_empty() {
            return false;
        }
        self.offer_exiled_cards_with_paid_rules(
            item.controller,
            remaining,
            Some(1),
            ResolutionPaidCast {
                kinds: 12,
                exile_instants_sorceries: false,
                bottom_on_graveyard: true,
                spend_any_mana: false,
            },
        );
        true
    }

    pub(super) fn register_evidence_source(&mut self, id: CardId) {
        let d = self.evidence_data();
        if !d.sources.contains(&id) {
            d.sources.push(id);
        }
    }
    fn evidence_card(&self, player: Player, object: ObjectRef) -> bool {
        (object.card as usize) < self.state.objects.len()
            && additional_costs::graveyard_exile_card_matches(self, player, object, None)
    }
    fn evidence_values<'a>(
        &'a self,
        player: Player,
        excluded: &'a [ObjectRef],
    ) -> impl Iterator<Item = u64> + 'a {
        self.state
            .objects
            .iter()
            .enumerate()
            .filter_map(move |(index, card)| {
                let object = self.object_ref(index as CardId);
                (!excluded.contains(&object) && self.evidence_card(player, object))
                    .then(|| card.card.fixed_mana_value())
            })
    }

    fn can_collect_evidence(&self, player: Player, threshold: u64, excluded: &[ObjectRef]) -> bool {
        if threshold == 0 {
            return true;
        }
        // Nonnegative values make existence a linear sum, with no subset search.
        let mut total = 0u128;
        self.evidence_values(player, excluded).any(|value| {
            total += u128::from(value);
            total >= u128::from(threshold)
        })
    }

    pub(super) fn choosing_evidence_x(&self) -> bool {
        self.state.resolving.as_ref().is_some_and(|r| {
            r.program.get(r.pc.saturating_sub(1)) == Some(&Instruction::CollectEvidenceX)
        })
    }

    pub(super) fn maximum_evidence_x(&self, player: Player) -> u32 {
        // X uses the shared scalar declaration domain. Summing all eligible
        // values gives its upper bound because the eventual payment can overpay.
        self.evidence_values(player, &[])
            .map(u128::from)
            .sum::<u128>()
            .min(u128::from(u32::MAX)) as u32
    }

    pub(super) fn choose_resolution_evidence_x(&mut self, value: u32) {
        self.state.resolving.as_mut().unwrap().item.x = Some(value);
        assert!(self.start_resolution_evidence(u64::from(value), 0, false));
        self.evidence_selection_mut().optional = false;
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
            source_exiles: Box::default(),
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
    pub(super) fn start_resolution_evidence(&mut self, n: u64, skip: usize, ward: bool) -> bool {
        self.start_resolution_evidence_payment(n, skip, ward, Box::default())
    }

    pub(super) fn start_source_evidence_return(&mut self, threshold: u64) -> bool {
        let item = &self.state.resolving.as_ref().unwrap().item;
        let (Some(source), Some(movement)) = (item.source, item.source_move.as_deref()) else {
            return false;
        };
        if movement.before != source.object
            || movement.origin != Zone::Battlefield
            || movement.destination != Zone::Graveyard
        {
            return false;
        }
        let source_exiles: Box<[_]> = movement.objects_after().collect();
        if !source_exiles
            .iter()
            .all(|&object| self.source_exilable(object))
        {
            return false;
        }
        self.start_resolution_evidence_payment(threshold, 0, false, source_exiles)
    }

    fn source_exilable(&self, object: ObjectRef) -> bool {
        let owner = self.state.objects[object.card as usize].owner;
        self.evidence_card(owner, object)
    }

    fn start_resolution_evidence_payment(
        &mut self,
        n: u64,
        skip: usize,
        ward: bool,
        source_exiles: Box<[ObjectRef]>,
    ) -> bool {
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
        if !self.can_collect_evidence(p, n, &source_exiles) {
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
            source_exiles,
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
            let e = self.evidence_selection();
            return e
                .source_exiles
                .iter()
                .all(|&object| self.source_exilable(object))
                && (!exact
                    || self.evidence_selection_total(e.player, e.excluded, &e.selected)
                        == Some(e.total));
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
            && !e.source_exiles.contains(&r)
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
            let exiled_sources = self.commit_evidence(&e, source, None);
            let entries: Vec<_> = exiled_sources
                .into_iter()
                .filter_map(|object| {
                    let mut entry = self.source_return_entry(object, Zone::Exile, false, None)?;
                    entry.tapped = true;
                    Some(entry)
                })
                .collect();
            if !entries.is_empty() {
                self.stage_entries(entries, None);
            }
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
        self.commit_evidence(selection, a.item.source.unwrap().object, Some(&a.item));
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

    pub(super) fn commit_evidence(
        &mut self,
        e: &EvidenceSelection,
        source: ObjectRef,
        paying_ability: Option<&SpellView>,
    ) -> Vec<ObjectRef> {
        assert!(e.complete && e.total >= u128::from(e.threshold));
        assert_eq!(
            self.evidence_selection_total(e.player, e.excluded, &e.selected),
            Some(e.total)
        );
        assert!(
            e.source_exiles
                .iter()
                .all(|&object| { !e.selected.contains(&object) && self.source_exilable(object) })
        );
        let ids = e
            .source_exiles
            .iter()
            .chain(&e.selected)
            .map(|r| r.card)
            .collect::<Vec<_>>();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        self.finish_zone_events(&events, &[]);
        let (source_events, evidence_events) = events.split_at(e.source_exiles.len());
        let actual_exiles = |event: &ZoneChange| event.moved && event.destination == Zone::Exile;
        // A legal payment can be modified by a replacement (CR 118.11).
        // Linked consumers must use only the objects that actually reached exile.
        let exiled = evidence_events
            .iter()
            .filter(|event| actual_exiles(event))
            .flat_map(ZoneChange::objects_after)
            .collect::<Vec<_>>();
        if let Some(item) = paying_ability {
            self.remember_linked_evidence(item, &exiled);
        }
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
            exiled,
        });
        let sources = d.sources.clone();
        for id in sources {
            let source = self.object_view(id);
            if source.controller == e.player {
                for (i, a) in source.trigger_templates() {
                    if a.condition.event() == TriggerCondition::EvidenceCollected {
                        self.queue_trigger(&source, i);
                    }
                }
            }
        }
        source_events
            .iter()
            .filter(|event| actual_exiles(event))
            .flat_map(ZoneChange::objects_after)
            .collect()
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
