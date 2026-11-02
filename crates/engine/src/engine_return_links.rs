//! Exact-object death returns and source-independent next-end-step returns.
use super::*;
use crate::cards::{ReturnDeathFilter, ReturnKind};

/// Flat companion in one delayed return event; groups cannot nest.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ReturnCardView {
    pub object: ObjectRef,
    pub zone: Zone,
    pub controller: Player,
    pub counter: Option<(CounterKind, u64)>,
    #[serde(skip_serializing_if = "is_false")]
    pub tapped: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReturnLinkView {
    /// Exile this exact returned permanent at the next end step.
    #[serde(skip_serializing_if = "is_false")]
    pub exile_after: bool,
    /// The first card retains the legacy singleton shape; companions share ONE trigger.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(
        clippy::box_collection,
        reason = "Cold groups retain a single pointer in legacy singleton links"
    )]
    pub additional_returning: Option<Box<Vec<ReturnCardView>>>,
    pub object: ObjectRef,
    pub zone: Zone,
    pub controller: Player,
    pub counter: Option<(CounterKind, u64)>,
    /// Instruction-level tapped entry, retained through suspended departure choices.
    #[serde(skip_serializing_if = "is_false")]
    pub tapped: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ReturnLinkView {
    fn card(&self) -> ReturnCardView {
        ReturnCardView {
            object: self.object,
            zone: self.zone,
            controller: self.controller,
            counter: self.counter,
            tapped: self.tapped,
        }
    }
    pub(super) fn companions_bytes(&self) -> usize {
        self.additional_returning.as_ref().map_or(0, |rows| {
            std::mem::size_of::<Vec<ReturnCardView>>()
                + rows.len() * std::mem::size_of::<ReturnCardView>()
        })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn is_false(value: &bool) -> bool {
    !value
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedReturnView {
    #[serde(skip_serializing_if = "is_false")]
    pub own_end_step: bool,
    pub id: u64,
    pub source: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
    pub returning: ReturnLinkView,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DelayedReturn {
    pub view: DelayedReturnView,
    pub item: SpellView,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn modification(kind: ReturnKind) -> Option<(&'static [&'static str], Keywords)> {
    match kind {
        ReturnKind::Demon => Some((&["Demon"], Keywords::NONE)),
        ReturnKind::Angel => Some((&["Angel"], Keywords::FLYING)),
        ReturnKind::HeroFinality => Some((&["Hero"], Keywords::NONE)),
        ReturnKind::Revival => None,
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
                exile_after: false,
                additional_returning: None,
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
        let Some(source) = item.source else {
            return;
        };
        let view = DelayedReturnView {
            own_end_step: false,
            id: self.next_stack_id(),
            source: source.object,
            source_card: source.card,
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
    pub(super) fn exile_small_return_until_end_step(&mut self, item: &SpellView) {
        let target = item.targets.iter().find_map(|bound| match bound.target {
            Target::Card(id) if self.same_target_object(*bound) && self.on_battlefield(id) => {
                Some(id)
            }
            _ => None,
        });
        let Some(id) = target else { return };
        let small = ability_world::CharacteristicWorld::mana_value(self, id) <= 3;
        let events = self.move_objects_raw(&[id], Zone::Exile);
        let returning = events
            .iter()
            .filter_map(|event| {
                (small
                    && event.moved
                    && event.destination == Zone::Exile
                    && !event.before.is_token())
                .then_some(ReturnLinkView {
                    exile_after: true,
                    additional_returning: None,
                    object: event.after,
                    zone: Zone::Exile,
                    controller: item.controller,
                    counter: None,
                    tapped: true,
                })
            })
            .collect();
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .immediate_blink_return = Some(Box::new(returning));
        self.finish_zone_events(&events, &[]);
    }
    /// Register only an actually successful return, after its replacement choices.
    pub(super) fn register_delayed_return_exile(&mut self, item: &SpellView, object: ObjectRef) {
        self.register_delayed_return(
            item,
            ReturnLinkView {
                exile_after: true,
                additional_returning: None,
                object,
                zone: Zone::Battlefield,
                controller: item.controller,
                counter: None,
                tapped: false,
            },
        );
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
                        exile_after: false,
                        additional_returning: None,
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
                if row.exile_after {
                    movement.delayed_return_exile = self
                        .state
                        .resolving
                        .as_ref()
                        .map(|r| Box::new(r.item.clone()));
                }
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
        let mut returning = events.iter().filter_map(|event| {
            (event.moved && event.destination == Zone::Exile && !event.before.is_token()).then_some(
                ReturnCardView {
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
            )
        });
        if let Some(first) = returning.next() {
            let companions: Vec<_> = returning.collect();
            self.register_delayed_return(
                item,
                ReturnLinkView {
                    exile_after: false,
                    object: first.object,
                    zone: first.zone,
                    controller: first.controller,
                    counter: first.counter,
                    tapped: first.tapped,
                    additional_returning: (!companions.is_empty()).then(|| Box::new(companions)),
                },
            );
        }
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn return_death_card(&mut self, item: &SpellView, kind: ReturnKind) {
        let Some(link) = &item.return_link else {
            return;
        };
        let Some(source) = item.source else {
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
                self.stage_linked_return(&link, Some((kind, source.object)));
            }
            ReturnKind::Demon | ReturnKind::Angel => {
                link.counter = Some((
                    CounterKind::PlusOne,
                    if kind == ReturnKind::Demon { 2 } else { 1 },
                ));
                self.stage_linked_return(&link, Some((kind, source.object)));
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
        let mut retained = Vec::new();
        for row in *rows {
            if row.view.own_end_step && row.view.controller != self.state.active {
                retained.push(row);
                continue;
            }
            let mut item = row.item;
            item.id = self.next_stack_id();
            item.kind = StackKind::DelayedReturn;
            item.return_link = Some(Box::new(row.view.returning));
            item.targets.clear();
            item.crime_qualifying_target = false;
            item.target_type_slots = None;
            item.mode = None;
            self.state.pending_triggers.push(item);
        }
        if !retained.is_empty() {
            self.state.delayed_returns = Some(Box::new(retained));
        }
    }
    pub(super) fn resolve_delayed_return(&mut self, item: &SpellView) {
        let Some(link) = &item.return_link else {
            return;
        };
        if link.exile_after {
            if self.on_battlefield(link.object.card)
                && self.object_ref(link.object.card) == link.object
            {
                self.move_object(link.object.card, Zone::Exile);
            }
            return;
        }
        // Validate every exact incarnation independently, then stage one entry
        // batch. No priority, ETB collection or partial entry precedes companions.
        let cards = std::iter::once(link.card()).chain(
            link.additional_returning
                .as_ref()
                .into_iter()
                .flat_map(|rows| rows.iter().copied()),
        );
        let moves = cards
            .filter_map(|card| {
                let object = self.state.objects.get(card.object.card as usize)?;
                if object.generation != card.object.generation
                    || object.zone != card.zone
                    || object.is_token()
                {
                    return None;
                }
                let mut movement =
                    self.entry_move(card.object.card, Zone::Battlefield, card.tapped);
                movement.controller = card.controller;
                movement.effect_counter = card.counter;
                Some(movement)
            })
            .collect();
        self.stage_entries(moves, None);
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
                    .map(|row| {
                        std::mem::size_of::<DelayedReturn>()
                            + row.item.owned_bytes()
                            + row.view.returning.companions_bytes()
                    })
                    .sum::<usize>()
        })
    }
}
