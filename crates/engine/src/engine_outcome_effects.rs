//! Resolution effects whose followups depend on legal choices or actual outcomes.
use super::*;
use crate::cards::{CostAction, CostMatch};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// The attempted destruction is prevented before a simultaneous movement.
    /// Redirected destinations remain destruction; prevented moves do not.
    pub(super) fn destroy_targets_outcome(&mut self, targets: &[BoundTarget]) -> Vec<ZoneChange> {
        self.destroy_objects_outcome(targets.iter().filter_map(|bound| {
            let (Target::Card(id), Some(object)) = (bound.target, bound.object) else {
                return None;
            };
            (object.card == id).then_some(object)
        }))
    }
    pub(super) fn destroy_objects_outcome(
        &mut self,
        objects: impl IntoIterator<Item = ObjectRef>,
    ) -> Vec<ZoneChange> {
        let mut ids = Vec::new();
        for object in objects {
            let id = object.card;
            if self.on_battlefield(id)
                && self.object_ref(id) == object
                && !ids.contains(&id)
                && !self.effect_destruction_prevented(id)
            {
                ids.push(id);
            }
        }
        let observers = self.death_trigger_sources();
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        self.finish_zone_events(&events, &observers);
        events
    }

    pub(super) fn destroy_creatures_by_power(&mut self, minimum: u16) {
        // Read all powers before any creature leaves the battlefield.
        let query = self.characteristic_query();
        let objects = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                (object.zone == Zone::Battlefield
                    && self.is_creature(id)
                    && query.stats(id).power >= i128::from(minimum))
                .then(|| self.object_ref(id))
            })
            .collect::<Vec<_>>();
        self.destroy_objects_outcome(objects);
    }

    pub(super) fn destroy_creatures_by_toughness(&mut self, maximum: i128, remember: bool) {
        // Freeze the full group before any destruction changes derived toughness.
        let query = self.characteristic_query();
        let mut ids = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                (object.zone == Zone::Battlefield
                    && self.is_creature(id)
                    && query.stats(id).toughness <= maximum)
                    .then_some(id)
            })
            .collect::<Vec<_>>();
        ids.retain(|&id| !self.effect_destruction_prevented(id));
        let observers = self.death_trigger_sources();
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        if remember {
            let mut receipts: Vec<_> = events
                .iter()
                .filter(|event| {
                    event.moved && event.was_card && event.destination == Zone::Graveyard
                })
                .flat_map(ZoneChange::objects_after)
                .filter(|object| self.state.objects[object.card as usize].card.is_creature())
                .collect();
            receipts.sort_unstable();
            self.state.resolving.as_mut().unwrap().remembered_creatures =
                (!receipts.is_empty()).then(|| receipts.into());
        }
        self.finish_zone_events(&events, &observers);
    }

    pub(super) fn opponent_sacrifice_else_discard(
        &mut self,
        item: &SpellView,
        any: &'static [CostMatch],
    ) -> bool {
        let source = item.source.expect("sacrifice effect source").object;
        let filter = evidence::cost(0).filter;
        let kind = EffectChoiceKind::FilteredCost {
            action: CostAction::Sacrifice,
            any,
        };
        let spec = AdditionalCost { kind, filter };
        let opponent = item.controller.other();
        let can_sacrifice = (0..self.state.objects.len()).any(|id| {
            self.additional_candidate(spec, opponent, source, self.object_ref(id as CardId))
        });
        self.start_effect_choice(
            item,
            if can_sacrifice {
                kind
            } else {
                EffectChoiceKind::Discard
            },
            PlayerSet::Opponent,
            filter,
            1,
            None,
        )
    }
}
