// Suppression is applied to the whole frozen group before simultaneous destruction.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::RemoveAbilitiesThenDestroyCreaturesManaAtMost(
            AmountValue::ChosenX,
        )],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
