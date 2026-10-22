//! Public non-targeting proliferate selection, owned by the rare effect-choice frame.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProliferateSelection {
    pub selected_players: u8,
}
#[derive(Clone, Debug, Serialize)]
pub struct ProliferateChoiceView {
    pub candidate_players: Vec<Player>,
    pub selected_players: Vec<Player>,
}
impl Game {
    pub(super) fn start_proliferate(&mut self, item: &SpellView) {
        self.state.resolving.as_mut().unwrap().choice =
            Some(Box::new(effect_choices::EffectChoice {
                kind: EffectChoiceKind::Proliferate,
                filter: evidence::cost(0).filter,
                amount: 0,
                source: item.object,
                required: [0; 2],
                selected: std::array::from_fn(|_| Vec::new()),
                skip_to: None,
                greatest_power: [None; 2],
                discard_mana_value_life: None,
                hand_defender: None,
                hand_creature_entry: None,
                blight_copy_attacking: false,
                counter_removal: None,
                entry_life_bonus: None,
                proliferate: Some(Box::new(ProliferateSelection {
                    selected_players: 0,
                })),
            }));
        self.state.pending = Pending::Proliferate {
            player: item.controller,
        };
    }
    pub(super) fn proliferate_player_candidate(&self, player: Player) -> bool {
        self.state.players[player.index()].poison_counters > 0
            && self
                .state
                .resolving
                .as_ref()
                .and_then(|r| r.choice.as_ref())
                .and_then(|choice| choice.proliferate.as_ref())
                .is_some_and(|selection| selection.selected_players & (1 << player.index()) == 0)
    }
    pub(super) fn proliferate_choice_view(
        &self,
        choice: &effect_choices::EffectChoice,
    ) -> Option<ProliferateChoiceView> {
        let selection = choice.proliferate.as_ref()?;
        Some(ProliferateChoiceView {
            candidate_players: [Player::P0, Player::P1]
                .into_iter()
                .filter(|&p| self.proliferate_player_candidate(p))
                .collect(),
            selected_players: [Player::P0, Player::P1]
                .into_iter()
                .filter(|p| selection.selected_players & (1 << p.index()) != 0)
                .collect(),
        })
    }
    pub(super) fn choose_proliferate_permanent(&mut self, actor: Player, object: ObjectRef) {
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .as_mut()
            .unwrap()
            .selected[actor.index()]
        .push(object);
    }
    pub(super) fn choose_proliferate_player(&mut self, player: Player) {
        self.state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .as_mut()
            .unwrap()
            .proliferate
            .as_mut()
            .unwrap()
            .selected_players |= 1 << player.index();
    }
    pub(super) fn finish_proliferate(&mut self) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let player = resolution.item.controller;
        let choice = resolution.choice.take().unwrap();
        let selection = choice.proliferate.unwrap();
        // Poison is the only player counter produced by the implemented pool.
        // All permanent kinds, including sparse named and keyword counters, use
        // the same full mixed replacement event as ordinary counter placement.
        for player in [Player::P0, Player::P1] {
            if selection.selected_players & (1 << player.index()) != 0 {
                let counters = &mut self.state.players[player.index()].poison_counters;
                *counters = counters
                    .checked_add(1)
                    .capacity("player poison counters exhausted");
            }
        }
        let changes = choice.selected[player.index()]
            .iter()
            .copied()
            .filter_map(|object| {
                if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
                    return None;
                }
                let current = self.counter_amounts(object.card);
                let mut added = Counters::default();
                for kind in CounterKind::ALL {
                    if current.get(kind) > 0 {
                        added.add(kind, 1);
                    }
                }
                (!added.is_empty()).then_some((object, added))
            })
            .collect();
        self.state.pending = Pending::Priority;
        self.effect_counters(changes, replacements::ResumeMode::Program);
        self.run_program();
    }
}
