//! Source-independent extra turns inserted after the current turn (CR500.7).
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExtraTurnView {
    pub player: Player,
    pub power_up_prohibited: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TurnScheduleView {
    /// Creation order: the last pending row is taken first.
    pub pending: Vec<ExtraTurnView>,
    pub current: Option<ExtraTurnView>,
    pub normal_successor: Player,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(s: &Option<Box<TurnScheduleView>>) -> usize {
    s.as_ref().map_or(0, |s| {
        std::mem::size_of::<TurnScheduleView>()
            + s.pending.len() * std::mem::size_of::<ExtraTurnView>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn add_extra_turn(&mut self, player: Player, power_up_prohibited: bool) {
        self.state
            .turn_schedule
            .get_or_insert_with(|| {
                Box::new(TurnScheduleView {
                    pending: Vec::new(),
                    current: None,
                    normal_successor: self.state.active.other(),
                })
            })
            .pending
            .push(ExtraTurnView {
                player,
                power_up_prohibited,
            });
    }
    pub(super) fn scheduled_next_turn(&mut self) -> Option<Player> {
        let schedule = self.state.turn_schedule.as_mut()?;
        schedule.current = None;
        while let Some(turn) = schedule.pending.pop() {
            if let Some(skips) = self.state.turn_skips.as_mut()
                && skips[turn.player.index()] > 0
            {
                skips[turn.player.index()] -= 1;
                continue;
            }
            schedule.current = Some(turn);
            return Some(turn.player);
        }
        let next = schedule.normal_successor;
        self.state.turn_schedule = None;
        Some(next)
    }
    pub(super) fn power_up_prohibited_this_turn(&self) -> bool {
        self.state
            .turn_schedule
            .as_ref()
            .and_then(|s| s.current)
            .is_some_and(|t| t.power_up_prohibited)
    }
}
