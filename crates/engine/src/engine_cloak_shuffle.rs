//! Face-down cohorts use fresh public identities across concealed movements.
use super::*;
use std::num::NonZeroU32;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn face_up_disguise_creature(&self, player: Player, object: ObjectRef) -> bool {
        self.state
            .objects
            .get(object.card as usize)
            .is_some_and(|row| {
                row.zone == Zone::Battlefield
                    && row.generation == object.generation
                    && row.controller == player
                    && !self.is_face_down(object)
                    && row.card.disguise_spec().is_some()
                    && self.is_creature(object.card)
                    && self.printed_abilities_available(object.card)
            })
    }

    pub(super) fn finish_exiling_face_up_disguise_creatures(&mut self) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        let choice = resolution.choice.take().unwrap();
        let mut ids: Vec<_> = choice.selected[player.index()]
            .iter()
            .copied()
            .filter(|&object| self.face_up_disguise_creature(player, object))
            .map(|object| object.card)
            .collect();
        if ids.is_empty() {
            return;
        }
        ids.sort_unstable();
        let sources = self.death_trigger_sources();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        let cohort: Vec<_> = events
            .iter()
            .filter(|event| event.moved && event.destination == Zone::Exile && event.was_card)
            .flat_map(ZoneChange::objects_after)
            .filter(|&object| {
                self.object_ref(object.card) == object
                    && self.state.objects[object.card as usize].zone == Zone::Exile
                    && self.state.objects[object.card as usize].is_card()
            })
            .collect();
        for &object in &cohort {
            self.remember_face_down_exile(object, 0, None);
        }
        self.state.resolving.as_mut().unwrap().remembered_objects =
            (!cohort.is_empty()).then(|| cohort.into());
        self.finish_zone_events(&events, &sources);
    }

    pub(super) fn cloak_shuffled_remembered_exile(&mut self, controller: Player) {
        let Some(cohort) = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .remembered_objects
            .take()
        else {
            return;
        };
        let mut pile: Vec<_> = cohort
            .iter()
            .copied()
            .filter(|&object| {
                self.state.objects[object.card as usize].zone == Zone::Exile
                    && self.object_ref(object.card) == object
                    && self.exiled_face_down(object)
            })
            .collect();
        self.state.rng.shuffle(&mut pile);
        self.stage_hidden_cloaks(controller, pile);
    }

    /// The caller has already concealed the exact exile cards or private hand
    /// choice. Allocate fresh slots before any entry choices can be observed.
    pub(super) fn stage_hidden_cloaks(
        &mut self,
        controller: Player,
        objects: impl IntoIterator<Item = ObjectRef>,
    ) {
        let moves: Vec<_> = objects
            .into_iter()
            .filter_map(|object| {
                let (fresh, old) = self.fresh_hidden_cloak_slot(object)?;
                let mut movement = self
                    .face_down_entry(fresh, controller, FaceDownCause::Cloak, false)
                    .expect("validated hidden cloak card");
                movement.face_down.as_mut().unwrap().obscured_origin = Some(old);
                Some(movement)
            })
            .collect();
        if moves.is_empty() {
            self.cloak_one(None, controller, false);
        } else {
            self.stage_entries(moves, None);
        }
    }

    fn fresh_hidden_cloak_slot(&mut self, original: ObjectRef) -> Option<(ObjectRef, NonZeroU32)> {
        let card = self.state.objects.get(original.card as usize)?;
        let hidden = match card.zone {
            Zone::Hand => !self.state.known_hand.contains(&original),
            Zone::Exile => self.exiled_face_down(original),
            _ => false,
        };
        if card.generation != original.generation || !card.is_card() || !hidden {
            return None;
        }
        let old = NonZeroU32::new(
            original
                .card
                .checked_add(1)
                .capacity("hidden card slot exhausted"),
        )
        .unwrap();
        let mut card = card.clone();
        // The new generation must carry no information about the old slot.
        card.generation = 0;
        let id = CardId::try_from(self.state.objects.len()).capacity("object capacity exhausted");
        let fresh = ObjectRef {
            card: id,
            generation: 0,
        };
        self.state.objects.push(card);
        self.state.objects[original.card as usize].zone = Zone::Outside;
        self.copy_face_down_exile_knowledge(original, fresh);
        Some((fresh, old))
    }

    pub(super) fn finish_hidden_cloak_identity(
        &mut self,
        old: NonZeroU32,
        fresh: ObjectRef,
        origin: Zone,
        moved: bool,
    ) {
        let original = self.object_ref(old.get() - 1);
        let restored = !moved
            && self.object_ref(fresh.card) == fresh
            && self.state.objects[fresh.card as usize].zone == origin;
        let retired = if restored {
            self.state.objects[original.card as usize].zone = origin;
            self.copy_face_down_exile_knowledge(fresh, original);
            fresh
        } else {
            original
        };
        self.reveal_hideaway_object(retired);
        // On an actual move, the old fresh exile reference remains hidden
        // through event dispatch, which then prunes the obsolete record.
        let object = &mut self.state.objects[retired.card as usize];
        object.zone = Zone::Outside;
        object.generation = object
            .generation
            .checked_add(1)
            .capacity("object generation exhausted");
        object.copy_kind = CopyKind::Card;
    }
}
