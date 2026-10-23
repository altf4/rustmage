//! Exact-object death returns and source-independent next-end-step returns.
use super::*;
use crate::cards::{ReturnDeathFilter, ReturnKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReturnLinkView {
    pub object: ObjectRef,
    pub zone: Zone,
    pub controller: Player,
    pub counter: Option<(CounterKind, u64)>,
    pub tapped: bool,
    pub additional: Vec<ReturnCardView>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReturnCardView {
    pub object: ObjectRef,
    pub zone: Zone,
    pub controller: Player,
    pub counter: Option<(CounterKind, u64)>,
    pub tapped: bool,
}
impl ReturnLinkView {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.additional.len() * std::mem::size_of::<ReturnCardView>()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerDamageShieldView {
    pub source: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
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
impl Game {
    pub(super) fn return_death_matches(
        &self,
        filter: ReturnDeathFilter,
        source: &ObjectView,
        event: &ZoneChange,
    ) -> bool {
        let before = &event.before;
        match filter {
            ReturnDeathFilter::SourceWithoutCounter(kind) => {
                before.is_creature()
                    && source.reference == before.reference
                    && before.counters.get(kind) == 0
            }
            ReturnDeathFilter::SourceCounter { kind, minimum } => {
                before.is_creature()
                    && source.reference == before.reference
                    && before.counters.get(kind) >= minimum
            }
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
            ReturnDeathFilter::ControlledNontokenNonAngel => {
                before.is_creature()
                    && before.controller == source.controller
                    && !before.is_token()
                    && !before.has_subtype("Angel")
            }
        }
    }
    // CR603.10: both recipients and grantors use the same pre-departure world.
    pub(super) fn granted_return_death_triggers(
        &mut self,
        event: &ZoneChange,
        sources: &[ObjectView],
    ) {
        use crate::cards::StaticAbilityFragment;
        let recipient = &event.before;
        if !recipient.is_creature() {
            return;
        }
        for provider in sources {
            if provider.controller != recipient.controller
                || !provider.printed_abilities_available()
            {
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
                {
                    continue;
                }
                let index = usize::from(ability);
                let trigger = provider.card.triggers()[index];
                let TriggerCondition::ReturnDeath(filter) = trigger.condition.event() else {
                    continue;
                };
                if !trigger.condition.is_granted()
                    || !recipient.grant_available(provider.reference, provider.card, ability, None)
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
                });
                if self.queue_zone_trigger_from(recipient, index, event, Some(grant)) {
                    self.capture_return_death(event);
                }
            }
        }
    }
    pub(super) fn capture_return_death(&mut self, event: &ZoneChange) {
        self.state.pending_triggers.last_mut().unwrap().return_link =
            Some(Box::new(ReturnLinkView {
                object: event.after,
                zone: Zone::Graveyard,
                controller: event.before.owner,
                counter: Some((
                    CounterKind::Revival,
                    event.before.counters.get(CounterKind::Revival),
                )),
                tapped: false,
                additional: Vec::new(),
            }));
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
    pub(super) fn blink_immediate(&mut self, item: &SpellView) {
        let Some(id) = item
            .targets
            .iter()
            .copied()
            .filter(|bound| self.same_target_object(*bound))
            .find_map(|bound| match bound.target {
                Target::Card(id) if self.on_battlefield(id) => Some(id),
                _ => None,
            })
        else {
            return;
        };
        let owner = self.state.objects[id as usize].owner;
        let movement = self.move_object(id, Zone::Exile);
        // "That card" follows the first public destination, even if a replacement
        // redirects exile. Tokens cannot return (CR111.8). Later incarnations do not bind.
        if movement.moved
            && matches!(
                movement.destination,
                Zone::Exile | Zone::Graveyard | Zone::Command
            )
        {
            self.after_departure(programs::ProgramContinuation::Return(ReturnLinkView {
                object: movement.after,
                zone: movement.destination,
                controller: owner,
                counter: None,
                tapped: false,
                additional: Vec::new(),
            }));
        }
    }
    pub(super) fn blink_until_end_step(&mut self, item: &SpellView, under_owner: bool) {
        for bound in &item.targets {
            if !self.same_target_object(*bound) {
                continue;
            }
            let Target::Card(id) = bound.target else {
                continue;
            };
            if !self.on_battlefield(id) {
                continue;
            }
            let object = &self.state.objects[id as usize];
            let owner = object.owner;
            let movement = self.move_object(id, Zone::Exile);
            if movement.moved && movement.destination == Zone::Exile {
                self.register_delayed_return(
                    item,
                    ReturnLinkView {
                        object: movement.after,
                        zone: Zone::Exile,
                        controller: if under_owner { owner } else { item.controller },
                        counter: None,
                        tapped: false,
                        additional: Vec::new(),
                    },
                );
            }
        }
    }
    pub(super) fn return_death_card(&mut self, item: &SpellView, kind: ReturnKind) {
        let Some(link) = &item.return_link else {
            return;
        };
        let mut link = (**link).clone();
        match kind {
            ReturnKind::FlyingRobot => {
                link.counter = None;
                self.stage_linked_return(&link, Some((kind, item.object)));
            }
            ReturnKind::Persist => {
                link.counter = Some((CounterKind::MinusOne, 1));
                self.stage_linked_return(&link, None);
            }
            ReturnKind::WithoutAbilities => {
                link.counter = None;
                self.stage_linked_return(&link, Some((kind, item.object)));
            }
            ReturnKind::Revival => {
                let amount = link.counter.expect("captured revival counters").1;
                assert!(amount > 0);
                link.counter = Some((CounterKind::Revival, amount - 1));
                link.controller = item.controller;
                self.register_delayed_return(item, link);
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
    pub(super) fn stage_linked_return(
        &mut self,
        link: &ReturnLinkView,
        modification: Option<(ReturnKind, ObjectRef)>,
    ) {
        let primary = ReturnCardView {
            object: link.object,
            zone: link.zone,
            controller: link.controller,
            counter: link.counter,
            tapped: link.tapped,
        };
        let entries = std::iter::once(&primary)
            .chain(&link.additional)
            .filter_map(|card| {
                self.state
                    .objects
                    .get(card.object.card as usize)
                    .filter(|object| {
                        object.generation == card.object.generation
                            && object.zone == card.zone
                            && !object.is_token()
                    })?;
                let mut movement =
                    self.entry_move(card.object.card, Zone::Battlefield, card.tapped);
                movement.controller = card.controller;
                movement.effect_counter = card.counter;
                movement.return_modification = modification;
                Some(movement)
            })
            .collect::<Vec<_>>();
        if !entries.is_empty() {
            self.stage_entries(entries, None);
        }
    }
    pub(super) fn blink_group_tapped_until_end_step(&mut self, item: &SpellView) {
        let ids = item
            .targets
            .iter()
            .filter(|bound| self.same_target_object(**bound))
            .filter_map(|bound| match bound.target {
                Target::Card(id) if self.on_battlefield(id) => Some(id),
                _ => None,
            })
            .collect::<Vec<_>>();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let mut cards = events
            .iter()
            .filter(|event| {
                event.moved
                    && matches!(
                        event.destination,
                        Zone::Exile | Zone::Graveyard | Zone::Command
                    )
                    && !event.before.is_token()
            })
            .map(|event| ReturnCardView {
                object: event.after,
                zone: event.destination,
                controller: event.before.owner,
                counter: None,
                tapped: true,
            });
        if let Some(first) = cards.next() {
            self.register_delayed_return(
                item,
                ReturnLinkView {
                    object: first.object,
                    zone: first.zone,
                    controller: first.controller,
                    counter: None,
                    tapped: true,
                    additional: cards.collect(),
                },
            );
        }
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn prevent_player_damage_until_next_turn(&mut self, item: &SpellView) {
        self.state
            .player_damage_shields
            .get_or_insert_with(Default::default)
            .push(PlayerDamageShieldView {
                source: item.object,
                source_card: item.source_card,
                controller: item.controller,
            });
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
        let (types, keywords): (&'static [&'static str], _) = match kind {
            ReturnKind::WithoutAbilities => {
                self.remove_abilities_exact(
                    &[recipient],
                    crate::cards::AnimationDuration::Permanent,
                );
                return;
            }
            ReturnKind::FlyingRobot => {
                self.animate_exact(
                    recipient,
                    crate::cards::AnimationSpec {
                        artifact: true,
                        creature_types: crate::cards::AnimationCreatureTypes::Named(&["Robot"]),
                        base: Some((3, 3)),
                        keywords: Keywords::FLYING,
                        duration: crate::cards::AnimationDuration::Permanent,
                    },
                );
                return;
            }
            ReturnKind::Demon => (&["Demon"], Keywords::NONE),
            ReturnKind::Angel => (&["Angel"], Keywords::FLYING),
            ReturnKind::Revival | ReturnKind::Persist => return,
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
                all_creature_types: false,
                expires_turn: None,
                timestamp,
            });
        if keywords != Keywords::NONE {
            state.keyword_effects.push(KeywordEffectView {
                hexproof_colors: 0,
                expires_before_turn: None,
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
                            + row.view.returning.additional.len()
                                * std::mem::size_of::<ReturnCardView>()
                    })
                    .sum::<usize>()
        })
    }
}
