//! Exact-object death returns and source-independent next-end-step returns.
use super::*;
use crate::cards::{ReturnDeathFilter, ReturnKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReturnLinkView {
    pub object: ObjectRef,
    pub zone: Zone,
    pub controller: Player,
    pub counter: Option<(CounterKind, u64)>,
    /// Instruction-level tapped entry, retained through suspended departure choices.
    #[serde(skip_serializing_if = "is_false")]
    pub tapped: bool,
}
fn is_false(value: &bool) -> bool {
    !value
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedReturnView {
    pub id: u64,
    pub source: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
    pub returning: ReturnLinkView,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DelayedReturn {
    pub view: DelayedReturnView,
    pub item: SpellView,
}
pub(super) fn modification(kind: ReturnKind) -> Option<(&'static [&'static str], Keywords)> {
    match kind {
        ReturnKind::Demon => Some((&["Demon"], Keywords::NONE)),
        ReturnKind::Angel => Some((&["Angel"], Keywords::FLYING)),
        ReturnKind::HeroFinality => Some((&["Hero"], Keywords::NONE)),
        ReturnKind::Revival => None,
    }
}
impl Game {
    pub(super) fn return_death_matches(
        &self,
        filter: ReturnDeathFilter,
        source: &ObjectView,
        event: &ZoneChange,
    ) -> bool {
        let before = &event.before;
        match filter {
            ReturnDeathFilter::SourceNonDemon => {
                before.is_creature()
                    && source.reference == before.reference
                    && !before.has_subtype("Demon")
            }
            ReturnDeathFilter::SourceRevival => {
                before.is_creature()
                    && source.reference == before.reference
                    && before.counters.get(CounterKind::Revival) > 0
            }
            ReturnDeathFilter::ControlledSubtype(subtype) => {
                before.controller == source.controller && before.has_subtype(subtype)
            }
            ReturnDeathFilter::ControlledNontokenNonAngel => {
                before.is_creature()
                    && before.controller == source.controller
                    && !before.is_token()
                    && !before.has_subtype("Angel")
            }
        }
    }
    pub(super) fn capture_return_death(&mut self, event: &ZoneChange) {
        self.state.pending_triggers.last_mut().unwrap().return_link =
            Some(Box::new(ReturnLinkView {
                object: event.after,
                zone: Zone::Graveyard,
                tapped: false,
                controller: event.before.owner,
                counter: Some((
                    CounterKind::Revival,
                    event.before.counters.get(CounterKind::Revival),
                )),
            }));
    }
    pub(super) fn return_event_card_to_hand(&mut self, item: &SpellView) {
        let Some(link) = &item.return_link else {
            return;
        };
        if self
            .state
            .objects
            .get(link.object.card as usize)
            .is_some_and(|object| {
                object.generation == link.object.generation
                    && object.zone == link.zone
                    && link.zone == Zone::Graveyard
                    && !object.is_token()
            })
        {
            self.move_object(link.object.card, Zone::Hand);
        }
    }
    fn register_delayed_return(&mut self, item: &SpellView, returning: ReturnLinkView) {
        let view = DelayedReturnView {
            id: self.next_stack_id(),
            source: item.object,
            source_card: item.source_card,
            controller: item.controller,
            returning,
        };
        self.state
            .delayed_returns
            .get_or_insert_with(Default::default)
            .push(DelayedReturn {
                view,
                item: item.clone(),
            });
    }
    /// Exile as one movement batch, then return only the resulting exact physical cards.
    /// Entry counters join the existing replacement event before battlefield entry.
    pub(super) fn blink_immediately(
        &mut self,
        item: &SpellView,
        counter: Option<(CounterKind, u64)>,
        tapped: bool,
    ) {
        let ids: Vec<_> = item
            .targets
            .iter()
            .filter_map(|bound| {
                let Target::Card(id) = bound.target else {
                    return None;
                };
                (self.same_target_object(*bound) && self.on_battlefield(id)).then_some(id)
            })
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let returning = events
            .iter()
            .filter_map(|event| {
                (event.moved && event.destination == Zone::Exile && !event.before.is_token())
                    .then_some(ReturnLinkView {
                        object: event.after,
                        zone: Zone::Exile,
                        controller: event.before.owner,
                        counter,
                        tapped,
                    })
            })
            .collect();
        // Departure events can themselves stage a choice-bearing linked return.
        // Keep our exact exiled incarnations owned until that entry batch finishes.
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .immediate_blink_return = Some(Box::new(returning));
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn resume_immediate_blink(&mut self) -> bool {
        let Some(rows) = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .immediate_blink_return
            .take()
        else {
            return false;
        };
        let moves = rows
            .iter()
            .filter_map(|row| {
                if self.object_ref(row.object.card) != row.object
                    || self.state.objects[row.object.card as usize].zone != row.zone
                {
                    return None;
                }
                let mut movement = self.entry_move(row.object.card, Zone::Battlefield, row.tapped);
                movement.controller = row.controller;
                movement.effect_counter = row.counter;
                Some(movement)
            })
            .collect();
        self.stage_entries(moves, None);
        true
    }
    pub(super) fn blink_until_end_step(&mut self, item: &SpellView, under_owner: bool) {
        let ids: Vec<_> = item
            .targets
            .iter()
            .filter_map(|bound| {
                let Target::Card(id) = bound.target else {
                    return None;
                };
                (self.same_target_object(*bound) && self.on_battlefield(id)).then_some(id)
            })
            .collect();
        // Freeze all recipients and their owners before ANY departure. Linked
        // returns may suspend entry choices only after the entire exile batch.
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile && !event.before.is_token() {
                self.register_delayed_return(
                    item,
                    ReturnLinkView {
                        object: event.after,
                        zone: Zone::Exile,
                        tapped: false,
                        controller: if under_owner {
                            event.before.owner
                        } else {
                            item.controller
                        },
                        counter: None,
                    },
                );
            }
        }
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn return_death_card(&mut self, item: &SpellView, kind: ReturnKind) {
        let Some(link) = &item.return_link else {
            return;
        };
        let mut link = (**link).clone();
        match kind {
            ReturnKind::Revival => {
                let amount = link.counter.expect("captured revival counters").1;
                assert!(amount > 0);
                link.counter = Some((CounterKind::Revival, amount - 1));
                link.controller = item.controller;
                self.register_delayed_return(item, link);
            }
            ReturnKind::HeroFinality => {
                link.counter = Some((CounterKind::Finality, 1));
                self.stage_linked_return(&link, Some((kind, item.object)));
            }
            ReturnKind::Demon | ReturnKind::Angel => {
                link.counter = Some((
                    CounterKind::PlusOne,
                    if kind == ReturnKind::Demon { 2 } else { 1 },
                ));
                self.stage_linked_return(&link, Some((kind, item.object)));
            }
        }
    }
    fn stage_linked_return(
        &mut self,
        link: &ReturnLinkView,
        modification: Option<(ReturnKind, ObjectRef)>,
    ) {
        if !self
            .state
            .objects
            .get(link.object.card as usize)
            .is_some_and(|o| {
                o.generation == link.object.generation && o.zone == link.zone && !o.is_token()
            })
        {
            return;
        }
        let mut movement = self.entry_move(link.object.card, Zone::Battlefield, link.tapped);
        movement.controller = link.controller;
        movement.effect_counter = link.counter;
        movement.return_modification = modification;
        self.stage_entries(vec![movement], None);
    }
    pub(super) fn queue_delayed_returns(&mut self) {
        let Some(rows) = self.state.delayed_returns.take() else {
            return;
        };
        for row in *rows {
            let mut item = row.item;
            item.id = self.next_stack_id();
            item.kind = StackKind::DelayedReturn;
            item.return_link = Some(Box::new(row.view.returning));
            item.targets.clear();
            item.mode = None;
            self.state.pending_triggers.push(item);
        }
    }
    pub(super) fn resolve_delayed_return(&mut self, item: &SpellView) {
        if let Some(link) = &item.return_link {
            self.stage_linked_return(link, None);
        }
    }
    pub(super) fn apply_return_modification(
        &mut self,
        recipient: ObjectRef,
        kind: ReturnKind,
        source: ObjectRef,
    ) {
        let Some((types, keywords)) = modification(kind) else {
            return;
        };
        let timestamp = self.next_role_timestamp();
        let state = self.state.roles.as_mut().unwrap();
        state
            .creature_types
            .push(subtypes::CreatureTypesEffectView {
                recipient,
                source,
                types,
                additional: true,
                timestamp,
            });
        if keywords != Keywords::NONE {
            state.keyword_effects.push(KeywordEffectView {
                recipient,
                origin: KeywordEffectOrigin::Resolution { source },
                keywords,
                rules: Keywords::NONE,
                timestamp,
                expires_turn: None,
            });
        }
    }
    pub(super) fn return_links_bytes(&self) -> usize {
        self.state.delayed_returns.as_ref().map_or(0, |rows| {
            std::mem::size_of_val(&**rows)
                + rows
                    .iter()
                    .map(|row| std::mem::size_of::<DelayedReturn>() + row.item.owned_bytes())
                    .sum::<usize>()
        })
    }
}
