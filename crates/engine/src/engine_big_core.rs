//! Resolution-limited optional abilities and next-turn hand-play prohibitions.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn optional_use_key(item: &SpellView) -> TriggerUse {
        let StackKind::TriggeredAbility { ability } = item.kind else {
            panic!("optional once-per-turn use requires a triggered ability")
        };
        TriggerUse {
            source: item.source.expect("triggered ability has source").object,
            ability,
            origin: item.source_ability_origin,
        }
    }
    pub(super) fn optional_ability_used(&self, item: &SpellView) -> bool {
        self.state
            .triggered_this_turn
            .contains(&Self::optional_use_key(item))
    }
    pub(super) fn record_optional_ability_use(&mut self, item: &SpellView) {
        let key = Self::optional_use_key(item);
        if !self.state.triggered_this_turn.contains(&key) {
            self.state.triggered_this_turn.push(key);
        }
    }
    pub(super) fn hand_play_prohibited(&self) -> bool {
        self.state
            .hand_play_bans
            .as_ref()
            .is_some_and(|bans| !bans.is_empty())
    }
    pub(super) fn resolve_memory_vessel(&mut self, item: &SpellView) {
        // Until the activating player's next actual turn begins (CR614.10a).
        assert_eq!(item.controller, self.state.active);
        let mut ids = Vec::new();
        for player in [Player::P0, Player::P1] {
            ids.extend(
                self.state.players[player.index()]
                    .library
                    .iter()
                    .rev()
                    .filter(|&&id| self.state.objects[id as usize].is_card())
                    .take(7)
                    .copied(),
            );
        }
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.grant_exile_play_permission(event.before.owner, event.after, false);
                for permission in &mut self.state.exile_permissions {
                    if permission.object == event.after && permission.player == event.before.owner {
                        permission.expiry = ExilePlayExpiry::StartOfNextTurn(item.controller);
                    }
                }
            }
        }
        self.state
            .hand_play_bans
            .get_or_insert_default()
            .push(item.controller);
        self.finish_zone_events(&events, &[]);
    }
}
