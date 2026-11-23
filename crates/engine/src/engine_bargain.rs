//! Optional bargain declarations and exact provisional sacrifice shortcuts.
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
    pub(super) fn bargain_intent_available(&self, a: &Announcement, pay: bool) -> bool {
        a.item
            .source
            .is_some_and(|source| source.card.has_bargain())
            && if pay {
                self.bargain_material_may_be_available(a)
            } else {
                self.bargain_choice_fundable(a, None)
            }
    }

    fn bargain_material_may_be_available(&self, a: &Announcement) -> bool {
        let state = &self.state;
        if !state.proposal_affordability {
            return true;
        }
        let controlled = |object: &Object| {
            object.zone == Zone::Battlefield && object.controller == a.item.controller
        };
        self.state
            .objects
            .iter()
            .filter(|object| controlled(object))
            .any(|object| {
                object.is_token()
                    || object.card.kind_mask() & (16 | 32) != 0
                    || object
                        .card
                        .static_ability_groups()
                        .iter()
                        .any(|group| group.first_layer <= crate::cards::AbilityLayer::Type)
            })
            || state.roles.as_ref().is_some_and(|roles| {
                // A current form can supply an artifact or enchantment even if
                // neither type is printed. Credit it without a layered query.
                roles
                    .animations
                    .iter()
                    .any(|row| controlled(&state.objects[row.object.card as usize]))
            })
            || state.crew.as_ref().is_some_and(|crew| {
                crew.animated
                    .iter()
                    .any(|row| controlled(&state.objects[row.card as usize]))
            })
            || !payment_capacity::stable_optional_material_board(self, a)
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
        let Some(source) = a.item.source else {
            return false;
        };
        sacrifice.is_none_or(|object| {
            self.additional_candidate(COST, a.item.controller, source.object, object)
        })
    }

    pub(super) fn choose_bargain(&mut self, sacrifice: Option<ObjectRef>) {
        self.set_bargain_declaration(sacrifice.is_some(), sacrifice);
    }

    pub(super) fn declare_bargain(&mut self, pay: bool) {
        self.set_bargain_declaration(pay, None);
    }

    fn set_bargain_declaration(&mut self, pay: bool, sacrifice: Option<ObjectRef>) {
        let a = self.state.announcing.as_mut().unwrap();
        a.item.cast.as_mut().unwrap().bargain = Some(pay);
        a.additional = sacrifice;
        if a.item.source.unwrap().card.bargain_targets().is_some() {
            let item = a.item.clone();
            let plan = self.target_spec(&item);
            self.state.announcing.as_mut().unwrap().plan = plan;
        }
        self.continue_announcement_choices(false);
    }
}
