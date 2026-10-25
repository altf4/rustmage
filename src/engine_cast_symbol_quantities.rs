//! Retain an existing cast event only for programs consuming printed spell symbols.
use super::*;
fn amount(value: AmountValue) -> bool {
    match value {
        AmountValue::CastSpellManaSymbols(_) => true,
        AmountValue::Multiple { value, .. } => amount(*value),
        AmountValue::Sum(values) => values.iter().any(|v| amount(*v)),
        _ => false,
    }
}
pub(super) fn selected(program: &[Instruction]) -> bool {
    program.iter().any(|instruction| match *instruction {
        Instruction::Modal(modes)
        | Instruction::ModalOnce(modes)
        | Instruction::ModalOncePerTurn(modes)
        | Instruction::ModalUpToX { modes, .. } => modes.iter().any(|m| selected(m.program)),
        Instruction::SelectLibraryTo(effect)
        | Instruction::SelectLibraryThenMayTransform(effect) => {
            effect.maximum_mana_value.is_some_and(amount)
        }
        Instruction::UpToTargets(value)
        | Instruction::IfAmountAtLeast { value, .. }
        | Instruction::PutLandCards { maximum: value, .. }
        | Instruction::DiscardScaled(_, value)
        | Instruction::AddCastEntryCounters { amount: value, .. }
        | Instruction::DrawThenDiscardIfDrew { amount: value, .. }
        | Instruction::SetCreatureForm { base: value, .. }
        | Instruction::MoveCountersFromSource { amount: value, .. }
        | Instruction::DrawScaled(_, value)
        | Instruction::GainLifeScaled(_, value)
        | Instruction::LoseLifeScaled(_, value)
        | Instruction::DrainLifeScaled(value)
        | Instruction::LoseOpponentGainSame(value)
        | Instruction::DamageScaled(value)
        | Instruction::AddColorlessScaled(value)
        | Instruction::AddCombatRedScaled(value)
        | Instruction::Mobilize { amount: value, .. }
        | Instruction::CounterUnlessPayScaled { amount: value, .. }
        | Instruction::PutCountersScaled { amount: value, .. }
        | Instruction::PutPermanentCountersScaled { amount: value, .. }
        | Instruction::GrowScaled { amount: value, .. }
        | Instruction::GrowCreaturesScaled { amount: value, .. }
        | Instruction::GrowPermanentGroupScaled { amount: value, .. }
        | Instruction::CreateTokensScaled { amount: value, .. }
        | Instruction::CreateTokensWithCounters {
            counters: value, ..
        } => amount(value),
        _ => false,
    })
}
