//! Optional exact bargain resource declaration. Funding is pure and reserves the
//! selected permanent against sacrifice-producing mana abilities.
use super::*;
pub(super) const COST: AdditionalCost = AdditionalCost {
    kind: EffectChoiceKind::Sacrifice,
    filter: SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 0,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: true,
        subtype: None,
    },
};
impl Game {
    pub(super) fn bargain_cost_fundable(
        &self,
        actor: Player,
        card: CardId,
        cost: ManaCost,
        sacrifice: Option<ObjectRef>,
    ) -> bool {
        if let Some(object) = sacrifice
            && !self.additional_candidate(COST, actor, self.object_ref(card), object)
        {
            return false;
        }
        let definition = self.effective_spell_card(card);
        let discount = if sacrifice.is_some() {
            definition.bargain_reduction()
        } else {
            0
        };
        let Some(cost) = self.spell_total_with_reductions(
            actor,
            card,
            cost,
            definition,
            definition.fixed_mana_value(),
            0,
            ManaCost {
                colored: [0; 6],
                generic: discount,
            },
            0,
        ) else {
            return false;
        };
        self.can_fund_for(
            FundingGoal {
                actor,
                required: cost.colored,
                generic: cost.generic,
                source: None,
                reserve: sacrifice.map(|r| (r, COST)),
            },
            self.spell_mana_purpose_for_card(actor, card, self.effective_spell_card(card)),
        )
    }
    pub(super) fn initial_bargain_fundable(
        &self,
        actor: Player,
        card: CardId,
        cost: ManaCost,
    ) -> bool {
        self.bargain_cost_fundable(actor, card, cost, None)
            || (0..self.state.objects.len()).any(|id| {
                self.bargain_cost_fundable(actor, card, cost, Some(self.object_ref(id as CardId)))
            })
    }
    pub(super) fn bargain_choice_fundable(
        &self,
        a: &Announcement,
        sacrifice: Option<ObjectRef>,
    ) -> bool {
        a.cost_options()
            .any(|cost| self.bargain_cost_fundable(a.item.controller, a.item.source.expect("cost announcement has object source").id, cost, sacrifice))
    }
}
