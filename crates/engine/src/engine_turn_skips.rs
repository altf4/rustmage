//! Deterministic coin flips and actual-turn boundaries (CR500.11/614.10).
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn flip_coins(&mut self, player: Player, coins: u16) -> u64 {
        if coins == 0 {
            return 0;
        }
        let history = &mut self.state.players[player.index()].turn_facts;
        let first = !std::mem::replace(&mut history.coin_flipped_this_turn, true);
        if first
            && self.state.static_sources.iter().any(|&id| {
                self.state.objects[id as usize].controller == player
                    && self.active_rule_fragment(id, |fragment| {
                        matches!(fragment, StaticAbilityFragment::FirstCoinFlipsHeads)
                    })
            })
        {
            return u64::from(coins);
        }
        // Both supported uses ask for a binary result: heads or a won flip.
        (0..coins).filter(|_| self.state.rng.below(2) == 1).count() as u64
    }

    pub(super) fn flip_coins_skip_turns(&mut self, item: &SpellView, coins: u16) {
        let heads = self.flip_coins(item.controller, coins);
        if heads == 0 {
            return;
        }
        for player in self.program_players(PlayerSet::Targeted, item) {
            let skips = self
                .state
                .turn_skips
                .get_or_insert_with(|| Box::new([0; 2]));
            skips[player.index()] = skips[player.index()]
                .checked_add(heads)
                .capacity("turn skip count exhausted");
        }
    }
    pub(super) fn advance_to_actual_turn(&mut self) {
        let extra = self
            .state
            .turn_schedule
            .as_ref()
            .is_some_and(|s| !s.pending.is_empty());
        let mut next = self
            .scheduled_next_turn()
            .unwrap_or_else(|| self.state.active.other());
        if extra
            && self
                .state
                .turn_schedule
                .as_ref()
                .is_some_and(|s| s.current.is_some())
        {
            if self.state.turn_skips.as_deref() == Some(&[0; 2]) {
                self.state.turn_skips = None;
            }
            self.state.active = next;
            return;
        }
        if let Some(skips) = self.state.turn_skips.as_mut() {
            // Skip whole alternating cycles in constant time, even for very large
            // accumulated counts. Skipped turns produce no turn events or numbers.
            let cycles = skips[0].min(skips[1]);
            skips[0] -= cycles;
            skips[1] -= cycles;
            if skips[next.index()] > 0 {
                skips[next.index()] -= 1;
                next = next.other();
            }
            if **skips == [0; 2] {
                self.state.turn_skips = None;
            }
        }
        self.state.active = next;
    }
}
