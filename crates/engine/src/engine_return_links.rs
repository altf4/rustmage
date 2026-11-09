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
    pub(super) fn from_event(
        event: &ZoneChange,
        controller: Player,
        counter: Option<(CounterKind, u64)>,
        tapped: bool,
        exile_after: bool,
    ) -> Self {
        let additional_returning = event.melded_after.map(|cards| {
            Box::new(
                cards
                    .into_iter()
                    .filter(|&object| object != event.after)
                    .map(|object| ReturnCardView {
                        object,
                        zone: event.destination,
                        controller,
                        counter,
                        tapped,
                    })
                    .collect(),
            )
        });
        Self {
            exile_after,
            additional_returning,
            object: event.after,
            zone: event.destination,
            controller,
            counter,
            tapped,
        }
    }
    fn cards(&self) -> impl Iterator<Item = ReturnCardView> + '_ {
        std::iter::once(self.card()).chain(
            self.additional_returning
                .iter()
                .flat_map(|rows| rows.iter().copied()),
        )
    }
    fn set_recipient_values(&mut self, controller: Player, counter: Option<(CounterKind, u64)>) {
        self.controller = controller;
        self.counter = counter;
        for card in self
            .additional_returning
            .iter_mut()
            .flat_map(|rows| rows.iter_mut())
        {
            card.controller = controller;
            card.counter = counter;
        }
    }
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ImmediateBlink {
    pub rows: Vec<ReturnLinkView>,
    pub attacking: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn modification(kind: ReturnKind) -> Option<(&'static [&'static str], Keywords)> {
    match kind {
        ReturnKind::Demon => Some((&["Demon"], Keywords::NONE)),
        ReturnKind::Angel => Some((&["Angel"], Keywords::FLYING)),
        ReturnKind::HeroFinality => Some((&["Hero"], Keywords::NONE)),
        ReturnKind::Revival
        | ReturnKind::FlyingRobot
        | ReturnKind::WithoutAbilities
        | ReturnKind::Persist => None,
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
            ReturnDeathFilter::SourceCounter { kind, minimum } => {
                source.reference == before.reference && before.counters.get(kind) >= minimum
            }
            ReturnDeathFilter::SourceNonDemon => {
                before.is_creature()
                    && source.reference == before.reference
                    && !before.has_subtype("Demon")
            }
            ReturnDeathFilter::SourceWithoutCounter(kind) => {
                before.is_creature()
                    && source.reference == before.reference
                    && before.counters.get(kind) == 0
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
    /// Both the grantor and recipient are from the same pre-departure world.
    /// Reading live providers here would lose grants during simultaneous deaths.
    pub(super) fn controlled_creature_death_triggers(
        &mut self,
        event: &ZoneChange,
        sources: &[ObjectView],
    ) {
        use crate::cards::StaticAbilityFragment;
        let recipient = &event.before;
        if !event.moved
            || recipient.zone != Zone::Battlefield
            || event.destination != Zone::Graveyard
            || !recipient.is_creature()
        {
            return;
        }
        for provider in sources {
            if provider.zone != Zone::Battlefield || provider.controller != recipient.controller {
                continue;
            }
            for fragment in provider
                .card
                .static_ability_groups()
                .iter()
                .flat_map(|group| group.fragments)
            {
                let StaticAbilityFragment::ControlledCreatureTrigger {
                    ability,
                    other,
                    nontoken,
                } = *fragment
                else {
                    continue;
                };
                if (other && provider.reference == recipient.reference)
                    || (nontoken && recipient.is_token())
                    || !recipient.grant_available(provider.reference, provider.card, ability, None)
                {
                    continue;
                }
                let index = usize::from(ability);
                let trigger = provider.card.triggers()[index];
                let TriggerCondition::ReturnDeath(filter) = trigger.condition.event() else {
                    continue;
                };
                if !trigger.condition.is_granted()
                    || !self.return_death_matches(filter, recipient, event)
                {
                    continue;
                }
                let grant = Box::new(AttachmentTriggerView {
                    grantor: provider.reference,
                    granting_card: provider.card,
                    recipient: recipient.reference,
                    defender: None,
                    granted: true,
                    origin: provider.copied_grant_origin(),
                });
                if self.queue_zone_trigger_from(recipient, index, event, Some(grant)) {
                    self.capture_return_death(event);
                }
            }
        }
    }
    pub(super) fn capture_return_death(&mut self, event: &ZoneChange) {
        self.state.pending_triggers.last_mut().unwrap().return_link =
            Some(Box::new(ReturnLinkView::from_event(
                event,
                event.before.owner,
                Some((
                    CounterKind::Revival,
                    event.before.counters.get(CounterKind::Revival),
                )),
                false,
                false,
            )));
    }
    pub(super) fn return_event_card_to_hand(&mut self, item: &SpellView) {
        let Some(link) = &item.return_link else {
            return;
        };
        let returning: Vec<_> = link
            .cards()
            .filter(|&card| card.zone == Zone::Graveyard && self.can_follow_return(card))
            .map(|card| card.object.card)
            .collect();
        let events = self.move_objects_raw(&returning, Zone::Hand);
        self.finish_zone_events(&events, &[]);
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
        let mut packet = item.clone();
        packet.targets.clear();
        packet.clear_initiating_event_references();
        self.state
            .delayed_returns
            .get_or_insert_with(Default::default)
            .push(DelayedReturn { view, item: packet });
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
                .then(|| ReturnLinkView::from_event(event, item.controller, None, true, true))
            })
            .collect();
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .immediate_blink_return = Some(Box::new(ImmediateBlink {
            rows: returning,
            attacking: false,
        }));
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
        self.blink_immediately_inner(item, counter, tapped, false, false);
    }
    pub(super) fn blink_immediately_attacking(&mut self, item: &SpellView) {
        self.blink_immediately_inner(item, None, true, true, true);
    }
    pub(super) fn blink_following_public_move(&mut self, item: &SpellView) {
        self.blink_immediately_inner(item, None, false, false, true);
    }
    fn blink_immediately_inner(
        &mut self,
        item: &SpellView,
        counter: Option<(CounterKind, u64)>,
        tapped: bool,
        attacking: bool,
        follow_public_move: bool,
    ) {
        let mut ids = Vec::new();
        for &bound in &item.targets {
            if let Target::Card(id) = bound.target
                && self.same_target_object(bound)
                && self.on_battlefield(id)
                && !ids.contains(&id)
            {
                ids.push(id);
            }
        }
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let returning = events
            .iter()
            .filter_map(|event| {
                (event.moved
                    && !event.before.is_token()
                    && (event.destination == Zone::Exile
                        || follow_public_move
                            && matches!(event.destination, Zone::Graveyard | Zone::Command)))
                .then(|| {
                    ReturnLinkView::from_event(
                        event,
                        if attacking {
                            item.controller
                        } else {
                            event.before.owner
                        },
                        counter,
                        tapped,
                        false,
                    )
                })
            })
            .collect();
        // Departure events can themselves stage a choice-bearing linked return.
        // Keep our exact exiled incarnations owned until that entry batch finishes.
        let resolution = self.state.resolving.as_mut().unwrap();
        resolution.immediate_blink_return = Some(Box::new(ImmediateBlink {
            rows: returning,
            attacking,
        }));
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn resume_immediate_blink(&mut self) -> bool {
        let resolution = self.state.resolving.as_mut().unwrap();
        let Some(blink) = resolution.immediate_blink_return.take() else {
            return false;
        };
        let attacking = blink.attacking;
        let moves = blink
            .rows
            .iter()
            .flat_map(|link| link.cards().map(|card| (card, link.exile_after)))
            .filter_map(|(row, exile_after)| {
                if !self.can_follow_return(row) {
                    return None;
                }
                let mut movement = self.entry_move(row.object.card, Zone::Battlefield, row.tapped);
                movement.controller = row.controller;
                movement.effect_counter = row.counter;
                movement.choose_attacking_defender = attacking;
                if exile_after {
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
        self.blink_group_until_end_step(item, under_owner, false);
    }
    pub(super) fn blink_group_until_end_step(
        &mut self,
        item: &SpellView,
        under_owner: bool,
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
        // Freeze all recipients and their owners before ANY departure. Linked
        // returns may suspend entry choices only after the entire exile batch.
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let mut returning = events
            .iter()
            .filter(|event| {
                event.moved
                    && matches!(
                        event.destination,
                        Zone::Exile | Zone::Graveyard | Zone::Command
                    )
                    && !event.before.is_token()
            })
            .flat_map(|event| {
                event.objects_after().map(|object| ReturnCardView {
                    object,
                    zone: event.destination,
                    tapped,
                    controller: if under_owner {
                        event.before.owner
                    } else {
                        item.controller
                    },
                    counter: None,
                })
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
        let counter = match kind {
            ReturnKind::WithoutAbilities => None,
            ReturnKind::Revival => {
                let amount = link.counter.expect("captured revival counters").1;
                assert!(amount > 0);
                link.set_recipient_values(
                    item.controller,
                    Some((CounterKind::Revival, amount - 1)),
                );
                self.register_delayed_return(item, link);
                return;
            }
            ReturnKind::Persist => Some((CounterKind::MinusOne, 1)),
            ReturnKind::HeroFinality => Some((CounterKind::Finality, 1)),
            ReturnKind::FlyingRobot => link.counter,
            ReturnKind::Demon => Some((CounterKind::PlusOne, 2)),
            ReturnKind::Angel => Some((CounterKind::PlusOne, 1)),
        };
        link.set_recipient_values(link.controller, counter);
        let modification = (kind != ReturnKind::Persist).then_some((kind, source.object));
        self.stage_linked_return(&link, modification);
    }
    fn can_follow_return(&self, card: ReturnCardView) -> bool {
        self.state
            .objects
            .get(card.object.card as usize)
            .is_some_and(|object| {
                object.generation == card.object.generation
                    && object.zone == card.zone
                    && !object.is_token()
            })
    }
    fn stage_linked_return(
        &mut self,
        link: &ReturnLinkView,
        modification: Option<(ReturnKind, ObjectRef)>,
    ) {
        let movements = link
            .cards()
            .filter(|&card| self.can_follow_return(card))
            .map(|card| {
                let mut movement =
                    self.entry_move(card.object.card, Zone::Battlefield, card.tapped);
                movement.controller = card.controller;
                movement.effect_counter = card.counter;
                movement.return_modification = modification;
                movement
            })
            .collect();
        self.stage_entries(movements, None);
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
            item.target_type_slots = None;
            item.mode = None;
            self.state.pending_triggers.push(item);
        }
        if !retained.is_empty() {
            self.state.delayed_returns = Some(Box::new(retained));
        }
    }
    pub(super) fn return_event_card_tapped_under_control(&mut self, item: &SpellView) {
        if let Some(mut link) = item.return_link.as_deref().cloned() {
            link.set_recipient_values(item.controller, None);
            self.stage_linked_return(&link, None);
        }
    }

    pub(super) fn resolve_delayed_return(&mut self, item: &SpellView) {
        let Some(link) = &item.return_link else {
            return;
        };
        if link.exile_after {
            let ids: Vec<_> = link
                .cards()
                .filter(|&card| self.can_follow_return(card))
                .map(|card| card.object.card)
                .collect();
            let events = self.move_objects_raw(&ids, Zone::Exile);
            self.finish_zone_events(&events, &[]);
        } else {
            self.stage_linked_return(link, None);
        }
    }
    pub(super) fn apply_return_modification(
        &mut self,
        recipient: ObjectRef,
        kind: ReturnKind,
        source: ObjectRef,
    ) {
        if kind == ReturnKind::WithoutAbilities {
            self.remove_abilities_exact(
                &[recipient],
                source,
                self.state.objects[recipient.card as usize].controller,
                crate::cards::AnimationDuration::Permanent,
            );
            return;
        }
        if kind == ReturnKind::FlyingRobot {
            let timestamp = self.next_role_timestamp();
            self.state.roles.as_mut().unwrap().animations.push(
                characteristic_layers::flying_robot_form(recipient, timestamp),
            );
            return;
        }
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

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerDamageShieldView {
    pub source: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn prevent_player_damage_until_next_turn(&mut self, item: &SpellView) {
        let source = item.source.expect("prevention source");
        self.state
            .roles
            .get_or_insert_default()
            .player_damage_shields
            .push(PlayerDamageShieldView {
                source: source.object,
                source_card: source.card,
                controller: item.controller,
            });
    }
}
