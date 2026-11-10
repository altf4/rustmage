//! Tap receipts retain the event for mana replacement and triggered mana abilities.
use super::*;
use crate::cards::StaticAbilityFragment;
use ability_world::CharacteristicWorld;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) enum LandAtTap {
    Basic,
    Nonbasic,
}

impl LandAtTap {
    fn from_card(card: Card) -> Self {
        if card.is_basic_land() {
            Self::Basic
        } else {
            Self::Nonbasic
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ManaTapBonus {
    pub fixed: Mana,
    /// One freely selected type actually produced by the land, per trigger.
    pub amount: u32,
    // Evaluated after paying costs, against the base mana event's output.
    #[serde(skip)]
    pub(super) colorless: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) land_at_tap: Option<LandAtTap>,
    #[serde(skip)]
    multiplier: u64,
}

impl Default for ManaTapBonus {
    fn default() -> Self {
        Self {
            fixed: [0; 6],
            amount: 0,
            colorless: 0,
            land_at_tap: None,
            multiplier: 1,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManaTapBonus {
    pub(super) fn has_any(self) -> bool {
        self.has_trigger_bonuses() || self.land_at_tap.is_some() || self.multiplier != 1
    }
    pub(super) fn has_trigger_bonuses(self) -> bool {
        self.fixed != [0; 6] || self.amount != 0 || self.colorless != 0
    }
    pub(super) fn replace_production(self, mut production: Mana) -> Option<Mana> {
        if self.multiplier != 1 {
            for amount in &mut production {
                *amount = u32::try_from(u128::from(*amount) * u128::from(self.multiplier)).ok()?;
            }
        }
        Some(production)
    }
    pub(super) fn unreplaced_capacity(self, available: Mana) -> Mana {
        if self.multiplier == 1 {
            return available;
        }
        available.map(|amount| (u64::from(amount) / self.multiplier) as u32)
    }
    pub(super) fn unreplaced_required(self, amount: u128) -> u128 {
        amount.div_ceil(u128::from(self.multiplier))
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
    fn fixed_for(self, production: Mana) -> Mana {
        let mut fixed = self.fixed;
        if production[5] != 0 {
            fixed[5] = fixed[5]
                .checked_add(self.colorless)
                .capacity("mana representation exhausted");
        }
        fixed
    }
    pub(super) fn visit(self, production: Mana, mut emit: impl FnMut(Mana) -> bool) -> bool {
        // CR106.12a: tapping for mana must actually produce mana. A captured
        // fixed-color trigger does not fire for an ability that produces zero.
        if production == [0; 6] {
            return emit([0; 6]);
        }
        let fixed = self.fixed_for(production);
        self.domain(production).visit(|mana| {
            let bonus = std::array::from_fn(|i| {
                fixed[i]
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

    pub(super) fn with_event_effects<W: CharacteristicWorld + ?Sized>(
        mut self,
        world: &W,
        actor: Player,
        source: Option<ObjectRef>,
    ) -> Self {
        let Some(land) = source else { return self };
        if world.static_sources().is_empty() {
            return self;
        }
        // CR106.12: check the providers as the mana ability produces mana,
        // after paying its activation costs. Retain a departed land's identity.
        let live =
            world.reference(land.card) == land && world.object(land.card).zone == Zone::Battlefield;
        let land_kind = if live {
            (world.kinds(land.card) & 2 != 0)
                .then(|| LandAtTap::from_card(world.definition(land.card)))
        } else {
            self.land_at_tap
        };
        let Some(land_kind) = land_kind else {
            return self;
        };
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
                .flat_map(|group| group.fragments)
            {
                let available = || {
                    query.continuous_fragment_available(source, |candidate| {
                        std::ptr::eq(candidate, fragment)
                    })
                };
                let (production, amount, colorless) = match *fragment {
                    StaticAbilityFragment::ControlledBasicLandManaMultiplier(factor)
                        if provider.controller == actor && land_kind == LandAtTap::Basic =>
                    {
                        if available() {
                            // Each replacement applies once. A saturated factor is
                            // already larger than any representable positive output.
                            self.multiplier =
                                self.multiplier.saturating_mul(u64::from(factor.max(1)));
                        }
                        continue;
                    }
                    StaticAbilityFragment::ControlledLandTapManaBonus(mana)
                        if provider.controller == actor =>
                    {
                        (Some(ManaProduction::Fixed(mana)), 0, 0)
                    }
                    StaticAbilityFragment::ControlledLandColorlessTapManaBonus
                        if provider.controller == actor =>
                    {
                        (None, 0, 1u32)
                    }
                    StaticAbilityFragment::BasicLandTapManaBonus(amount)
                        if land_kind == LandAtTap::Basic =>
                    {
                        (None, u32::from(amount), 0)
                    }
                    // CR303.4m: a departed land has no current Aura.
                    StaticAbilityFragment::AttachmentManaBonus(production)
                        if live && provider.attached_to == Some(land) =>
                    {
                        (Some(production), 0, 0)
                    }
                    _ => continue,
                };
                if !available() {
                    continue;
                }
                let count = trigger_count(&query, source);
                self.amount = self
                    .amount
                    .checked_add(
                        amount
                            .checked_mul(count)
                            .capacity("mana representation exhausted"),
                    )
                    .capacity("mana representation exhausted");
                self.colorless = self
                    .colorless
                    .checked_add(
                        colorless
                            .checked_mul(count)
                            .capacity("mana representation exhausted"),
                    )
                    .capacity("mana representation exhausted");
                let Some(production) = production else {
                    continue;
                };
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
    let object = world.object(id);
    let land_at_tap = (object.zone == Zone::Battlefield
        && object.controller == actor
        && world.kinds(id) & 2 != 0)
        .then(|| LandAtTap::from_card(world.definition(id)));
    ManaTapBonus {
        land_at_tap,
        ..ManaTapBonus::default()
    }
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
    pub(super) fn add_mana_tap_event(
        &mut self,
        actor: Player,
        receipt: ManaTapBonus,
        production: Mana,
        quality: Option<mana_lots::Quality>,
    ) {
        let replaced = receipt
            .replace_production(production)
            .capacity("mana representation exhausted");
        self.add_quality_mana(actor, replaced, quality);
        self.finish_mana_tap_bonus(actor, receipt, production);
    }

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
        self.produce_tap_mana_bonus(actor, receipt.fixed_for(production));
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
