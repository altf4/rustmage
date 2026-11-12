//! Incremental non-targeting tap costs and exact, resolved Saddle designations.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SaddlePayment {
    pub source: ObjectRef,
    pub actor: Player,
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    pub complete: bool,
}
/// A designation is independent of who tapped to pay its activation cost.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SaddleHistory {
    mount: ObjectRef,
    saddled: bool,
    contributors: Vec<ObjectRef>,
}

impl SaddleHistory {
    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.contributors.len() * std::mem::size_of::<ObjectRef>()
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SaddleChoiceView {
    pub source: ObjectRef,
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    /// None only if the exact signed aggregate is outside i128; legality stays exact.
    pub selected_power: Option<i128>,
    pub can_finish: bool,
}
/// Each sign sums magnitudes with a u64 carry above u128. CardId bounds the
/// number of terms, so the high limb cannot overflow. Cancellation is exact.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Default)]
pub(super) struct PowerSum {
    positive: (u64, u128),
    negative: (u64, u128),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl PowerSum {
    pub(super) fn add(&mut self, p: i128) {
        let limb = if p < 0 {
            &mut self.negative
        } else {
            &mut self.positive
        };
        let (low, carry) = limb.1.overflowing_add(p.unsigned_abs());
        limb.1 = low;
        limb.0 += u64::from(carry);
    }
    fn difference(self) -> (bool, u64, u128) {
        let neg = self.negative > self.positive;
        let (a, b) = if neg {
            (self.negative, self.positive)
        } else {
            (self.positive, self.negative)
        };
        let (low, borrow) = a.1.overflowing_sub(b.1);
        (neg, a.0 - b.0 - u64::from(borrow), low)
    }
    pub(super) fn enough(self, n: u16) -> bool {
        let (neg, hi, lo) = self.difference();
        !neg && (hi > 0 || lo >= u128::from(n))
    }
    pub(super) fn represented(self) -> Option<i128> {
        let (neg, hi, lo) = self.difference();
        if hi > 0 {
            return None;
        }
        if neg {
            if lo == 1u128 << 127 {
                Some(i128::MIN)
            } else {
                i128::try_from(lo).ok().map(|n| -n)
            }
        } else {
            i128::try_from(lo).ok()
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ActivatedAbility {
    pub(super) fn saddle_minimum(&self) -> Option<u16> {
        match self.program {
            [Instruction::BecomeSaddled { minimum }] => Some(*minimum),
            _ => None,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    #[cfg(feature = "test-support")]
    pub(super) fn crew_saddle_power(&self, card: CardId, subject: CardId, subtype: &str) -> i128 {
        ability_world::CharacteristicWorld::crew_saddle_contribution(self, card, subject, subtype)
    }
    pub(super) fn is_saddled(&self, object: ObjectRef) -> bool {
        self.source_saddled(object)
            && self.on_battlefield(object.card)
            && self.object_ref(object.card) == object
    }

    fn saddle_history(&self, mount: ObjectRef) -> Option<&SaddleHistory> {
        self.state
            .saddle_history
            .iter()
            .find(|row| row.mount == mount)
    }

    /// An old exact incarnation retains the designation it had at departure.
    pub(super) fn source_saddled(&self, mount: ObjectRef) -> bool {
        self.saddle_history(mount).is_some_and(|row| row.saddled)
    }

    pub(super) fn saddle_contributors(&self, mount: ObjectRef) -> &[ObjectRef] {
        self.saddle_history(mount)
            .map_or(&[], |row| row.contributors.as_slice())
    }

    fn saddle_history_mut(&mut self, mount: ObjectRef) -> &mut SaddleHistory {
        let history = &mut self.state.saddle_history;
        let index = history
            .iter()
            .position(|row| row.mount == mount)
            .unwrap_or_else(|| {
                history
                    .try_reserve(1)
                    .capacity("Saddle history capacity exhausted");
                history.push(SaddleHistory {
                    mount,
                    saddled: false,
                    contributors: Vec::new(),
                });
                history.len() - 1
            });
        &mut history[index]
    }
    pub(super) fn saddle_rider_candidate(
        &self,
        mount: ObjectRef,
        player: Player,
        object: ObjectRef,
        effect: SaddleRiderEffect,
    ) -> bool {
        self.saddle_contributors(mount).contains(&object)
            && self.on_battlefield(object.card)
            && self.object_ref(object.card) == object
            && self.is_creature(object.card)
            && match effect {
                SaddleRiderEffect::CopyNonlegendaryAttacking => {
                    !ability_world::CharacteristicWorld::legendary(self, object.card)
                }
                SaddleRiderEffect::SacrificeOneForPower => {
                    self.state.objects[object.card as usize].controller == player
                        && !ability_world::CharacteristicWorld::sacrifice_prohibited(
                            self,
                            object.card,
                        )
                }
                SaddleRiderEffect::ReturnAnyToHand | SaddleRiderEffect::BlinkSourceAndUpToOne => {
                    true
                }
            }
    }

    pub(super) fn start_saddle_rider_choice(
        &mut self,
        item: &SpellView,
        effect: SaddleRiderEffect,
    ) -> bool {
        if effect == SaddleRiderEffect::SacrificeOneForPower {
            self.state.resolving.as_mut().unwrap().sacrificed_power = None;
        }
        self.start_effect_choice(
            item,
            EffectChoiceKind::SaddleRider(effect),
            PlayerSet::Controller,
            evidence::cost(0).filter,
            u16::from(effect != SaddleRiderEffect::ReturnAnyToHand),
            (effect == SaddleRiderEffect::SacrificeOneForPower).then_some(0),
        )
    }

    pub(super) fn finish_saddle_rider_choice(&mut self, effect: SaddleRiderEffect) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let choice = resolution.choice.take().unwrap();
        let controller = resolution.item.controller;
        let chosen = &choice.selected[controller.index()];
        match effect {
            SaddleRiderEffect::ReturnAnyToHand => {
                let ids: Vec<_> = chosen.iter().map(|object| object.card).collect();
                let events = self.move_objects_raw(&ids, Zone::Hand);
                self.finish_zone_events(&events, &[]);
            }
            SaddleRiderEffect::BlinkSourceAndUpToOne => {
                let mount = choice.source;
                let ids: Vec<_> = std::iter::once(mount)
                    .chain(chosen.iter().copied())
                    .filter(|&object| {
                        self.on_battlefield(object.card) && self.object_ref(object.card) == object
                    })
                    .map(|object| object.card)
                    .collect();
                if !ids.is_empty() {
                    self.blink_battlefield_objects_immediately(&ids);
                }
            }
            SaddleRiderEffect::CopyNonlegendaryAttacking => {
                if let Some(&object) = chosen.first() {
                    let item = resolution.item.clone();
                    let recipe = self.copiable_recipe(object);
                    self.stage_tapped_attacking_copy(&item, recipe);
                }
            }
            SaddleRiderEffect::SacrificeOneForPower => {
                if let Some(&object) = chosen.first() {
                    let power = self.stats(object.card).power;
                    if self.sacrifice_cost_batch_atomic(&[(controller, object.card)]) {
                        self.state.resolving.as_mut().unwrap().sacrificed_power =
                            Some(Box::new(power));
                    }
                }
            }
        }
    }

    fn saddle_eligible(&self, source: ObjectRef, p: Player, r: ObjectRef) -> bool {
        self.state.objects.get(r.card as usize).is_some_and(|o| {
            o.generation == r.generation
                && o.zone == Zone::Battlefield
                && o.controller == p
                && self.is_creature(r.card)
                && !o.tapped
                && r != source
        })
    }
    #[cfg(feature = "test-support")]
    pub(super) fn saddle_fundable(&self, source: ObjectRef, p: Player, minimum: u16) -> bool {
        if minimum == 0 {
            return true;
        }
        let mut available = 0u16;
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.saddle_eligible(source, p, r) {
                let power = self.crew_saddle_power(r.card, source.card, "Mount").max(0);
                available = available.saturating_add(u16::try_from(power).unwrap_or(u16::MAX));
                if available >= minimum {
                    return true;
                }
            }
        }
        false
    }
    fn saddle_frame_valid(&self) -> bool {
        let Some(f) = self.state.saddle_payment.as_deref() else {
            return false;
        };
        self.on_battlefield(f.source.card)
            && self.object_ref(f.source.card) == f.source
            && self.state.objects[f.source.card as usize].controller == f.actor
            && f.selected
                .iter()
                .all(|&r| self.saddle_eligible(f.source, f.actor, r))
    }
    fn saddle_sums(&self, maximum: bool) -> PowerSum {
        let f = self.state.saddle_payment.as_ref().unwrap();
        self.contributor_sum(f.source, f.actor, &f.selected, true, maximum, None)
    }
    pub fn saddle_choice(&self) -> Option<SaddleChoiceView> {
        let f = self.state.saddle_payment.as_ref()?;
        if self.semantic_actor() != Some(f.actor) {
            return None;
        }
        let sum = self.saddle_sums(false);
        Some(SaddleChoiceView {
            source: f.source,
            minimum: f.minimum,
            selected: f.selected.clone(),
            selected_power: sum.represented(),
            can_finish: !f.complete
                && self.saddle_frame_valid()
                && self.contributor_selection_fundable(&f.selected, true),
        })
    }
    pub(super) fn visit_saddle_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let f = self.state.saddle_payment.as_ref().unwrap();
        if !self.saddle_frame_valid() {
            return false;
        }
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            if !self.saddle_eligible(f.source, f.actor, object) || f.selected.contains(&object) {
                continue;
            }
            let mut selected = f.selected.clone();
            selected.push(object);
            if self.contributor_selection_fundable(&selected, false)
                && emit(Action::ChooseSaddler { object })
            {
                return true;
            }
        }
        self.contributor_selection_fundable(&f.selected, true) && emit(Action::FinishSaddle)
    }
    pub(super) fn start_saddle_payment(&mut self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            return false;
        };
        let Some(minimum) = a
            .item
            .source
            .expect("cost announcement has object source")
            .card
            .activation(ability)
            .saddle_minimum()
        else {
            return false;
        };
        assert!(a.pay_window && a.item.targets.is_empty());
        if self.state.saddle_payment.is_some() {
            return false;
        }
        self.state.saddle_payment = Some(Box::new(SaddlePayment {
            source: a.item.source.expect("Saddle source").object,
            actor: a.item.controller,
            minimum,
            selected: Vec::new(),
            complete: false,
        }));
        self.state.pending = Pending::Saddle {
            player: a.item.controller,
        };
        true
    }
    pub(super) fn finish_saddle_payment(&mut self) {
        assert!(self.saddle_frame_valid());
        let f = self.state.saddle_payment.as_ref().unwrap();
        assert!(self.contributor_selection_fundable(&f.selected, true));
        let actor = f.actor;
        let free = self
            .state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .expect("frozen Saddle cost")
            == ManaCost {
                colored: [0; 6],
                generic: 0,
            };
        if free && self.saddle_sums(false).enough(f.minimum) {
            let f = self.state.saddle_payment.take().unwrap();
            let mut a = self.state.announcing.take().unwrap();
            if let Some(phase) = self.pay_activation(&mut a.item, [0; 6], Counters::default()) {
                self.suspend_activation_payment(
                    a.item,
                    phase,
                    ActivationCostRoute::Printed,
                    activations::ActivationPaymentCompletion::Saddle(f.selected),
                );
                return;
            }
            self.finish_paid_saddle(a.item, f.selected);
            return;
        }
        self.state.saddle_payment.as_mut().unwrap().complete = true;
        self.state.pending = Pending::PayCosts { player: actor };
    }
    pub(super) fn finish_paid_saddle(&mut self, item: SpellView, selected: Vec<ObjectRef>) {
        let mount = item.source.expect("Saddle has an object source").object;
        if !selected.is_empty() {
            // CR 702.171c: the tap cost relates the two incarnations even
            // when the activation never resolves or the mount leaves play.
            let history = &mut self.saddle_history_mut(mount).contributors;
            crew_participants::remember_paid_contributors(history, &selected);
        }
        self.tap_objects_as(
            item.controller,
            &selected.iter().map(|r| r.card).collect::<Vec<_>>(),
        );
        let participants = self.capture_crew_saddle_participants(&selected);
        self.queue_crew_saddle_participant_triggers(participants, mount);
        self.state.priority = item.controller;
        self.push_stack_item(item);
        self.state.pending = Pending::Priority;
        self.state.passes = 0;
    }
    pub(super) fn resolve_saddle(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        if !self.on_battlefield(source.id) || self.object_ref(source.id) != source.object {
            return;
        }
        self.mark_saddled(source.object);
    }
    pub(super) fn mark_saddled_subject(&mut self, item: &SpellView, subject: Subject) {
        for id in self.keyword_subjects(item, subject) {
            self.mark_saddled(self.object_ref(id));
        }
    }
    fn mark_saddled(&mut self, recipient: ObjectRef) {
        if !self.on_battlefield(recipient.card) || self.object_ref(recipient.card) != recipient {
            return;
        }
        let history = self.saddle_history_mut(recipient);
        if !history.saddled {
            history.saddled = true;
            let source = self.object_view(recipient.card);
            for (index, a) in source.trigger_templates() {
                if matches!(a.condition.event(), TriggerCondition::SelfSaddledFirst) {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
}
