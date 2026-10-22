//! Triggered additions belong to the mana event, not its surviving providers.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ManaTapBonus {
    fixed: Mana,
    types: u8,
    amount: u32,
}
impl ManaTapBonus {
    pub(super) fn has_any(self) -> bool {
        self.fixed != [0; 6] || self.amount != 0
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TapBonusChoice {
    player: Player,
    types: u8,
    remaining: u32,
    pub previous: Option<Pending>,
}
impl Game {
    pub(super) fn capture_mana_tap_bonus(&self, source: CardId, production: Mana) -> ManaTapBonus {
        let types = production
            .iter()
            .enumerate()
            .fold(0u8, |mask, (i, n)| mask | if *n == 0 { 0 } else { 1 << i });
        ManaTapBonus {
            fixed: self.mana_tap_bonus(source),
            types,
            amount: if types == 0 {
                0
            } else {
                ability_world::CharacteristicWorld::basic_land_tap_mana_bonus(
                    &self.characteristic_query(),
                    source,
                    false,
                )
            },
        }
    }
    pub(super) fn capture_activation_mana_tap_bonus(
        &self,
        source: CardId,
        ability: u16,
        production: Mana,
    ) -> ManaTapBonus {
        if self.state.objects[source as usize].card.activations()[ability as usize]
            .cost
            .tap
        {
            self.capture_mana_tap_bonus(source, production)
        } else {
            ManaTapBonus::default()
        }
    }
    pub(super) fn finish_mana_tap_bonus(&mut self, player: Player, bonus: ManaTapBonus) {
        self.add_mana(player, bonus.fixed);
        if bonus.amount == 0 {
            return;
        }
        if bonus.types.count_ones() == 1 {
            let mut mana = [0; 6];
            mana[bonus.types.trailing_zeros() as usize] = bonus.amount;
            self.add_mana(player, mana);
            return;
        }
        let state = self.state.mana_lots.get_or_insert_with(Default::default);
        assert!(state.tap_bonus.is_none(), "one mana event at a time");
        state.tap_bonus = Some(Box::new(TapBonusChoice {
            player,
            types: bonus.types,
            remaining: bonus.amount,
            previous: None,
        }));
    }
    pub(super) fn mana_tap_bonus_pending(&self) -> bool {
        self.state
            .mana_lots
            .as_ref()
            .is_some_and(|s| s.tap_bonus.is_some())
    }
    pub(super) fn mana_tap_bonus_active(&self) -> bool {
        self.state
            .mana_lots
            .as_ref()
            .and_then(|s| s.tap_bonus.as_ref())
            .is_some_and(|c| c.previous.is_some())
    }
    /// Cost replacement and entry choices finish first. Neither SBAs nor further
    /// announcement cost choices may run until these triggered mana additions do.
    pub(super) fn advance_mana_tap_bonus(&mut self) -> bool {
        let Some(choice) = self
            .state
            .mana_lots
            .as_mut()
            .and_then(|s| s.tap_bonus.as_mut())
        else {
            return false;
        };
        if choice.previous.is_none() {
            choice.previous = Some(self.state.pending.clone());
        }
        self.state.pending = Pending::EffectMana {
            player: choice.player,
            production: ManaProduction::Choice {
                mask: choice.types,
                amount: 1,
            },
        };
        true
    }
    pub(super) fn choose_mana_tap_bonus(&mut self, production: Mana) {
        let choice = self
            .state
            .mana_lots
            .as_ref()
            .unwrap()
            .tap_bonus
            .as_ref()
            .unwrap();
        let player = choice.player;
        self.add_mana(player, production);
        let state = self.state.mana_lots.as_mut().unwrap();
        let choice = state.tap_bonus.as_mut().unwrap();
        choice.remaining -= 1;
        if choice.remaining == 0 {
            self.state.pending = state.tap_bonus.take().unwrap().previous.unwrap();
            self.compact_mana_lots();
        }
    }
}
