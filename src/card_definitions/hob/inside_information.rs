pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Player {
                opponent_only: true,
            },
            min: 1,
            max: 1,
        }),
        &[Instruction::ExileTargetLibraryPlayLife {
            amount: AmountValue::ChosenX,
        }],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
