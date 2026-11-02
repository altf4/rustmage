//! Resolution effects whose followups depend on legal choices or actual outcomes.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// The attempted destruction is prevented before a simultaneous movement.
    /// Redirected destinations remain destruction; prevented moves do not.
    pub(super) fn destroy_targets_outcome(&mut self, targets: &[BoundTarget]) -> Vec<ZoneChange> {
        let mut ids = Vec::new();
        for &bound in targets {
            if self.same_target_object(bound)
                && let Target::Card(id) = bound.target
                && self.on_battlefield(id)
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
