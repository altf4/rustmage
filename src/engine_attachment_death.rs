//! Opt-in tracking for an Aura's own return after its enchanted creature dies.
//! Reuse the existing source-movement packet; no generation arithmetic or new
//! return registry, and no accidental rebinding of unrelated death observers.
use super::*;
fn follows_first_public_source_move(item: &SpellView) -> bool {
    match item.kind {
        StackKind::TriggeredAbility { ability } => item
            .trigger_definition()
            .triggers()
            .get(usize::from(ability))
            .is_some_and(|ability| {
                ability.program.iter().any(|instruction| {
                    matches!(
                        instruction,
                        Instruction::ReturnSourceFromFirstPublicZoneToHand
                    )
                })
            }),
        _ => false,
    }
}
impl Game {
    pub(super) fn bind_first_public_source_moves(&mut self, events: &[ZoneChange]) {
        fn bind(item: &mut SpellView, events: &[ZoneChange]) {
            if item.source_move.is_some() || !follows_first_public_source_move(item) {
                return;
            }
            if let Some(event) = events.iter().find(|event| {
                event.moved
                    && event.before.reference == item.object
                    && event.before.zone == Zone::Battlefield
                    && matches!(event.destination, Zone::Graveyard | Zone::Exile)
            }) {
                item.source_move = Some(Box::new(TriggerSourceMoveView {
                    before: event.before.reference,
                    after: event.after,
                    origin: event.before.zone,
                    destination: event.destination,
                }));
            }
        }
        for item in self
            .state
            .pending_triggers
            .iter_mut()
            .chain(&mut self.state.stack)
        {
            bind(item, events);
        }
        if let Some(resolution) = &mut self.state.resolving {
            bind(&mut resolution.item, events);
        }
    }
    pub(super) fn return_source_from_first_public_zone_to_hand(&mut self, item: &SpellView) {
        let (reference, zone) = item
            .source_move
            .as_ref()
            .map_or((item.object, Zone::Battlefield), |movement| {
                (movement.after, movement.destination)
            });
        if matches!(zone, Zone::Battlefield | Zone::Graveyard | Zone::Exile)
            && self.object_ref(reference.card) == reference
            && self.state.objects[reference.card as usize].zone == zone
        {
            self.move_object(reference.card, Zone::Hand);
        }
    }
}
