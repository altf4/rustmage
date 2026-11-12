//! Exact linked exile incarnations and choice continuations.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LinkedExileMode {
    ReturnCreature { plus_one: u64 },
    CopyMutant,
    CopyCreatureUntilEndOfTurn,
    CopyWhileAttached,
}

fn mutant_copy(card: Card) -> CopyRecipe {
    let mut recipe = CopyRecipe::new(card);
    recipe.overrides = Some(CopyOverrides {
        legendary: Some(false),
        types: Some(CopyTypes {
            mutant: true,
            ..CopyTypes::default()
        }),
        ..CopyOverrides::NONE
    });
    recipe
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn linked_exile_for_item(&self, item: &SpellView) -> Vec<ObjectRef> {
        let Some(source) = item.source else {
            return Vec::new();
        };
        self.linked_exile_for_cohort(
            source.object,
            item.source_ability_origin,
            item.linked_activation_cohort(),
        )
    }

    pub(super) fn link_exile_to_ability(
        &mut self,
        source: ObjectRef,
        ability_origin: Option<u64>,
        exiled: ObjectRef,
    ) {
        self.link_exile_with_cohort(source, ability_origin, None, exiled);
    }

    pub(super) fn link_exile_with_cohort(
        &mut self,
        source: ObjectRef,
        ability_origin: Option<u64>,
        acquired_link_cohort: Option<u64>,
        exiled: ObjectRef,
    ) {
        self.state
            .linked_choices
            .get_or_insert_default()
            .links
            .push(LinkedExileView {
                exiled_turn: self.state.turn,
                source,
                exiled,
                ability_origin,
                acquired_link_cohort,
            });
    }

    pub(super) fn linked_exile_colors<W: ability_world::CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        source: ObjectRef,
        origin: Option<u64>,
    ) -> u8 {
        self.state.linked_choices.as_ref().map_or(0, |choices| {
            choices.links.iter().fold(0, |colors, link| {
                let exiled = link.exiled;
                if link.source == source
                    && link.ability_origin == origin
                    && link.acquired_link_cohort.is_none()
                    && world.reference(exiled.card) == exiled
                    && world.object(exiled.card).zone == Zone::Exile
                    && world.object(exiled.card).is_card()
                    && !self.exiled_face_down(exiled)
                {
                    colors | (world.colors(exiled.card) & 31)
                } else {
                    colors
                }
            })
        })
    }

    fn remember_linked_exile_events(&mut self, item: &SpellView, events: &[ZoneChange]) {
        let Some(source) = item.source else {
            return;
        };
        for event in events {
            if event.was_card && event.moved && event.destination == Zone::Exile {
                for exiled in event.objects_after() {
                    self.link_exile_with_cohort(
                        source.object,
                        item.source_ability_origin,
                        item.linked_activation_cohort(),
                        exiled,
                    );
                }
            }
        }
    }

    pub(super) fn exile_target_library_linked(&mut self, item: &SpellView, amount: u16) {
        let ids = self
            .program_players(PlayerSet::Targeted, item)
            .flat_map(|p| exile::top_library_cards(self, p, u128::from(amount)))
            .collect::<Vec<_>>();
        self.exile_objects_linked(item, &ids);
    }

    pub(super) fn exile_objects_linked(&mut self, item: &SpellView, ids: &[CardId]) {
        let events = self.move_objects_raw(ids, Zone::Exile);
        self.remember_linked_exile_events(item, &events);
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn exile_controller_library_top_linked(
        &mut self,
        item: &SpellView,
        amount: u16,
        face_down: bool,
    ) {
        let ids = exile::top_library_cards(self, item.controller, u128::from(amount));
        let events = self.move_objects_raw(&ids, Zone::Exile);
        if face_down {
            for event in &events {
                if event.was_card && event.moved && event.destination == Zone::Exile {
                    self.remember_face_down_exile(event.after, 0, None);
                }
            }
        }
        self.remember_linked_exile_events(item, &events);
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn exile_grave_targets_linked(
        &mut self,
        item: &SpellView,
        creature_reflexive: Option<u16>,
    ) {
        let source = item
            .source
            .expect("linked exile requires an object source")
            .object;
        let ids: Vec<_> = item
            .targets
            .iter()
            .filter(|bound| self.same_target_object(**bound))
            .filter_map(|bound| bound.object)
            .filter_map(|object| {
                (self.state.objects[object.card as usize].zone == Zone::Graveyard)
                    .then_some(object.card)
            })
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.link_exile_with_cohort(
                    source,
                    item.source_ability_origin,
                    item.linked_activation_cohort(),
                    event.after,
                );
                if creature_reflexive.is_some() && event.was_card && event.before.is_creature() {
                    self.remember_borrowed_exile(item, event.after, true);
                }
            }
        }
        self.finish_zone_events(&events, &[]);
        if let Some(ability) = creature_reflexive {
            for event in events {
                if event.was_card
                    && event.moved
                    && event.destination == Zone::Exile
                    && event.before.is_creature()
                {
                    self.queue_reflexive(item, ability, Some(event.after));
                }
            }
        }
    }

    pub(super) fn exile_targets_linked(&mut self, item: &SpellView, capture_mutant: bool) {
        let ids: Vec<_> = item
            .targets
            .iter()
            .filter(|&&target| self.same_target_object(target))
            .filter_map(|target| target.object.map(|object| object.card))
            .collect();
        if capture_mutant && let Some(&id) = ids.first() {
            let card = self.state.objects[id as usize].card;
            self.state
                .resolving
                .as_mut()
                .unwrap()
                .item
                .captured_artifact_copy = Some(Box::new(mutant_copy(card)));
        }
        self.exile_objects_linked(item, &ids);
    }

    pub(super) fn resolution_linked_exile_candidates(&self, source: ObjectRef) -> Vec<ObjectRef> {
        let resolution = self.state.resolving.as_ref().unwrap();
        match self
            .state
            .linked_choices
            .as_ref()
            .and_then(|state| state.linked_exile_mode)
        {
            Some(LinkedExileMode::ReturnCreature { .. }) => self
                .linked_exile_for_item(&resolution.item)
                .into_iter()
                .filter(|object| {
                    let card = &self.state.objects[object.card as usize];
                    card.is_card() && card.card.is_creature()
                })
                .collect(),
            Some(LinkedExileMode::CopyMutant) => self.linked_exile_for_item(&resolution.item),
            Some(LinkedExileMode::CopyCreatureUntilEndOfTurn) => {
                if !self.on_battlefield(source.card) || self.object_ref(source.card) != source {
                    return Vec::new();
                }
                self.linked_exile_for_item(&resolution.item)
                    .into_iter()
                    .filter(|object| {
                        let exiled = &self.state.objects[object.card as usize];
                        exiled.is_card()
                            && exiled.card.is_creature()
                            && !self.exiled_face_down(*object)
                    })
                    .collect()
            }
            Some(LinkedExileMode::CopyWhileAttached) => {
                let item = &resolution.item;
                let Some(event) = item.attachment_trigger.as_deref() else {
                    return Vec::new();
                };
                if !self.exact_attachment_event_is_live(event) {
                    return Vec::new();
                }
                self.state
                    .exile_links
                    .iter()
                    .filter(|link| {
                        let card = &self.state.objects[link.exiled.card as usize];
                        link.source == source
                            && link.ability_origin == item.source_ability_origin
                            && self.object_ref(link.exiled.card) == link.exiled
                            && card.zone == Zone::Exile
                            && card.is_card()
                            && card.card.is_creature()
                            && !self.exiled_face_down(link.exiled)
                    })
                    .map(|link| link.exiled)
                    .collect()
            }
            None => self.linked_exile_for_item(&resolution.item),
        }
    }

    pub(super) fn start_linked_exile_effect(
        &mut self,
        item: &SpellView,
        mode: LinkedExileMode,
    ) -> bool {
        let Some(source) = item.source else {
            return false;
        };
        self.state
            .linked_choices
            .get_or_insert_default()
            .linked_exile_mode = Some(mode);
        let candidates = self.resolution_linked_exile_candidates(source.object);
        if candidates.is_empty()
            || mode == LinkedExileMode::CopyWhileAttached && candidates.len() == 1
        {
            self.state
                .linked_choices
                .as_mut()
                .unwrap()
                .linked_exile_mode = None;
            if let Some(&object) = candidates.first() {
                self.finish_linked_exile_effect(mode, object);
            }
            return false;
        }
        self.state.pending = Pending::LinkedExile {
            player: item.controller,
        };
        true
    }

    pub(super) fn finish_linked_exile_effect(&mut self, mode: LinkedExileMode, object: ObjectRef) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        match mode {
            LinkedExileMode::ReturnCreature { plus_one } => {
                self.state
                    .linked_choices
                    .as_mut()
                    .unwrap()
                    .linked_exile_selected = Some(object);
                let mut movement = self.entry_move(object.card, Zone::Battlefield, false);
                movement.controller = player;
                if plus_one != 0 {
                    movement.effect_counter = Some((CounterKind::PlusOne, plus_one));
                }
                self.stage_entries(vec![movement], None);
            }
            LinkedExileMode::CopyCreatureUntilEndOfTurn => {
                let source = resolution.item.source.expect("linked copy source").object;
                if self.on_battlefield(source.card) && self.object_ref(source.card) == source {
                    let recipe = self.copiable_recipe(object);
                    self.apply_temporary_copy(source, recipe, CopyDeadline::EndOfTurn);
                }
            }
            LinkedExileMode::CopyMutant => {
                let recipe = mutant_copy(self.state.objects[object.card as usize].card);
                self.stage_copy_creation(player, recipe, 1);
            }
            LinkedExileMode::CopyWhileAttached => {
                let event = resolution.item.attachment_trigger.as_deref().copied();
                if let Some(event) = event
                    && self.exact_attachment_event_is_live(&event)
                {
                    self.apply_temporary_copy(
                        event.recipient,
                        CopyRecipe::new(self.state.objects[object.card as usize].card),
                        CopyDeadline::WhileAttached {
                            source: event.grantor,
                            origin: event.origin.unwrap(),
                        },
                    );
                }
            }
        }
    }

    pub(super) fn return_other_linked_exile(&mut self, item: &SpellView) {
        let excluded = self
            .state
            .linked_choices
            .as_mut()
            .and_then(|state| state.linked_exile_selected.take());
        let moves = self
            .linked_exile_for_item(item)
            .into_iter()
            .filter(|object| Some(*object) != excluded)
            .map(|object| self.entry_move(object.card, Zone::Hand, false))
            .collect();
        self.stage_entries(moves, None);
    }

    pub(super) fn exile_opponent_until_nonland_linked(&mut self, item: &SpellView) {
        let player = item.controller.other();
        // Exile one card at a time. Stop at the first nonland that actually reached exile.
        while let Some(card) = self.top_library_card(player) {
            let events = self.move_objects_raw(&[card], Zone::Exile);
            self.remember_linked_exile_events(item, &events);
            let moved = events.iter().any(|event| event.moved);
            let nonland = events.iter().any(|event| {
                event.moved && event.destination == Zone::Exile && !event.before.card.is_land()
            });
            self.finish_zone_events(&events, &[]);
            if !moved || nonland {
                break;
            }
        }
    }

    pub(super) fn start_linked_exile_cast(&mut self, item: &SpellView) -> bool {
        let remaining = self.linked_exile_for_item(item);
        if remaining.is_empty() {
            return false;
        }
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            discard_instead_of_mana: false,
            permanent_type: None,
            permanent_spell_only: false,
            uncast_to_hand: false,
            siege_transformed: false,
            spell_kinds: 0,
            paid: None,
            casts_remaining: Some(1),
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: false,
            player: item.controller,
            remaining,
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
        true
    }

    pub(super) fn return_graveyard_name_tapped(&mut self, item: &SpellView) {
        let Some(target) = item
            .targets
            .iter()
            .find(|&&bound| self.same_target_object(bound))
            .and_then(|bound| bound.object)
        else {
            return;
        };
        let name = self.object_name(target.card);
        let moves = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Graveyard
                    && object.owner == item.controller
                    && object.is_card()
                    && crate::card_names::same_name(self.object_name(*id as CardId), name)
            })
            .map(|(id, _)| self.entry_move(id as CardId, Zone::Battlefield, true))
            .collect();
        self.stage_entries(moves, None);
    }
}
