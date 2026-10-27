//! Public creature-type choice and one owned, replacement-aware token batch.
use super::*;
use crate::cards::CreatureType;

#[derive(Clone, Debug, Serialize)]
pub struct CreatureTypeChoiceView {
    pub player: Player,
    pub selected: Option<CreatureType>,
}
impl Game {
    pub(super) fn visit_creature_type_tokens(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        crate::CREATURE_TYPES.iter().any(|name| {
            emit(Action::ChooseCreatureType {
                creature_type: CreatureType::from_name(name).expect("validated taxonomy"),
            })
        })
    }
    pub(super) fn choose_type_for_tokens(&mut self, creature_type: CreatureType) {
        let r = self.state.resolving.as_mut().unwrap();
        let Instruction::CreateTokensForChosenCreatureType { token } = r.program[r.pc - 1] else {
            unreachable!("creature-type token continuation")
        };
        r.chosen_creature_type = Some(creature_type);
        let player = r.item.controller;
        let count = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == player
                    && self.is_creature(*id as CardId)
                    && self.has_subtype(*id as CardId, creature_type.name())
            })
            .count() as u128;
        // pc already passed the instruction before offering the choice. Any
        // replacement/entry suspension resumes after this exact creation batch.
        self.create_tokens(player, token, count, false);
        self.run_program();
    }
    pub(super) fn creature_type_tokens_view(&self) -> Option<CreatureTypeChoiceView> {
        let r = self.state.resolving.as_ref()?;
        if !matches!(self.state.pending, Pending::CreatureTypeChoice { .. })
            && r.chosen_creature_type.is_none()
        {
            return None;
        }
        Some(CreatureTypeChoiceView {
            player: r.item.controller,
            selected: r.chosen_creature_type,
        })
    }
}
