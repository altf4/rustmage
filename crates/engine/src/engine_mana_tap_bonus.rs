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
    /// One freely selected type actually produced by the permanent, per trigger.
    pub amount: u32,
    // Evaluated after paying costs, against the base mana event's output.
    #[serde(skip)]
    pub(super) colorless: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) land_at_tap: Option<LandAtTap>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub(super) artifact_token_at_tap: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub(super) creature_at_tap: bool,
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
            artifact_token_at_tap: false,
            creature_at_tap: false,
            multiplier: 1,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManaTapBonus {
    pub(super) fn has_any(self) -> bool {
        self.has_trigger_bonuses()
            || self.land_at_tap.is_some()
            || self.artifact_token_at_tap
            || self.creature_at_tap
            || self.multiplier != 1
    }
    pub(super) fn has_trigger_bonuses(self) -> bool {
        self.fixed != [0; 6] || self.amount != 0 || self.colorless != 0
    }
    pub(super) fn with_event_effects<W: CharacteristicWorld + ?Sized>(
        self,
        world: &W,
        actor: Player,
        source: Option<ObjectRef>,
    ) -> ManaTapReceipt {
        let mut receipt = ManaTapReceipt {
            base: self,
            ..ManaTapReceipt::default()
        };
        let Some(permanent) = source else {
            return receipt;
        };
        if world.static_sources().is_empty() {
            return receipt;
        }
        // CR106.12: providers must be active when the ability produces mana.
        // A source that departed while paying costs keeps its tap-time identity.
        let object = world.object(permanent.card);
        let live = world.reference(permanent.card) == permanent && object.zone == Zone::Battlefield;
        let kinds = live.then(|| world.kinds(permanent.card));
        let land_kind = kinds.map_or(self.land_at_tap, |kinds| {
            (kinds & 2 != 0).then(|| LandAtTap::from_card(world.definition(permanent.card)))
        });
        let artifact_token = kinds.map_or(self.artifact_token_at_tap, |kinds| {
            kinds & 16 != 0 && object.is_token()
        });
        let creature = kinds.map_or(self.creature_at_tap, |kinds| kinds & 1 != 0);
        if land_kind.is_none() && !artifact_token && !creature {
            return receipt;
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
                .flat_map(|group| group.fragments)
            {
                let available = || {
                    query.continuous_fragment_available(source, |candidate| {
                        std::ptr::eq(candidate, fragment)
                    })
                };
                let (production, amount, colorless) = match *fragment {
                    StaticAbilityFragment::ControlledBasicLandManaMultiplier(factor)
                        if provider.controller == actor && land_kind == Some(LandAtTap::Basic) =>
                    {
                        if available() {
                            // Each replacement applies once. A saturated factor is
                            // already larger than any representable positive output.
                            receipt.base.multiplier = receipt
                                .base
                                .multiplier
                                .saturating_mul(u64::from(factor.max(1)));
                        }
                        continue;
                    }
                    StaticAbilityFragment::ControlledLandTapManaBonus(mana)
                        if provider.controller == actor && land_kind.is_some() =>
                    {
                        (Some(ManaProduction::Fixed(mana)), 0, 0)
                    }
                    StaticAbilityFragment::ControlledCreatureTapManaBonus(mana)
                        if provider.controller == actor && creature =>
                    {
                        (Some(ManaProduction::Fixed(mana)), 0, 0)
                    }
                    StaticAbilityFragment::ControlledLandColorlessTapManaBonus
                        if provider.controller == actor && land_kind.is_some() =>
                    {
                        (None, 0, 1u32)
                    }
                    StaticAbilityFragment::ControlledArtifactTokenTapManaBonus
                        if provider.controller == actor && artifact_token =>
                    {
                        (None, 1u32, 0)
                    }
                    StaticAbilityFragment::BasicLandTapManaBonus(amount)
                        if land_kind == Some(LandAtTap::Basic) =>
                    {
                        (None, u32::from(amount), 0)
                    }
                    // CR303.4m: a departed land has no current Aura.
                    StaticAbilityFragment::AttachmentManaBonus(production)
                        if live
                            && land_kind.is_some()
                            && provider.attached_to == Some(permanent) =>
                    {
                        (Some(production), 0, 0)
                    }
                    _ => continue,
                };
                if !available() {
                    continue;
                }
                let count = trigger_count(&query, source);
                let quality = mana_lots::Quality::ORDINARY.with_producer(world, source);
                let mut fixed = None;
                if let Some(production) = production {
                    let production = production.in_world(world, source);
                    let multiple = production
                        .visit(|mana| fixed.replace(mana).is_some_and(|previous| previous != mana));
                    if multiple {
                        assert!(
                            matches!(
                                production,
                                ManaProduction::Choice { .. } | ManaProduction::Combination { .. }
                            ),
                            "attachment bonus requires fixed or selectable colors"
                        );
                        // An additional trigger makes its own choice, with the same producer.
                        receipt.choices.extend(std::iter::repeat_n(
                            TapBonusOption {
                                production,
                                quality: Some(quality),
                            },
                            count as usize,
                        ));
                        continue;
                    }
                }
                let scaled = |value: u32| {
                    value
                        .checked_mul(count)
                        .capacity("mana representation exhausted")
                };
                let fixed = fixed.unwrap_or_default().map(scaled);
                if fixed != [0; 6] || amount != 0 || colorless != 0 {
                    receipt.sources.push(TapBonusSource {
                        quality,
                        fixed,
                        amount: scaled(amount),
                        colorless: scaled(colorless),
                    });
                }
            }
        }
        receipt
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Default)]
pub(super) struct ManaTapReceipt {
    base: ManaTapBonus,
    sources: Vec<TapBonusSource>,
    choices: Vec<TapBonusOption>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
struct TapBonusSource {
    quality: mana_lots::Quality,
    fixed: Mana,
    amount: u32,
    colorless: u32,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default)]
