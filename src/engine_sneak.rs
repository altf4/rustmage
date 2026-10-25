//! CR702.190: alternative spell cost, not a hand activated ability.
use super::*;
pub(super) const COST: AdditionalCost = AdditionalCost {
    kind: EffectChoiceKind::ReturnUnblockedAttacker,
    filter: SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 1,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
};
impl Game {
    pub(super) fn sneak_attacker(&self, actor: Player, r: ObjectRef) -> bool {
        self.state.active == actor
            && self.state.step == Step::Blockers
            && self.object_ref(r.card) == r
            && self.state.objects[r.card as usize].zone == Zone::Battlefield
            && self.state.objects[r.card as usize].controller == actor
            && self
                .state
                .attackers
                .iter()
                .any(|a| a.card == r.card && !a.blocked)
    }
    pub(super) fn sneak_cast_available(&self, actor: Player, card: CardId) -> bool {
        let o = &self.state.objects[card as usize];
        if self.state.active != actor
            || self.state.step != Step::Blockers
            || o.is_token()
            || o.card.sneak_cost().is_none()
            || !(o.zone == Zone::Hand
                || self.can_play_exiled(actor, card)
                || self.can_cast_library_top(actor, card)
                || self.has_graveyard_cast(actor, card, GraveyardCastKind::Cast))
        {
            return false;
        }
        self.permitted_cast_available(
            actor,
            card,
            CastMethod::Sneak,
            CastFace::Normal,
            false,
            false,
        )
    }
}
