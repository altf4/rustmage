pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: object(0),
            min: 0,
            max: usize::MAX,
        }),
        &[
            Instruction::TargetsExactlyX,
            Instruction::CreateCopiesOfEachTarget(AmountValue::ChosenX),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 3,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
