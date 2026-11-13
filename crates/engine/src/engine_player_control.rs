//! Controlling a player changes decisions and information, never game resources.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ControlledPlayer {
    pub player: Player,
    pub controller: Player,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ControlEffect {
    control: ControlledPlayer,
    order: u64,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct PlayerControlState {
    pending: [Option<ControlEffect>; 2],
    pending_combat: [Option<ControlEffect>; 2],
    turn: Option<ControlEffect>,
    combat: Option<ControlEffect>,
    next_order: u64,
}

impl PlayerControlState {
    fn current(&self) -> Option<ControlledPlayer> {
        self.turn
            .into_iter()
            .chain(self.combat)
            .max_by_key(|effect| effect.order)
            .map(|effect| effect.control)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct PlayerControlView {
    pub pending: [Option<Player>; 2],
    #[serde(skip_serializing_if = "no_pending_control")]
    pub pending_combat: [Option<Player>; 2],
    pub current: Option<ControlledPlayer>,
    pub semantic_actor: Option<Player>,
}

fn no_pending_control(pending: &[Option<Player>; 2]) -> bool {
    *pending == [None; 2]
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn schedule_player_control(
        &mut self,
        player: Player,
        controller: Player,
        combat_only: bool,
    ) {
        // CR 724.1a: compare creation order for effects whose durations overlap.
        let state = self.state.player_control.get_or_insert_default();
        let order = state.next_order;
        state.next_order = order
            .checked_add(1)
            .capacity("player control order exhausted");
        let effects = if combat_only {
            &mut state.pending_combat
        } else {
            &mut state.pending
        };
        effects[player.index()] = Some(ControlEffect {
            control: ControlledPlayer { player, controller },
            order,
        });
    }

    pub(super) fn begin_player_control_turn(&mut self) {
        let Some(state) = self.state.player_control.as_mut() else {
            return;
        };
        state.turn = state.pending[self.state.active.index()].take();
        state.combat = None;
        self.refresh_player_control_visibility();
    }

    pub(super) fn begin_player_control_combat(&mut self) {
        let Some(state) = self.state.player_control.as_mut() else {
            return;
        };
        if let Some(effect) = state.pending_combat[self.state.active.index()].take() {
            state.combat = Some(effect);
            self.refresh_player_control_visibility();
        }
    }

    pub(super) fn end_player_control_combat(&mut self) {
        if let Some(state) = self.state.player_control.as_mut()
            && state.combat.take().is_some()
        {
            self.refresh_player_control_visibility();
        }
    }

    fn refresh_player_control_visibility(&mut self) {
        let state = self.state.player_control.as_ref().unwrap();
        if let Some(control) = state.current() {
            self.remember_controlled_player_faces(control.player, control.controller);
        } else if state
            .pending
            .iter()
            .chain(&state.pending_combat)
            .all(Option::is_none)
        {
            self.state.player_control = None;
        }
    }

    fn current_player_control(&self) -> Option<ControlledPlayer> {
        self.state.player_control.as_ref()?.current()
    }

    pub(super) fn decision_controller(&self, player: Player) -> Player {
        self.current_player_control()
            .filter(|control| control.player == player)
            .map_or(player, |control| control.controller)
    }

    pub(super) fn shares_information(&self, viewer: Player, player: Player) -> bool {
        viewer == player || self.decision_controller(player) == viewer
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
        let controller =
            |effect: Option<ControlEffect>| effect.map(|effect| effect.control.controller);
        Some(PlayerControlView {
            pending: state.pending.map(controller),
            pending_combat: state.pending_combat.map(controller),
            current: state.current(),
            semantic_actor: self.semantic_actor(),
        })
    }
}
