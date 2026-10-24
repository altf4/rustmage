//! Token creation completes before a distinct, replaceable counter-placement event.
use super::*;
impl Game {
    pub(super) fn create_tokens_with_counters(
        &mut self,
        item: &SpellView,
        token: Card,
        amount: u16,
        kind: CounterKind,
        counters: AmountValue,
    ) {
        let amount_counters = u64::try_from(self.item_amount(counters, item))
            .capacity("created token counter quantity exhausted");
        self.stage_token_creation(
            item.controller,
            token,
            u128::from(amount),
            false,
            replacements::TokenCompletion::Counters {
                kind,
                amount: amount_counters,
            },
        );
    }
    pub(super) fn created_token_counter_placements(
        &self,
        player: Player,
        created: &[ObjectRef],
        kind: CounterKind,
        amount: u64,
    ) -> Vec<replacements::CounterPlacement> {
        if amount == 0 {
            return Vec::new();
        }
        created
            .iter()
            .copied()
            .filter_map(|object| {
                if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
                    return None;
                }
                let o = &self.state.objects[object.card as usize];
                // A token replacement changes the creation event itself. The
                // following counter instruction applies to every resulting token.
                Some(replacements::CounterPlacement {
                    object,
                    placed_by: player,
                    card: o.card,
                    owner: o.owner,
                    controller: o.controller,
                    subtypes: self.granted_subtypes(object.card),
                    kind: self.current_kinds(object.card),
                    amount: Counters::of(kind, amount),
                    effect: true,
                    entry_index: None,
                    private: false,
                })
            })
            .collect()
    }
}