pub(super) struct TapBonusOutput {
    pub total: Mana,
    pub lots: Vec<(mana_lots::Quality, Mana)>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl TapBonusOutput {
    fn add(&mut self, mana: Mana, quality: Option<mana_lots::Quality>) {
        for (total, amount) in self.total.iter_mut().zip(mana) {
            *total = total
                .checked_add(amount)
                .capacity("mana representation exhausted");
        }
        if let Some(quality) = quality
            && mana != [0; 6]
        {
            self.lots.push((quality, mana));
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TapBonusOption {
    production: ManaProduction,
    quality: Option<mana_lots::Quality>,
}

fn visit_bonus_options(
    output: &mut TapBonusOutput,
    options: &[TapBonusOption],
    emit: &mut impl FnMut(&TapBonusOutput) -> bool,
) -> bool {
    let Some((current, rest)) = options.split_first() else {
        return emit(output);
    };
    let total = output.total;
    let lot_count = output.lots.len();
    current.production.visit(|mana| {
        output.add(mana, current.quality);
        let found = visit_bonus_options(output, rest, emit);
        output.total = total;
        output.lots.truncate(lot_count);
        found
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManaTapReceipt {
    pub(super) fn has_trigger_bonuses(&self) -> bool {
        self.base.has_trigger_bonuses() || !self.sources.is_empty() || !self.choices.is_empty()
    }
    pub(super) fn replace_production(&self, mut production: Mana) -> Option<Mana> {
        if self.base.multiplier != 1 {
            for amount in &mut production {
                *amount =
                    u32::try_from(u128::from(*amount) * u128::from(self.base.multiplier)).ok()?;
            }
        }
        Some(production)
    }
    pub(super) fn unreplaced_capacity(&self, available: Mana) -> Mana {
        if self.base.multiplier == 1 {
            return available;
        }
        available.map(|amount| (u64::from(amount) / self.base.multiplier) as u32)
    }
    pub(super) fn unreplaced_required(&self, amount: u128) -> u128 {
        amount.div_ceil(u128::from(self.base.multiplier))
    }
    fn output(&self, production: Mana) -> (TapBonusOutput, Vec<TapBonusOption>) {
        let mut output = TapBonusOutput::default();
        // CR106.12a: tapping for mana must actually produce mana.
        if production == [0; 6] || !self.has_trigger_bonuses() {
            return (output, Vec::new());
        }
        let mut options = self.choices.clone();
        let mask = production
            .iter()
            .enumerate()
            .fold(0u8, |mask, (color, &amount)| {
                mask | (u8::from(amount != 0) << color)
            });
        let mut add = |mut fixed: Mana, amount, colorless, quality| {
            if production[5] != 0 {
                fixed[5] = fixed[5]
                    .checked_add(colorless)
                    .capacity("mana representation exhausted");
            }
            if amount != 0 {
                if mask.count_ones() == 1 {
                    let color = mask.trailing_zeros() as usize;
                    fixed[color] = fixed[color]
                        .checked_add(amount)
                        .capacity("mana representation exhausted");
                } else {
                    options.push(TapBonusOption {
                        production: ManaProduction::Combination { mask, amount },
                        quality,
                    });
                }
            }
            output.add(fixed, quality);
        };
        add(self.base.fixed, self.base.amount, self.base.colorless, None);
        for source in &self.sources {
            add(
                source.fixed,
                source.amount,
                source.colorless,
                Some(source.quality),
            );
        }
        (output, options)
    }

    pub(super) fn visit(
        &self,
        production: Mana,
        mut emit: impl FnMut(&TapBonusOutput) -> bool,
    ) -> bool {
        let (mut output, options) = self.output(production);
        visit_bonus_options(&mut output, &options, &mut emit)
    }

    pub(super) fn singleton(&self, production: Mana) -> Option<TapBonusOutput> {
        let (output, options) = self.output(production);
        options.is_empty().then_some(output)
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
    let kinds = if object.zone == Zone::Battlefield && object.controller == actor {
        world.kinds(id)
    } else {
        0
    };
    ManaTapBonus {
        land_at_tap: (kinds & 2 != 0).then(|| LandAtTap::from_card(world.definition(id))),
        artifact_token_at_tap: kinds & 16 != 0 && object.is_token(),
        creature_at_tap: kinds & 1 != 0,
        ..ManaTapBonus::default()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TapBonusChoice {
    pub player: Player,
    options: Vec<TapBonusOption>,
    at: usize,
    pub previous: Option<Pending>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl TapBonusChoice {
    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.options.len() * std::mem::size_of::<TapBonusOption>()
            + self.previous.as_ref().map_or(0, Pending::owned_bytes)
    }

    pub(super) fn visit_output(
        &self,
        production: Mana,
        mut emit: impl FnMut(&TapBonusOutput) -> bool,
    ) -> bool {
        let mut output = TapBonusOutput::default();
        output.add(production, self.options[self.at].quality);
        visit_bonus_options(&mut output, &self.options[self.at + 1..], &mut emit)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn add_mana_tap_event(
        &mut self,
        actor: Player,
        receipt: ManaTapReceipt,
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
        receipt: ManaTapReceipt,
        production: Mana,
    ) {
        let (output, options) = receipt.output(production);
        self.produce_tap_mana_bonus(actor, &output);
        if !options.is_empty() {
            let state = self.state.mana_lots.get_or_insert_default();
            assert!(state.tap_bonus.is_none(), "one mana event at a time");
            state.tap_bonus = Some(Box::new(TapBonusChoice {
                player: actor,
                options,
                at: 0,
                previous: None,
            }));
        }
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
    pub(super) fn mana_tap_bonus_source(&self) -> Option<ObjectRef> {
        let choice = self.state.mana_lots.as_ref()?.tap_bonus.as_ref()?;
        choice.previous.as_ref()?;
        choice.options.get(choice.at)?.quality?.producer
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
            production: choice.options[choice.at].production,
            recipients: std::array::from_fn(|i| i == choice.player.index()),
        };
        true
    }
    pub(super) fn choose_mana_tap_bonus(&mut self, production: Mana) {
        let mut choice = self
            .state
            .mana_lots
            .as_mut()
            .unwrap()
            .tap_bonus
            .take()
            .unwrap();
        let mut output = TapBonusOutput::default();
        output.add(production, choice.options[choice.at].quality);
        self.produce_tap_mana_bonus(choice.player, &output);
        choice.at += 1;
        if choice.at == choice.options.len() {
            self.state.pending = choice.previous.expect("active bonus choice");
            self.compact_mana_lots();
        } else {
            self.state.mana_lots.as_mut().unwrap().tap_bonus = Some(choice);
        }
    }
}
