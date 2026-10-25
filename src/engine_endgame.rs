//! Live terminal restrictions and the existing turn's explicit cleanup boundary.
use super::*;
impl Game {
    pub(super) fn cant_lose_game(&self, player: Player) -> bool {
        self.state.static_sources.iter().any(|&id| {
            self.state.objects[id as usize].controller == player
                && self.active_rule_fragment(id, |f| {
                    matches!(
                        f,
                        crate::cards::StaticAbilityFragment::CantLoseGameAndOpponentsCantWin
                    )
                })
        })
    }
    pub(super) fn allowed_game_losses(&self, mut lost: [bool; 2]) -> [bool; 2] {
        for player in [Player::P0, Player::P1] {
            if lost[player.index()] && self.cant_lose_game(player) {
                lost[player.index()] = false;
            }
        }
        lost
    }
    /// Call with already frozen, permitted losses for one simultaneous event.
    pub(super) fn commit_game_losses(&mut self, lost: [bool; 2]) {
        self.state.outcome = match lost {
            [true, true] => Some(Outcome::Draw),
            [true, false] => Some(Outcome::Winner(Player::P1)),
            [false, true] => Some(Outcome::Winner(Player::P0)),
            _ => None,
        };
        if self.state.outcome.is_some() {
            self.state.pending = Pending::Terminal;
        }
    }
    pub(super) fn try_win_game(&mut self, player: Player) {
        if !self.cant_lose_game(player.other()) {
            self.state.outcome = Some(Outcome::Winner(player));
            self.state.pending = Pending::Terminal;
        }
    }
    pub(super) fn distinct_subtype_names(&self, player: Player, subtype: &str) -> usize {
        let mut names = std::collections::BTreeSet::new();
        for (id, object) in self.state.objects.iter().enumerate() {
            if object.zone == Zone::Battlefield
                && object.controller == player
                && self.has_subtype(id as CardId, subtype)
            {
                let name = self.object_name(id as CardId);
                if !name.is_empty() {
                    names.insert(crate::card_names::comparison_key(name));
                }
            }
        }
        names.len()
    }
    pub(super) fn end_turn_effect(&mut self) {
        // Preexisting waiting triggers cease. Triggers caused by this process survive.
        self.state.pending_triggers.clear();
        self.state.ordering_triggers.clear();
        self.abandon_copy_continuations();
        self.abandon_linked_choices();
        self.finish_paradigm_copy_offer();
        self.state.resolution_free_casts = None;
        // Include the currently resolving spell and any physical spell incarnation.
        let spells: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Stack)
            .map(|(id, _)| id as CardId)
            .collect();
        self.move_objects_raw(&spells, Zone::Exile);
        self.state.stack.clear();
        self.state.resolving = None;
        self.state.announcing = None;
        self.state.entry_batch = None;
        self.state.replacements = None;
        self.state.mobilize = None;
        self.state.saddle_payment = None;
        if let Some(crew) = &mut self.state.crew {
            crew.payment = None;
        }
        self.state.ending_turn = true;
        self.give_priority();
        // apply_transaction performs SBAs before permitting cleanup to begin.
    }
    /// Defer trigger stacking while a pre-cleanup SBA choice remains outstanding.
    pub(super) fn continue_ending_turn(&mut self) -> bool {
        if !self.state.ending_turn {
            return false;
        }
        if self.state.outcome.is_some() || self.state.sba_batch.is_some() {
            return true;
        }
        // Current CR724.1c-d checks SBAs before ending the current combat step.
        self.state.attackers.clear();
        self.state.blocking.clear();
        self.state.blocks.clear();
        self.state.selected_attackers.clear();
        self.state.selected_blocks.clear();
        self.state.first_strikers.clear();
        self.state.hits.clear();
        self.state.damage_choices.clear();
        self.state.attack_declared = false;
        self.state.ending_turn = false;
        self.clear_step_mana(false, true);
        self.state.step = Step::Cleanup;
        if self.above_hand_limit(self.state.active, 0) {
            self.state.pending = Pending::Discard;
            return true;
        }
        self.finish_cleanup();
        false
    }
}
