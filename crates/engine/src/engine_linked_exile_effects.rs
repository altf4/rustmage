//! Exact linked exile incarnations and choice continuations.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LinkedExileMode {
    ReturnCreature,
    CopyMutant,
}

fn mutant_copy(card: Card) -> CopyRecipe {
    let mut recipe = CopyRecipe::new(card);
    recipe.overrides = Some(CopyOverrides {
        legendary: Some(false),
        types: Some(CopyTypes { mutant: true, ..CopyTypes::default() }),
        ..CopyOverrides::NONE
    });
    recipe
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn linked_exile_for_item(&self, item: &SpellView) -> Vec<ObjectRef> {
        let Some(source) = item.source else { return Vec::new(); };
        self.linked_exile_for_origin(source.object, item.source_ability_origin)
    }

    fn remember_linked_exile_events(&mut self, item: &SpellView, events: &[ZoneChange]) {
        let Some(source) = item.source else { return; };
        for event in events {
            if event.was_card && event.moved && event.destination == Zone::Exile {
                self.state.linked_choices.get_or_insert_default().links.push(LinkedExileView {
                    source: source.object,
                    exiled: event.after,
                    ability_origin: item.source_ability_origin,
                });
            }
        }
    }

    pub(super) fn exile_controller_library_top_linked(&mut self, item: &SpellView, amount: u16) {
        let ids = exile::top_library_cards(self, item.controller, u128::from(amount));
        let events = self.move_objects_raw(&ids, Zone::Exile);
        self.remember_linked_exile_events(item, &events);
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn exile_targets_linked(&mut self, item: &SpellView, capture_mutant: bool) {
        let ids: Vec<_> = item.targets.iter()
            .filter(|&&target| self.same_target_object(target))
            .filter_map(|target| target.object.map(|object| object.card)).collect();
        if capture_mutant && let Some(&id) = ids.first() {
            let card = self.state.objects[id as usize].card;
            self.state.resolving.as_mut().unwrap().item.captured_artifact_copy = Some(Box::new(mutant_copy(card)));
        }
        let events = self.move_objects_raw(&ids, Zone::Exile);
        self.remember_linked_exile_events(item, &events);
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn resolution_linked_exile_candidates(&self, source: ObjectRef) -> Vec<ObjectRef> {
        let resolution = self.state.resolving.as_ref().unwrap();
        match resolution.linked_exile_mode {
            Some(LinkedExileMode::ReturnCreature) => self.linked_exile_for_item(&resolution.item)
                .into_iter().filter(|object| self.state.objects[object.card as usize].card.is_creature()).collect(),
            Some(LinkedExileMode::CopyMutant) => self.linked_exile_for_item(&resolution.item),
            None => self.linked_exile_candidates(source),
        }
    }

    pub(super) fn start_linked_exile_effect(&mut self, item: &SpellView, mode: LinkedExileMode) -> bool {
        let Some(source) = item.source else { return false; };
        self.state.resolving.as_mut().unwrap().linked_exile_mode = Some(mode);
        if self.resolution_linked_exile_candidates(source.object).is_empty() {
            self.state.resolving.as_mut().unwrap().linked_exile_mode = None;
            return false;
        }
        self.state.pending = Pending::LinkedExile { player: item.controller };
        true
    }

    pub(super) fn finish_linked_exile_effect(&mut self, mode: LinkedExileMode, object: ObjectRef) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        match mode {
            LinkedExileMode::ReturnCreature => {
                resolution.linked_exile_selected = Some(object);
                let mut movement = self.entry_move(object.card, Zone::Battlefield, false);
                movement.controller = player;
                self.stage_entries(vec![movement], None);
            }
            LinkedExileMode::CopyMutant => {
                let recipe = mutant_copy(self.state.objects[object.card as usize].card);
                self.stage_copy_creation(player, recipe, 1);
            }
        }
    }

    pub(super) fn return_other_linked_exile(&mut self, item: &SpellView) {
        let excluded = self.state.resolving.as_ref().unwrap().linked_exile_selected;
        let moves = self.linked_exile_for_item(item).into_iter()
            .filter(|object| Some(*object) != excluded)
            .map(|object| self.entry_move(object.card, Zone::Hand, false)).collect();
        self.stage_entries(moves, None);
    }

    pub(super) fn exile_opponent_until_nonland_linked(&mut self, item: &SpellView) {
        let player = item.controller.other();
        // Exile one card at a time. Stop at the first nonland that actually reached exile.
        while let Some(card) = self.top_library_card(player) {
            let events = self.move_objects_raw(&[card], Zone::Exile);
            self.remember_linked_exile_events(item, &events);
            let moved = events.iter().any(|event| event.moved);
            let nonland = events.iter().any(|event| event.moved && event.destination == Zone::Exile
                && !event.before.card.is_land());
            self.finish_zone_events(&events, &[]);
            if !moved || nonland { break; }
        }
    }

    pub(super) fn start_linked_exile_cast(&mut self, item: &SpellView) -> bool {
        let remaining = self.linked_exile_for_item(item);
        if remaining.is_empty() { return false; }
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false, paid: None, casts_remaining: Some(1), pending_cast: None,
            remaining_mana_value: None, alternative_cost: None, maximum_mana_value: None,
            origin: Zone::Exile, exile_on_graveyard: false, allow_lands: false,
            player: item.controller, remaining,
        }));
        self.state.pending = Pending::ResolutionCasts { player: item.controller };
        true
    }

    pub(super) fn return_graveyard_name_tapped(&mut self, item: &SpellView) {
        let Some(target) = item.targets.iter().find(|&&bound| self.same_target_object(bound))
            .and_then(|bound| bound.object) else { return; };
        let name = self.object_name(target.card);
        let moves = self.state.objects.iter().enumerate().filter(|(id, object)| {
            object.zone == Zone::Graveyard && object.owner == item.controller && object.is_card()
                && crate::card_names::same_name(self.object_name(*id as CardId), name)
        }).map(|(id, _)| self.entry_move(id as CardId, Zone::Battlefield, true)).collect();
        self.stage_entries(moves, None);
    }
}
