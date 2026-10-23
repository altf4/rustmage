//! Public APNAP decisions precede all discards and draws of a hand refresh.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct WheelPlan {
    pub draw: u16,
    pub first: Player,
    pub answered: u8,
    pub accepted: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WheelChoiceView {
    pub source: ObjectRef,
    pub draw: u16,
    /// APNAP answers are public; none means this player has not answered yet.
    pub decisions: [Option<bool>; 2],
}
impl Game {
    pub(super) fn wheel_choice(&self) -> Option<WheelChoiceView> {
        let r = self.state.resolving.as_ref()?;
        let w = r.wheel.as_ref()?;
        Some(WheelChoiceView {
            source: r.item.object,
            draw: w.draw,
            decisions: std::array::from_fn(|index| {
                (w.answered & (1 << index) != 0).then_some(w.accepted & (1 << index) != 0)
            }),
        })
    }
    pub(super) fn start_wheel(&mut self, draw: u16) {
        let r = self.state.resolving.as_mut().unwrap();
        assert!(r.wheel.as_ref().is_none_or(|w| w.answered == 3));
        r.wheel = Some(Box::new(WheelPlan {
            draw,
            first: self.state.active,
            answered: 0,
            accepted: 0,
        }));
        self.state.pending = Pending::Wheel {
            player: self.state.active,
        };
    }
    pub(super) fn choose_wheel(&mut self, player: Player, accept: bool) {
        let next = {
            let w = self
                .state
                .resolving
                .as_mut()
                .unwrap()
                .wheel
                .as_mut()
                .unwrap();
            let bit = 1 << player.index();
            assert_eq!(w.answered & bit, 0);
            w.answered |= bit;
            if accept {
                w.accepted |= bit;
            }
            [w.first, w.first.other()]
                .into_iter()
                .find(|p| w.answered & (1 << p.index()) == 0)
        };
        if let Some(player) = next {
            self.state.pending = Pending::Wheel { player };
        } else {
            self.state.pending = Pending::Priority;
            self.run_program();
        }
    }
}
