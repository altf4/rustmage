//! Real nonmana source quantities and mandatory, untargeted resolution counter costs.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SpellView {
    pub(super) fn source_counter_cost(&self) -> Option<CounterKind> {
        match self.kind {
            StackKind::ActivatedAbility { ability } => {
                self.source?.card.activation(ability).source_counter_cost()
            }
            _ => None,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// A compact u64 domain, independent of the mana-X grammar. Source tap and
    /// counter reservations remain owned by the enclosing announcement.
    pub fn source_counter_choice(&self) -> Option<SourceCounterChoiceView> {
        if !matches!(self.state.pending, Pending::SourceCounterAmount { .. }) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let kind = a.item.source_counter_cost()?;
        let source = a.item.source?;
        let object = self.state.objects.get(source.object.card as usize)?;
        if object.zone != Zone::Battlefield
            || self.object_ref(source.id) != source.object
            || object.controller != a.item.controller
        {
            return None;
        }
        Some(SourceCounterChoiceView {
            source: source.object,
            kind,
            maximum: self.counter_count(source.id, kind),
        })
    }
    pub(super) fn choose_source_counter_amount(&mut self, amount: u64) {
        let kind = self.source_counter_choice().unwrap().kind;
        let a = self.state.announcing.as_mut().unwrap();
        a.counter_payment = Counters::of(kind, amount);
        let loyalty_ability = matches!(a.item.kind, StackKind::ActivatedAbility { ability }
            if a.item.source.expect("activated ability has a source").card.activation(ability).cost.loyalty.is_some());
        if kind == CounterKind::Loyalty && loyalty_ability {
            a.item.loyalty_x = Some(amount);
            a.item.x = u32::try_from(amount).ok();
        }
        a.item.source_counter_payment = Some(SourceCounterPaymentView {
            kind,
            amount,
            paid: false,
        });
        self.continue_announcement_choices(false);
    }
    pub(super) fn resolution_counter_candidate(&self, player: Player, object: ObjectRef) -> bool {
        self.state
            .objects
            .get(object.card as usize)
            .is_some_and(|o| {
                o.zone == Zone::Battlefield
                    && self.object_ref(object.card) == object
                    && o.controller == player
                    && self.is_creature(object.card)
            })
    }
    pub(super) fn visit_resolution_counter_payments(
        &self,
        player: Player,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            if !self.resolution_counter_candidate(player, object) {
                continue;
            }
            for counter in CounterKind::ALL {
                if self.counter_count(object.card, counter) > 0
                    && emit(Action::ChooseCreatureCounterCost { object, counter })
                {
                    return true;
                }
            }
        }
        false
    }
    pub(super) fn pay_resolution_creature_counter(
        &mut self,
        object: ObjectRef,
        counter: CounterKind,
    ) {
        let r = self.state.resolving.as_mut().unwrap();
        assert_eq!(
            r.choice.take().unwrap().kind,
            EffectChoiceKind::RemoveCreatureCounters(1)
        );
        let player = r.item.controller;
        self.pay_counters(object.card, &Counters::of(counter, 1));
        self.draw_many(player, 1);
        // This honors ordinary miracle pauses and resumes the same owned program.
        self.run_program();
    }
}
