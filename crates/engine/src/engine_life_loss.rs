//! Amount replacements for actual and speculative life loss.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn replacement_amount<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    base: i128,
) -> i128 {
    if base <= 0 {
        return 0;
    }
    let query = characteristic_query::CharacteristicQuery::in_world(world);
    let active = world.game().state.active;
    let mut amount = base;
    for &id in world.static_sources() {
        let source = world.object(id);
        if source.zone != Zone::Battlefield
            || source.controller != active
            || source.controller == player
        {
            continue;
        }
        for fragment in world
            .definition(id)
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
        {
            if matches!(
                fragment,
                StaticAbilityFragment::OpponentLifeLossDoubleOnOwnTurn
            ) && query
                .continuous_fragment_available(id, |candidate| std::ptr::eq(candidate, fragment))
            {
                amount = amount
                    .checked_mul(2)
                    .capacity("life loss replacement exhausted");
            }
        }
    }
    amount
}
