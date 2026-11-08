//! Triggered mana receipts retain the tap event while output types are chosen later.
use super::*;
use ability_world::CharacteristicWorld;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ManaTapBonus {
    pub fixed: Mana,
    /// One freely selected type actually produced by the land, per trigger.
    pub amount: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManaTapBonus {
    pub(super) fn has_any(self) -> bool {
        self.fixed != [0; 6] || self.amount != 0
    }
    pub(super) fn domain(self, production: Mana) -> ManaProduction {
        let mask = production
            .iter()
            .enumerate()
            .fold(0, |m, (i, n)| m | if *n != 0 { 1 << i } else { 0 });
        if self.amount == 0 || mask == 0 {
            ManaProduction::Fixed([0; 6])
        } else {
            ManaProduction::Combination {
                mask,
                amount: self.amount,
            }
        }
    }
    pub(super) fn visit(self, production: Mana, mut emit: impl FnMut(Mana) -> bool) -> bool {
        self.domain(production).visit(|mana| {
            let bonus = std::array::from_fn(|i| {
                self.fixed[i]
                    .checked_add(mana[i])
                    .capacity("mana representation exhausted")
            });
            emit(bonus)
        })
    }
    pub(super) fn singleton(self, production: Mana) -> Option<Mana> {
        let mut result = None;
        let mut multiple = false;
        self.visit(production, |bonus| {
            if result.is_some() {
                multiple = true;
                return true;
            }
            result = Some(bonus);
            false
        });
        if multiple { None } else { result }
    }

    pub(super) fn with_attachment_bonuses<W: CharacteristicWorld + ?Sized>(
        mut self,
        world: &W,
        actor: Player,
        source: Option<ObjectRef>,
    ) -> Self {
        let Some(land) = source else { return self };
        // CR106.12a, 303.4m: a land sacrificed as a mana cost is no longer
        // enchanted when the paid mana ability resolves.
        if world.attachment_sources().is_empty()
            || world.reference(land.card) != land
            || world.object(land.card).zone != Zone::Battlefield
            || world.object(land.card).controller != actor
            || world.kinds(land.card) & 2 == 0
        {
            return self;
        }
        let query = characteristic_query::CharacteristicQuery::in_world(world);
        for &source in world.attachment_sources() {
            let provider = world.object(source);
            if provider.zone != Zone::Battlefield || provider.attached_to != Some(land) {
                continue;
            }
            for fragment in world
                .definition(source)
                .static_ability_groups()
                .iter()
                .flat_map(|group| group.fragments)
            {
                let crate::cards::StaticAbilityFragment::AttachmentManaBonus(production) =
                    *fragment
                else {
                    continue;
                };
                if !query.continuous_fragment_available(source, |candidate| {
                    std::ptr::eq(candidate, fragment)
                }) {
                    continue;
                }
                let count = trigger_count(&query, source);
                let mut fixed = None;
                production.in_world(world, source).visit(|mana| {
                    assert!(
                        fixed.is_none(),
                        "attachment bonus requires fixed or chosen-color mana"
                    );
                    fixed = Some(mana);
                    false
                });
                for (total, amount) in self.fixed.iter_mut().zip(fixed.unwrap_or_default()) {
                    *total = total
                        .checked_add(
                            amount
                                .checked_mul(count)
                                .capacity("mana representation exhausted"),
                        )
                        .capacity("mana representation exhausted");
                }
            }
        }
        self
    }
}

fn trigger_count<W: CharacteristicWorld + ?Sized>(
    query: &characteristic_query::CharacteristicQuery<'_, W>,
    source: CardId,
) -> u32 {
    trigger_multiplicity::additional_triggers(query, source)
        .checked_add(1)
        .and_then(|n| u32::try_from(n).ok())
        .capacity("mana trigger count exhausted")
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn capture<W: CharacteristicWorld + ?Sized>(
    world: &W,
    actor: Player,
    id: CardId,
) -> ManaTapBonus {
    let mut bonus = ManaTapBonus {
        fixed: world.controlled_land_tap_bonus(actor, id),
        amount: 0,
    };
    let land = world.object(id);
    if land.zone != Zone::Battlefield
        || land.controller != actor
        || !land.card.is_basic_land()
        || world.kinds(id) & 2 == 0
    {
        return bonus;
    }
    let query = characteristic_query::CharacteristicQuery::in_world(world);
    for &source in world.static_sources() {
        let provider = world.object(source);
        if provider.zone != Zone::Battlefield {
            continue;
        }
        for fragment in world
            .definition(source)
            .static_ability_groups()
            .iter()
            .flat_map(|g| g.fragments)
        {
            let crate::cards::StaticAbilityFragment::BasicLandTapManaBonus(amount) = *fragment
            else {
                continue;
            };
            if !query.continuous_fragment_available(source, |candidate| {
                std::ptr::eq(candidate, fragment)
            }) {
                continue;
            }
            bonus.amount = bonus
                .amount
                .checked_add(
                    u32::from(amount)
                        .checked_mul(trigger_count(&query, source))
                        .capacity("mana representation exhausted"),
                )
                .capacity("mana representation exhausted");
        }
    }
    bonus
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TapBonusChoice {
    pub player: Player,
    pub production: ManaProduction,
    pub previous: Option<Pending>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn finish_mana_tap_bonus(
        &mut self,
        actor: Player,
        receipt: ManaTapBonus,
        production: Mana,
    ) {
        if let Some(bonus) = receipt.singleton(production) {
            self.produce_tap_mana_bonus(actor, bonus);
            return;
        }
        self.produce_tap_mana_bonus(actor, receipt.fixed);
        let state = self.state.mana_lots.get_or_insert_default();
        assert!(state.tap_bonus.is_none(), "one mana event at a time");
        state.tap_bonus = Some(Box::new(TapBonusChoice {
            player: actor,
            production: receipt.domain(production),
            previous: None,
        }));
    }
    pub(super) fn mana_tap_bonus_pending(&self) -> bool {
        self.state
            .mana_lots
            .as_ref()
            .is_some_and(|s| s.tap_bonus.is_some())
    }
    pub(super) fn mana_tap_bonus_active(&self) -> bool {
        self.state
            .mana_lots
            .as_ref()
            .and_then(|s| s.tap_bonus.as_ref())
            .is_some_and(|c| c.previous.is_some())
    }
    pub(super) fn advance_mana_tap_bonus(&mut self) -> bool {
        let Some(choice) = self
            .state
            .mana_lots
            .as_mut()
            .and_then(|s| s.tap_bonus.as_mut())
        else {
            return false;
        };
        if choice.previous.is_none() {
            choice.previous = Some(self.state.pending.clone());
        }
        self.state.pending = Pending::EffectMana {
            player: choice.player,
            production: choice.production,
            recipients: std::array::from_fn(|i| i == choice.player.index()),
        };
        true
    }
    pub(super) fn choose_mana_tap_bonus(&mut self, production: Mana) {
        let choice = self
            .state
            .mana_lots
            .as_mut()
            .unwrap()
            .tap_bonus
            .take()
            .unwrap();
        self.produce_tap_mana_bonus(choice.player, production);
        self.state.pending = choice.previous.expect("active bonus choice");
        self.compact_mana_lots();
    }
}
