//! Optional exact bargain resource declaration. Funding is pure and reserves the
//! selected permanent against sacrifice-producing mana abilities.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
        self.spell_cost_option_available(actor, card, cost)
    }
    pub(super) fn visit_bargain_choices(
        &self,
        a: &Announcement,
        mut emit: impl FnMut(Option<ObjectRef>) -> bool,
    ) -> bool {
        std::iter::once(None)
            .chain((0..self.state.objects.len()).map(|id| Some(self.object_ref(id as CardId))))
            .any(|sacrifice| self.bargain_choice_fundable(a, sacrifice) && emit(sacrifice))
    }

    fn bargain_choice_fundable(&self, a: &Announcement, sacrifice: Option<ObjectRef>) -> bool {
        let Some(source) = a.item.source else { return false; };
        sacrifice.is_none_or(|object| {
            self.additional_candidate(COST, a.item.controller, source.object, object)
        })
    }

    pub(super) fn choose_bargain(&mut self, sacrifice: Option<ObjectRef>) {
        let a = self.state.announcing.as_mut().unwrap();
        a.item.cast.as_mut().unwrap().bargain = Some(sacrifice.is_some());
        a.additional = sacrifice;
        if a.item.source.unwrap().card.bargain_targets().is_some() {
            let item = a.item.clone();
            let plan = self.target_spec(&item);
            self.state.announcing.as_mut().unwrap().plan = plan;
        }
        self.continue_announcement_choices(false);
    }
}
