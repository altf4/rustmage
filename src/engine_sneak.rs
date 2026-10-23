//! Sneak is an alternative spell cost, not an activated ability or a free cast.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SneakCastView {
    pub attacker: ObjectRef,
    /// A creature may remain attacking after its original planeswalker leaves.
    pub defender: Option<CombatDefender>,
}

pub(super) const RETURN_COST: AdditionalCost = AdditionalCost {
    kind: EffectChoiceKind::ReturnToHand,
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
    pub(super) fn sneak_origin_allowed(&self, player: Player, card: CardId) -> bool {
        let object = &self.state.objects[card as usize];
        match object.zone {
            Zone::Hand => object.owner == player,
            Zone::Graveyard => self.has_graveyard_cast(player, card, GraveyardCastKind::Cast),
            Zone::Exile => self.can_cast_exiled(player, card),
            Zone::Library => self.can_cast_library_top(player, card),
            _ => false,
        }
    }
    pub(super) fn sneak_attacker(
        &self,
        player: Player,
        reference: ObjectRef,
    ) -> Option<SneakCastView> {
        let object = self.state.objects.get(reference.card as usize)?;
        if object.zone != Zone::Battlefield
            || object.controller != player
            || object.generation != reference.generation
            || !self.is_creature(reference.card)
        {
            return None;
        }
        self.state.attackers.iter().find_map(|attacker| {
            (attacker.card == reference.card && !attacker.blocked).then_some(SneakCastView {
                attacker: reference,
                defender: attacker.defender,
            })
        })
    }
    pub(super) fn sneak_reservation_valid(&self, announcement: &Announcement) -> bool {
        announcement.additional_paid
            || announcement.item.cast.is_none_or(|cast| {
                cast.method != CastMethod::Sneak
                    || announcement.item.sneak.as_ref().is_some_and(|s| {
                        announcement.additional == Some(s.attacker)
                            && self
                                .sneak_attacker(announcement.item.controller, s.attacker)
                                .is_some()
                    })
            })
    }
    pub(super) fn sneak_cost_fundable(
        &self,
        player: Player,
        card: CardId,
        attacker: ObjectRef,
        cost: ManaCost,
    ) -> bool {
        let cost = self.reduced_spell_cost(player, card, cost);
        self.can_fund_for(
            FundingGoal {
                actor: player,
                required: cost.colored,
                generic: cost.generic,
                source: None,
                reserve: Some((attacker, RETURN_COST)),
            },
            self.spell_mana_purpose_for_card(player, card, self.effective_spell_card(card)),
        )
    }
    pub(super) fn visit_sneak_casts(
        &self,
        player: Player,
        card: CardId,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if !self.sneak_origin_allowed(player, card) {
            return false;
        }
        self.visit_sneak_candidates(player, card, false, &mut |attacker| {
            emit(Action::AnnounceSneak { card, attacker })
        })
    }
    pub(super) fn visit_sneak_candidates(
        &self,
        player: Player,
        card: CardId,
        flash_surcharge: bool,
        emit: &mut impl FnMut(ObjectRef) -> bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        if self.state.active != player
            || self.state.step != Step::Blockers
            || self.state.pending != Pending::Priority
            || object.is_token()
            || !object.card.is_supported()
            || object.card.sneak_costs().is_empty()
        {
            return false;
        }
        let targets = match object.card.effect() {
            Effect::Spell(plan, _) | Effect::Aura(plan) => {
                self.cast_targets_available(plan, player, self.object_ref(card))
            }
            Effect::Modal(modes) => modes
                .iter()
                .any(|m| self.cast_targets_available(m.targets, player, self.object_ref(card))),
            _ => true,
        };
        if !targets {
            return false;
        }
        for attack in &self.state.attackers {
            let attacker = self.object_ref(attack.card);
            if self.sneak_attacker(player, attacker).is_some()
                && object.card.sneak_costs().iter().any(|&cost| {
                    self.sneak_cost_fundable(
                        player,
                        card,
                        attacker,
                        Self::with_flash_surcharge(cost, object.card, flash_surcharge),
                    )
                })
                && emit(attacker)
            {
                return true;
            }
        }
        false
    }
    pub(super) fn sneak_defender_exists(&self, player: Player, defender: CombatDefender) -> bool {
        match defender {
            CombatDefender::Player(other) => other != player,
            CombatDefender::Planeswalker(reference) => self
                .state
                .objects
                .get(reference.card as usize)
                .is_some_and(|o| {
                    o.zone == Zone::Battlefield
                        && o.generation == reference.generation
                        && o.controller != player
                        && self.current_kinds(reference.card) & 64 != 0
                }),
        }
    }
}
