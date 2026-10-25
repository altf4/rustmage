//! Controlling a player changes decisions and information, never game resources.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ControlledPlayer {
    pub player: Player,
    pub controller: Player,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct PlayerControlState {
    pending: [Option<Player>; 2],
    current: Option<ControlledPlayer>,
}
#[derive(Clone, Debug, Serialize)]
pub struct PlayerControlView {
    pub pending: [Option<Player>; 2],
    pub current: Option<ControlledPlayer>,
    pub semantic_actor: Option<Player>,
}
impl Game {
    pub(super) fn schedule_player_control(&mut self, player: Player, controller: Player) {
        // CR723.1a: the latest created effect for the next actual turn wins.
        self.state.player_control.get_or_insert_default().pending[player.index()] =
            Some(controller);
    }
    pub(super) fn begin_player_control_turn(&mut self) {
        let Some(state) = self.state.player_control.as_mut() else {
            return;
        };
        state.current = state.pending[self.state.active.index()]
            .take()
            .map(|controller| ControlledPlayer {
                player: self.state.active,
                controller,
            });
        if state.current.is_none() && state.pending.iter().all(Option::is_none) {
            self.state.player_control = None;
        }
    }
    pub(super) fn decision_controller(&self, player: Player) -> Player {
        self.state
            .player_control
            .as_ref()
            .and_then(|s| s.current)
            .filter(|r| r.player == player)
            .map_or(player, |r| r.controller)
    }
    pub(super) fn shares_information(&self, viewer: Player, player: Player) -> bool {
        viewer == player
            || self
                .state
                .player_control
                .as_ref()
                .and_then(|s| s.current)
                .is_some_and(|r| r.player == player && r.controller == viewer)
    }
    pub(super) fn information_mask(&self, viewer: Player) -> u8 {
        [Player::P0, Player::P1]
            .into_iter()
            .filter(|&p| self.shares_information(viewer, p))
            .fold(0, |mask, p| mask | (1 << p.index()))
    }
    pub(super) fn choice_information_player(&self, viewer: Player) -> Player {
        self.semantic_actor()
            .filter(|&p| self.shares_information(viewer, p))
            .unwrap_or(viewer)
    }
    pub(super) fn player_control_view(&self) -> Option<PlayerControlView> {
        let state = self.state.player_control.as_ref()?;
        Some(PlayerControlView {
            pending: state.pending,
            current: state.current,
            semantic_actor: self.semantic_actor(),
        })
    }
}
