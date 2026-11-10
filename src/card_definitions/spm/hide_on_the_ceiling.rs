pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: object(1 | 16),
            min: 0,
            max: usize::MAX,
        }),
        &[
            Instruction::TargetsExactlyX,
            Instruction::BlinkExiledTargetsUntilEndStep,
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
