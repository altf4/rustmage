//! Sneak is an alternative spell cost, not an activated ability or a free cast.
use super::*;
use crate::cards::StaticAbilityFragment;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SneakCastView {
    pub attacker: ObjectRef,
    /// A creature may remain attacking after its original planeswalker leaves.
    pub defender: Option<CombatDefender>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub granted_cost: Option<ManaCost>,
    #[serde(skip)]
    pub(super) extra: Option<Box<sneak_costs::SneakExtraCost>>,
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
                granted_cost: None,
                extra: None,
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
        flash_surcharge: bool,
    ) -> bool {
        if self.state.objects[card as usize]
            .card
            .additional_cost()
            .is_some()
        {
            let mut child = self.clone();
            let item = child.sneak_cast_item(player, card, attacker);
            // Start with a projecting frame so opening the proposal cannot recursively
            // ask for its own feasibility. All mandatory targets are still selected.
            let mut item = item;
            item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
            item.sneak
                .as_mut()
                .unwrap()
                .extra
                .as_mut()
                .unwrap()
                .projecting = true;
            child.open_payment_announcement(item, Counters::default(), None);
            let a = child.state.announcing.as_ref().unwrap();
            return a
                .cost_options()
                .any(|cost| child.sneak_extra_fundable(a, cost, None, None));
        }
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
            || (object.card.sneak_costs().is_empty()
                && self.granted_sneak_cost(player, card).is_none())
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
                && object
                    .card
                    .sneak_costs()
                    .iter()
                    .copied()
                    .chain(self.granted_sneak_cost(player, card))
                    .any(|cost| {
                        self.sneak_cost_fundable(
                            player,
                            card,
                            attacker,
                            Self::with_flash_surcharge(cost, object.card, flash_surcharge),
                            flash_surcharge,
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

impl Game {
    pub(super) fn granted_sneak_cost(&self, player: Player, card: CardId) -> Option<ManaCost> {
        let target = &self.state.objects[card as usize];
        if target.zone != Zone::Graveyard || target.owner != player || !target.card.is_creature() {
            return None;
        }
        self.state.static_sources.iter().find_map(|&source| {
            let o = &self.state.objects[source as usize];
            if o.zone != Zone::Battlefield
                || o.controller != player
                || !self.printed_abilities_available(source)
            {
                return None;
            }
            o.card
                .static_ability_groups()
                .iter()
                .flat_map(|g| g.fragments)
                .find_map(|fragment| {
                    if let StaticAbilityFragment::GrantGraveyardCreatureSneak {
                        minimum_class_level,
                        cost,
                    } = *fragment
                    {
                        (self.class_level(self.object_ref(source)) >= minimum_class_level)
                            .then_some(cost)
                    } else {
                        None
                    }
                })
        })
    }
}

impl Game {
    pub(super) fn sneak_cast_item(
        &mut self,
        player: Player,
        card: CardId,
        attacker: ObjectRef,
    ) -> SpellView {
        let mut sneak = self
            .sneak_attacker(player, attacker)
            .expect("legal Sneak attacker");
        sneak.granted_cost = self.granted_sneak_cost(player, card);
        if self.state.objects[card as usize]
            .card
            .additional_cost()
            .is_some()
        {
            sneak.extra = Some(Box::new(sneak_costs::SneakExtraCost {
                after_return: None,
                selected: None,
                paid: false,
                projecting: false,
            }));
        }
        let mut item = self.cast_item(player, card, [0; 6]);
        item.cast.as_mut().unwrap().method = CastMethod::Sneak;
        if item.source_card.x_symbols() > 0 {
            item.x = Some(0);
        }
        item.sneak = Some(Box::new(sneak));
        item
    }
}
