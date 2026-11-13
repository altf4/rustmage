//! Temporary allowances to play one card linked to an exact source.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LimitedExilePlay {
    pub source: ObjectRef,
    pub ability_origin: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquired_link_cohort: Option<u64>,
    pub player: Player,
    pub remaining: u32,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl LimitedExilePlay {
    fn same_ability(self, other: Self) -> bool {
        self.source == other.source
            && self.ability_origin == other.ability_origin
            && self.acquired_link_cohort == other.acquired_link_cohort
            && self.player == other.player
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn grant_linked_exile_play(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        let approval = LimitedExilePlay {
            source: source.object,
            ability_origin: item.source_ability_origin,
            acquired_link_cohort: item.linked_activation_cohort(),
            player: item.controller,
            remaining: 1,
        };
        let grants = &mut self
            .state
            .linked_choices
            .get_or_insert_default()
            .play_grants;
        if let Some(grant) = grants.iter_mut().find(|grant| grant.same_ability(approval)) {
            grant.remaining = grant
                .remaining
                .checked_add(1)
                .capacity("linked play allowance exhausted");
        } else {
            grants.push(approval);
        }
    }

    pub(super) fn linked_exile_play(
        &self,
        player: Player,
        card: CardId,
    ) -> Option<LimitedExilePlay> {
        let object = &self.state.objects[card as usize];
        if object.zone != Zone::Exile || !object.is_card() {
            return None;
        }
        let state = self.state.linked_choices.as_ref()?;
        let exiled = self.object_ref(card);
        state.play_grants.iter().copied().find(|grant| {
            grant.player == player
                && grant.remaining != 0
                && state.links.iter().any(|link| {
                    link.source == grant.source
                        && link.ability_origin == grant.ability_origin
                        && link.acquired_link_cohort == grant.acquired_link_cohort
                        && link.exiled == exiled
                })
        })
    }

    pub(super) fn prepare_linked_exile_cast(
        &self,
        player: Player,
        card: CardId,
    ) -> Option<LimitedExilePlay> {
        let approval = self.linked_exile_play(player, card)?;
        (!self.can_cast_exiled_independently(player, card)
            && !self.can_cast_prepared(player, card)
            && !self.can_play_adventure(player, card))
        .then_some(approval)
    }

    fn consume_linked_exile_play(&mut self, approval: LimitedExilePlay) {
        let grant = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .play_grants
            .iter_mut()
            .find(|grant| grant.same_ability(approval))
            .expect("approved linked play");
        grant.remaining = grant
            .remaining
            .checked_sub(1)
            .capacity("linked play allowance exhausted");
    }

    pub(super) fn commit_linked_exile_cast(&mut self, item: &SpellView) {
        let Some(state) = self.state.linked_choices.as_mut() else {
            return;
        };
        let Some(index) = state.play_approvals.iter().position(|row| row.0 == item.id) else {
            return;
        };
        let (_, approval, independent_free) = state.play_approvals.remove(index);
        if independent_free
            && item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::Free)
        {
            return;
        }
        if item.cast.is_some_and(|cast| {
            cast.exile_play_surcharge == 0
                && !matches!(
                    cast.method,
                    CastMethod::Airbend
                        | CastMethod::LinkedExileWaterbend
                        | CastMethod::ExileLife
                        | CastMethod::Plot
                        | CastMethod::Foretell
                        | CastMethod::Warp
                )
        }) {
            self.consume_linked_exile_play(approval);
        }
    }

    pub(super) fn commit_linked_exile_land(&mut self, player: Player, card: CardId) {
        if let Some(approval) = self.linked_exile_play(player, card)
            && !self.can_play_exiled_land_independently(player, card)
        {
            self.consume_linked_exile_play(approval);
        }
    }

    pub(super) fn expire_linked_exile_plays(&mut self) {
        if let Some(state) = &mut self.state.linked_choices {
            state.play_grants.clear();
        }
    }

    pub(super) fn limited_exile_plays(&self) -> Option<Vec<LimitedExilePlay>> {
        self.state
            .linked_choices
            .as_ref()
            .filter(|state| !state.play_grants.is_empty())
            .map(|state| state.play_grants.clone())
    }
}
