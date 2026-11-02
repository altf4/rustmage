//! Exact exile suspend grants and ordinary counterable suspend triggers (CR702.62).
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SuspendState {
    cards: Vec<ObjectRef>,
    offering: Option<ObjectRef>,
    casting: Option<(u64, ObjectRef)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(state: &Option<Box<SuspendState>>) -> usize {
    state.as_ref().map_or(0, |state| {
        std::mem::size_of::<SuspendState>() + state.cards.len() * std::mem::size_of::<ObjectRef>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn release_suspend(&mut self) {
        if self.state.suspend.as_ref().is_some_and(|state| {
            state.cards.is_empty() && state.offering.is_none() && state.casting.is_none()
        }) {
            self.state.suspend = None;
        }
    }
    fn has_suspend(&self, object: ObjectRef) -> bool {
        self.object_ref(object.card) == object
            && self.state.objects[object.card as usize].zone == Zone::Exile
            && self.state.objects[object.card as usize].is_card()
            && self
                .state
                .suspend
                .as_ref()
                .is_some_and(|state| state.cards.contains(&object))
    }
    pub(super) fn suspend_views(&self) -> Option<Vec<ObjectRef>> {
        let cards: Vec<_> = self
            .state
            .suspend
            .as_ref()?
            .cards
            .iter()
            .copied()
            .filter(|&object| self.has_suspend(object))
            .collect();
        (!cards.is_empty()).then_some(cards)
    }
    pub(super) fn forget_suspend_card(&mut self, object: ObjectRef) {
        if let Some(state) = &mut self.state.suspend {
            state.cards.retain(|&r| r != object);
        }
        self.release_suspend();
    }
    pub(super) fn suspend_captured_spell(&mut self, item: &SpellView, time: u16) {
        let Some(event) = &item.copy_event else {
            return;
        };
        let Some(source) = event.spell.source else {
            return;
        };
        // A captured departed spell remains copyable, but its new graveyard or
        // exile incarnation is never rebound by the later exile instruction.
        if !self.state.stack.iter().any(|spell| {
            spell.kind == StackKind::Spell
                && spell.id == event.spell.id
                && spell.source.is_some_and(|s| s.object == source.object)
        }) || self.object_ref(source.id) != source.object
            || self.state.objects[source.id as usize].zone != Zone::Stack
        {
            return;
        }
        let physical = self.state.objects[source.id as usize].is_card();
        let moved = self.move_object(source.id, Zone::Exile);
        if !physical || !moved.moved || moved.destination != Zone::Exile {
            return;
        }
        let state = self.state.suspend.get_or_insert_with(Default::default);
        if !state.cards.contains(&moved.after) {
            state.cards.push(moved.after);
        }
        self.effect_counter(
            source.id,
            CounterKind::Time,
            u64::from(time),
            replacements::ResumeMode::Program,
        );
    }
    fn queue_suspend_trigger(&mut self, object: ObjectRef, kind: StackKind) {
        let owner = self.state.objects[object.card as usize].owner;
        let mut item = self.activation_item(owner, object.card, 0);
        item.kind = kind;
        item.attachment_trigger = None;
        item.source_counter_lki = None;
        item.source_quantity_lki = None;
        self.state.pending_triggers.push(item);
    }
    pub(super) fn queue_suspend_upkeep(&mut self) {
        let cards: Vec<_> = self.state.suspend.as_ref().map_or_else(Vec::new, |state| {
            state
                .cards
                .iter()
                .copied()
                .filter(|&object| {
                    self.has_suspend(object)
                        && self.state.objects[object.card as usize].owner == self.state.active
                        && self.counter_count(object.card, CounterKind::Time) > 0
                })
                .collect()
        });
        for object in cards {
            self.queue_suspend_trigger(object, StackKind::SuspendTick);
        }
    }
    pub(super) fn queue_suspend_cast(&mut self, object: ObjectRef) {
        if self.has_suspend(object) {
            self.queue_suspend_trigger(object, StackKind::SuspendCast);
        }
    }
    pub(super) fn resolve_suspend_tick(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        // The upkeep's intervening-if checks suspension again on resolution.
        if self.has_suspend(source.object) && self.counter_count(source.id, CounterKind::Time) > 0 {
            self.pay_counters(source.id, &Counters::of(CounterKind::Time, 1));
        }
    }
    pub(super) fn resolve_suspend_cast(&mut self, item: &SpellView) -> bool {
        let Some(source) = item.source else {
            return false;
        };
        // The last-counter trigger only checks that the exact card is still
        // exiled; adding another time counter does not invalidate that trigger.
        if !self.has_suspend(source.object) {
            return false;
        }
        self.state.suspend.as_mut().unwrap().offering = Some(source.object);
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: None,
            casts_remaining: Some(1),
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: true,
            player: item.controller,
            remaining: vec![source.object],
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
        true
    }
    pub(super) fn remember_suspend_cast(&mut self, object: ObjectRef, stack: u64) {
        let current = self.object_ref(object.card);
        if let Some(state) = &mut self.state.suspend
            && state.offering == Some(object)
        {
            state.casting = Some((stack, current));
        }
    }
    pub(super) fn commit_suspend_cast(&mut self, stack: Option<usize>) {
        let Some(index) = stack else {
            return;
        };
        let item = &self.state.stack[index];
        let Some(source) = item.source else {
            return;
        };
        if !self
            .state
            .suspend
            .as_ref()
            .is_some_and(|state| state.casting == Some((item.id, source.object)))
        {
            return;
        }
        let controller = item.controller;
        let creature = source.card.kind_mask() & 1 != 0;
        let object = source.object;
        self.state.suspend.as_mut().unwrap().casting = None;
        if creature {
            let timestamp = self.next_role_timestamp();
            self.state
                .roles
                .as_mut()
                .unwrap()
                .keyword_effects
                .push(KeywordEffectView {
                    recipient: object,
                    origin: KeywordEffectOrigin::SuspendHaste { controller },
                    keywords: Keywords::HASTE,
                    rules: Keywords::NONE,
                    timestamp,
                    expires_turn: None,
                });
        }
    }
    pub(super) fn finish_suspend_offer(&mut self) {
        if let Some(state) = &mut self.state.suspend {
            state.offering = None;
            state.casting = None;
        }
        self.release_suspend();
    }
    pub(super) fn end_suspend_haste(&mut self, object: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state.keyword_effects.retain(|row| {
                row.recipient != object
                    || !matches!(row.origin, KeywordEffectOrigin::SuspendHaste { .. })
            });
        }
    }
}
