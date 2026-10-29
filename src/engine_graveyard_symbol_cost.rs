//! Owned non-targeting printed-symbol cost and the cards it actually exiled.
use super::*;
use crate::cards::ChoiceColor;
impl Game {
    pub(super) fn graveyard_symbol_total(&self, cards: &[ObjectRef], color: ChoiceColor) -> u128 {
        cards
            .iter()
            .map(|r| {
                u128::from(
                    self.state.objects[r.card as usize]
                        .card
                        .mana_symbols_of_color(color),
                )
            })
            .sum()
    }
    pub(super) fn graveyard_symbol_selection_complete(&self) -> bool {
        let Some(a) = &self.state.announcing else {
            return false;
        };
        let Some(AdditionalCost {
            kind: EffectChoiceKind::ExileGraveyardSymbols { color, minimum },
            ..
        }) = a.additional_cost()
        else {
            return false;
        };
        let selected = a
            .additional_group
            .as_deref()
            .map_or(&[][..], |v| v.as_slice());
        self.graveyard_symbol_total(selected, color) >= u128::from(minimum)
            && self.can_fund_symbol_cost_group(
                self.funding_goal(a),
                self.purpose_for_item(&a.item),
                a.additional_cost().unwrap(),
                a.item.object,
                selected,
            )
    }
    pub(super) fn finish_graveyard_symbol_cost(&mut self) {
        let a = self.state.announcing.as_mut().unwrap();
        a.additional = a.additional_group.as_ref().and_then(|v| v.last().copied());
        self.state.pending = Pending::PayCosts {
            player: a.item.controller,
        };
        self.resume_cast_cost_selection();
    }
    pub(super) fn copy_paid_exiled_cards_free(
        &mut self,
        item: &SpellView,
        maximum_casts: u16,
    ) -> bool {
        let definitions = item
            .exiled_symbol_cards
            .as_ref()
            .into_iter()
            .flat_map(|v| v.iter())
            .filter_map(|r| {
                (self.object_ref(r.card) == *r
                    && self.state.objects[r.card as usize].zone == Zone::Exile)
                    .then_some(self.state.objects[r.card as usize].card)
            })
            .collect::<Vec<_>>();
        if definitions.is_empty() {
            return false;
        }
        let remaining = definitions
            .into_iter()
            .map(|card| self.allocate_card_copy(card, item.controller))
            .collect();
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: None,
            casts_remaining: Some(maximum_casts),
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: false,
            player: item.controller,
            remaining,
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
        true
    }
}
