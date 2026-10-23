//! Generic activation discounts, queried in actual and borrowed mana worlds.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::StaticAbilityFragment;

pub(super) fn reduced<W: CharacteristicWorld + ?Sized>(
    world: &W,
    _actor: Player,
    source: CardId,
    mut cost: ManaCost,
    upper: bool,
) -> ManaCost {
    let object = world.object(source);
    if object.zone != Zone::Battlefield
        || !world.is_token(source)
        || (!upper && (world.kinds(source) & 16 == 0))
    {
        return cost;
    }
    let mut reduction = 0u32;
    for &id in &world.game().state.static_sources {
        let provider = world.object(id);
        if provider.zone != Zone::Battlefield
            || (!upper
                && (provider.controller != object.controller
                    || !world.printed_abilities_available(id)))
        {
            continue;
        }
        for group in provider.card.static_ability_groups() {
            for fragment in group.fragments {
                if let StaticAbilityFragment::ArtifactTokenActivationReduction(amount) = *fragment {
                    reduction = reduction.saturating_add(amount);
                }
            }
        }
    }
    cost.generic = cost.generic.saturating_sub(reduction);
    cost
}
impl Game {
    pub(super) fn reduced_activation_cost(
        &self,
        actor: Player,
        source: CardId,
        cost: ManaCost,
    ) -> ManaCost {
        reduced(self, actor, source, cost, false)
    }
    pub(super) fn activation_total_cost(&self, a: &Announcement, cost: ManaCost) -> ManaCost {
        let mut cost = cost;
        cost.generic = cost
            .generic
            .checked_add(if a.has_x_cost() {
                a.item.x.unwrap_or(0)
            } else {
                0
            })
            .capacity("activation cost exhausted");
        self.reduced_activation_cost(a.item.controller, a.item.object.card, cost)
    }
}

impl EffectChoiceKind {
    pub(super) fn creature_counter_requirement(self) -> Option<(u16, Option<CounterKind>)> {
        match self {
            Self::RemoveCreatureCounters(n) => Some((n, None)),
            Self::RemoveCreatureCounter(k) => Some((1, Some(k))),
            _ => None,
        }
    }
}
