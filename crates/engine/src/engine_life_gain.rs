//! Amount replacements shared by real gains and speculative mana worlds.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::StaticAbilityFragment;
use std::collections::BTreeSet;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum LifeGainReplacement {
    Add(u16),
    Double,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl LifeGainReplacement {
    fn apply(self, amount: i128) -> i128 {
        match self {
            Self::Add(extra) => amount.checked_add(i128::from(extra)),
            Self::Double => amount.checked_mul(2),
        }
        .capacity("life gain replacement exhausted")
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LifeGainReplacementEffect {
    pub source: ObjectRef,
    pub source_card: Card,
    pub ability: u16,
    pub replacement: LifeGainReplacement,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LifeGainReplacementChoice {
    pub player: Player,
    pub amount: i128,
    pub effects: Vec<LifeGainReplacementEffect>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn can_double(card: Card) -> bool {
    card.static_ability_groups()
        .iter()
        .flat_map(|group| group.fragments)
        .any(|fragment| matches!(fragment, StaticAbilityFragment::LifeGainDouble))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn replacements<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
) -> Option<Vec<LifeGainReplacementEffect>> {
    let query = characteristic_query::CharacteristicQuery::in_world(world);
    let mut effects = Vec::new();
    for &id in world.static_sources() {
        let source = world.object(id);
        if source.zone != Zone::Battlefield {
            continue;
        }
        let card = world.definition(id);
        for (ability, fragment) in card
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .enumerate()
        {
            let available = || {
                query.continuous_fragment_available(id, |candidate| {
                    std::ptr::eq(candidate, fragment)
                })
            };
            let replacement = match *fragment {
                StaticAbilityFragment::CantGainLife if available() => return None,
                StaticAbilityFragment::LifeGainAdditional(extra)
                    if source.controller == player && available() =>
                {
                    LifeGainReplacement::Add(extra)
                }
                StaticAbilityFragment::LifeGainDouble
                    if source.controller == player && available() =>
                {
                    LifeGainReplacement::Double
                }
                _ => continue,
            };
            effects.push(LifeGainReplacementEffect {
                source: world.reference(id),
                source_card: card,
                ability: u16::try_from(ability).capacity("life gain replacement index exhausted"),
                replacement,
            });
        }
    }
    Some(effects)
}

/// Each addition can occur in any gap between the doublers. Collapse equivalent
/// orders before carrying alternative life totals into projected funding.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn replacement_amounts<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    base: i128,
) -> Vec<i128> {
    if base <= 0 {
        return vec![0];
    }
    let Some(effects) = replacements(world, player) else {
        return vec![0];
    };
    let doubles = effects
        .iter()
        .filter(|effect| effect.replacement == LifeGainReplacement::Double)
        .count();
    let Some(doubled) = (0..doubles).try_fold(base, |amount, _| amount.checked_mul(2)) else {
        return Vec::new();
    };
    let mut amounts = BTreeSet::from([doubled]);
    for effect in effects {
        let LifeGainReplacement::Add(extra) = effect.replacement else {
            continue;
        };
        let contributions =
            std::iter::successors(Some(i128::from(extra)), |amount| amount.checked_mul(2))
                .take(doubles + 1)
                .collect::<Vec<_>>();
        amounts = amounts
            .iter()
            .flat_map(|amount| {
                contributions
                    .iter()
                    .filter_map(move |extra| amount.checked_add(*extra))
            })
            .collect();
    }
    amounts.into_iter().collect()
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn gain_life(&mut self, player: Player, base: i128) {
        let amount = self.plan_life_gain(player, base);
        self.commit_life_gain(player, amount);
    }

    pub(super) fn plan_life_gain(&mut self, player: Player, base: i128) -> i128 {
        if base <= 0 {
            return 0;
        }
        // A retained mana event must use its settled amount without reopening
        // an ordered decision or querying providers from a different history.
        if let Some(amount) = self.take_frozen_mana_life_gain(player, base) {
            return amount;
        }
        // A prohibition takes precedence and produces no life-gain event.
        let Some(mut effects) = replacements(self, player) else {
            self.record_cast_mana_life_gain(player, base, 0);
            return 0;
        };
        let mut amount = base;
        let mut doubles = effects
            .iter()
            .filter(|effect| effect.replacement == LifeGainReplacement::Double)
            .count();
        while doubles != 0 && doubles != effects.len() {
            let choice = self.select_life_gain_replacement(player, amount, &effects);
            let effect = effects.remove(choice);
            doubles -= usize::from(effect.replacement == LifeGainReplacement::Double);
            amount = effect.replacement.apply(amount);
        }
        // The remaining additions or doublings commute.
        for effect in effects {
            amount = effect.replacement.apply(amount);
        }
        self.record_cast_mana_life_gain(player, base, amount);
        amount
    }
}
