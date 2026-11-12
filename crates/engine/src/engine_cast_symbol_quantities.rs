//! Retain a cast event for programs consuming its identity, mana value or symbols.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn amount(value: AmountValue, cave_only: bool) -> bool {
    match value {
        AmountValue::CaveManaSpent => true,
        AmountValue::CastSpellManaValue | AmountValue::CastSpellManaSymbols(_) => !cave_only,
        AmountValue::Multiple { value, .. } | AmountValue::Divided { value, .. } => {
            amount(*value, cave_only)
        }
        AmountValue::Sum(values) => values.iter().any(|v| amount(*v, cave_only)),
        _ => false,
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn selected(program: &[Instruction]) -> bool {
    selected_for(program, false)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn uses_cave_mana(card: Card) -> bool {
    card.triggers()
        .iter()
        .any(|ability| selected_for(ability.program, true))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn selected_for(program: &[Instruction], cave_only: bool) -> bool {
    program.iter().any(|instruction| match *instruction {
        Instruction::Modal(modes)
        | Instruction::ModalOnce(modes)
        | Instruction::ModalOncePerTurn(modes)
        | Instruction::ModalUpToX { modes, .. } => {
            modes.iter().any(|m| selected_for(m.program, cave_only))
        }
        Instruction::SelectLibraryTo(effect)
        | Instruction::SelectLibraryToHandRememberCount(effect)
        | Instruction::SelectLibraryThenMayTransform(effect)
        | Instruction::SelectLibraryThenReflexiveEquipment { effect, .. } => effect
            .maximum_mana_value
            .is_some_and(|value| amount(value, cave_only)),
        Instruction::AddCastEntryCounters { amount: value, .. } => {
            !cave_only || amount(value, cave_only)
        }
        Instruction::AddCastEntryCountersForControlledSubtypes { .. }
        | Instruction::GrantCastRebound
        | Instruction::GrantCastSpellHaste
        | Instruction::GrantCastSpellKeywords(_)
        | Instruction::GrantCastPlotOnResolution => !cave_only,
        Instruction::UpToTargets(value)
        | Instruction::DiscoverScaled(value)
        | Instruction::CastHandPermanentFreeUpTo(value)
        | Instruction::IfAmountAtLeast { value, .. }
        | Instruction::IfAmountParity { value, .. }
        | Instruction::PutLandCards { maximum: value, .. }
        | Instruction::DestroyTargetsAtMostManaValue(value)
        | Instruction::DamageBattlefieldScaled { amount: value, .. }
        | Instruction::DestroyManaValueExactly { amount: value, .. }
        | Instruction::DiscardScaled(_, value)
        | Instruction::SearchScaled { maximum: value, .. }
        | Instruction::DrawThenDiscardIfDrew { amount: value, .. }
        | Instruction::SetCreatureForm { base: value, .. }
        | Instruction::MoveCountersFromSource { amount: value, .. }
        | Instruction::RevealTargetHandSubsetDiscard(value)
        | Instruction::DrawScaled(_, value)
        | Instruction::DrawRemembered(value)
        | Instruction::MayMillAmount(value)
        | Instruction::Mill(_, MillAmount::Amount(value))
        | Instruction::GainLifeScaled(_, value)
        | Instruction::LoseLifeScaled(_, value)
        | Instruction::DrainLifeScaled(value)
        | Instruction::LoseOpponentGainSame(value)
        | Instruction::DamageScaled(value)
        | Instruction::DamageRememberPlayersScaled(value)
        | Instruction::DamageWithControllerScaled {
            controller_damage: value,
            ..
        }
        | Instruction::AddColorlessScaled(value)
        | Instruction::AddColorlessScaledFor(_, value)
        | Instruction::AddManaScaled { amount: value, .. }
        | Instruction::AddManaChosenScaled { amount: value, .. }
        | Instruction::AddCombatRedScaled(value)
        | Instruction::Mobilize { amount: value, .. }
        | Instruction::CounterUnlessPayScaled { amount: value, .. }
        | Instruction::PutCountersScaled { amount: value, .. }
        | Instruction::PutPermanentCountersScaled { amount: value, .. }
        | Instruction::PutPermanentCountersWithColors { amount: value, .. }
        | Instruction::PutLegendaryPermanentCountersScaled { amount: value, .. }
        | Instruction::PutNamedPermanentCountersScaled { amount: value, .. }
        | Instruction::GrowScaled { amount: value, .. }
        | Instruction::GrowFilteredScaled { amount: value, .. }
        | Instruction::GrowCreaturesScaled { amount: value, .. }
        | Instruction::GrowPermanentGroupScaled { amount: value, .. }
        | Instruction::CreateTokensScaled { amount: value, .. }
        | Instruction::CreateTokensRepeated { amount: value, .. }
        | Instruction::CreateTokensWithSourceDeparture { amount: value, .. }
        | Instruction::EndureScaled { amount: value, .. }
        | Instruction::CreateSizedToken { size: value, .. }
        | Instruction::CreateTokensWithCounters {
            counters: value, ..
        }
        | Instruction::CreateTokensThenCounters {
            counters: value, ..
        } => amount(value, cave_only),
        _ => false,
    })
}
