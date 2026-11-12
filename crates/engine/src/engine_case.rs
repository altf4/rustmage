//! Case designations bind to exact battlefield incarnations.
use super::ability_world::CharacteristicWorld;
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn is_live<W: CharacteristicWorld + ?Sized>(world: &W, object: ObjectRef) -> bool {
    world.object(object.card).zone == Zone::Battlefield && world.reference(object.card) == object
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn is_solved<W: CharacteristicWorld + ?Sized>(world: &W, object: ObjectRef) -> bool {
    world
        .game()
        .state
        .solved_cases
        .as_ref()
        .is_some_and(|objects| objects.contains(&object) && is_live(world, object))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn case_solved(&self, object: ObjectRef) -> bool {
        case::is_solved(self, object)
    }

    pub(super) fn solve_case(&mut self, object: ObjectRef, player: Player) {
        if !case::is_live(self, object) || self.case_solved(object) {
            return;
        }
        self.state
            .solved_cases
            .get_or_insert_with(|| Box::new(Vec::new()))
            .push(object);

        // A queued solve is credited to its ability controller, who can differ
        // from the current controller of the Case.
        let controller = self.state.objects[object.card as usize].controller;
        let sources = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, source)| {
                (source.zone == Zone::Battlefield
                    && source.card.triggers().iter().any(|ability| {
                        matches!(ability.condition.event(), TriggerCondition::CaseSolved(_))
                    }))
                .then_some(id as CardId)
            })
            .collect::<Vec<_>>();
        for (source, grant) in self.indexed_trigger_sources(&sources) {
            let definition = grant
                .as_ref()
                .map_or(source.card, |grant| grant.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                let TriggerCondition::CaseSolved(relation) = ability.condition.event() else {
                    continue;
                };
                let matches = match relation {
                    CastRelation::You => source.controller == player,
                    CastRelation::Opponent => source.controller != player,
                    CastRelation::Any => true,
                };
                if matches && self.queue_trigger_from(&source, index, grant.map(Box::new)) {
                    let packet = self.state.pending_triggers.last_mut().unwrap();
                    packet.event_player = Some(player);
                    packet.set_event_object(EventObjectView {
                        object,
                        facts: EventObjectFacts::Controller(controller),
                        semantic_objects: None,
                        conniver_creature: None,
                        subtype_lki: None,
                    });
                }
            }
        }
    }
    pub(super) fn forget_solved_case(&mut self, object: ObjectRef) {
        if let Some(v) = &mut self.state.solved_cases {
            v.retain(|r| *r != object);
            if v.is_empty() {
                self.state.solved_cases = None;
            }
        }
    }
    pub(super) fn grant_creature_graveyard_cast(&mut self, player: Player) {
        let cards: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Graveyard
                    && o.owner == player
                    && o.is_card()
                    && o.card.kind_mask() & 1 != 0)
                    .then_some(self.object_ref(id as CardId))
            })
            .collect();
        for card in cards {
            self.grant_graveyard_permission(player, card, GraveyardCastKind::Cast);
        }
    }
}
